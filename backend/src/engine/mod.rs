pub mod simulation_loop;
pub mod timeline;
pub mod world_state;

// M2 event subsystems (doc 12)
pub mod storyteller;
pub mod emergent_detector;
pub mod cascade_engine;
pub mod event_memory;
pub mod probability_tree;
pub mod narrative_filter;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, RwLock};

pub use crate::models::timeline::SimSpeed;
use crate::models::world::{TagHeatmap, TagThread, WorldState};
use crate::llm::gateway::LLMGateway;

// ---------------------------------------------------------------------------
// Engine state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineState {
    Paused,
    Running { speed: SimSpeed, tick_count: u64 },
    Stopped,
}

// ---------------------------------------------------------------------------
// Engine config (runtime) — re-export from crate::config
// ---------------------------------------------------------------------------

/// Engine configuration is defined in `crate::config::EngineConfig` to
/// guarantee a single source of truth across YAML deserialization and
/// engine runtime.  Fields specific to the engine (max_concurrent_llm,
/// llm_timeout_seconds, checkpoint_interval) live in the same struct so
/// they are always synchronised.
pub use crate::config::EngineConfig;

// ---------------------------------------------------------------------------
// Engine stats
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub total_ticks: u64,
    pub fastforward_ticks: u64,
    pub detailed_ticks: u64,
    pub total_llm_calls: u64,
    pub llm_calls_this_step: u64,
    pub characters_active: usize,
    pub events_triggered: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub wall_clock_elapsed: chrono::Duration,
}

/// Lock-free atomics for engine stats.
#[derive(Debug)]
pub struct AtomicEngineStats {
    pub total_ticks: AtomicU64,
    pub fastforward_ticks: AtomicU64,
    pub detailed_ticks: AtomicU64,
    pub total_llm_calls: AtomicU64,
    pub llm_calls_this_step: AtomicU64,
    pub characters_active: AtomicU64,
    pub events_triggered: AtomicU64,
}

impl Default for AtomicEngineStats {
    fn default() -> Self {
        Self {
            total_ticks: AtomicU64::new(0),
            fastforward_ticks: AtomicU64::new(0),
            detailed_ticks: AtomicU64::new(0),
            total_llm_calls: AtomicU64::new(0),
            llm_calls_this_step: AtomicU64::new(0),
            characters_active: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }
}

// ---------------------------------------------------------------------------
// LLM request/response types (shared between engine and llm modules)
// ---------------------------------------------------------------------------

pub type LLMRequestSender = mpsc::Sender<LLMRequest>;

#[derive(Debug)]
pub struct LLMRequest {
    pub char_id: uuid::Uuid,
    pub messages: Vec<LLMMessage>,
    pub response_tx: tokio::sync::oneshot::Sender<Result<LLMResponse, LLMError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub content: String,
    pub usage: TokenUsage,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum LLMError {
    #[error("LLM timeout")]
    Timeout,
    #[error("Rate limited")]
    RateLimited,
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Circuit breaker open")]
    CircuitBreakerOpen,
}

// ---------------------------------------------------------------------------
// Engine event (WS broadcast)
// ---------------------------------------------------------------------------

pub type BroadcastSender = broadcast::Sender<EngineEvent>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineEvent {
    Heartbeat,
    Tick {
        tick: u64,
        time: String,
        speed: SimSpeed,
    },
    DetailedTick {
        tick: u64,
        time: String,
        decisions: Vec<DecisionSummary>,
        events: Vec<crate::models::event::EventSummary>,
    },
    SpeedChanged {
        from: SimSpeed,
        to: SimSpeed,
        reason: String,
    },
    CouncilCompleted {
        round: u64,
        summary: String,
        cost: f64,
    },
    CharacterUpdate {
        char_id: uuid::Uuid,
        name: String,
        state_diff: crate::models::world::StateDiff,
    },
    CharacterAdded {
        char_id: uuid::Uuid,
        name: String,
        location: String,
    },
    CharacterRemoved {
        char_id: uuid::Uuid,
        name: String,
        reason: String,
    },
    EventTriggered {
        event_id: uuid::Uuid,
        event_type: String,
        title: String,
        severity: String,
        participants: Vec<String>,
        location_id: String,
    },
    RelationshipChanged {
        char_a: uuid::Uuid,
        char_b: uuid::Uuid,
        delta: crate::models::relationship::RelationshipDelta,
        new_values: crate::models::relationship::RelationshipValues,
        reason: String,
    },
    TagUpdated {
        thread_id: uuid::Uuid,
        status: String,
        freshness: String,
    },
    LLMCall {
        char_name: String,
        phase: String,
        latency_ms: u64,
        tokens: u32,
    },
    EnvironmentUpdate {
        temperature: f64,
        weather: String,
        season: String,
        daylight: f64,
    },
    Log {
        level: String,
        message: String,
        module: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionSummary {
    pub char_id: uuid::Uuid,
    pub char_name: String,
    pub action_type: String,
    pub description: String,
}

// ---------------------------------------------------------------------------
// TagIndex (placeholder for M4)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct TagIndex {
    pub heatmap: TagHeatmap,
    pub threads: Vec<TagThread>,
}

// ---------------------------------------------------------------------------
// SimulationEngine — main engine struct
// ---------------------------------------------------------------------------

pub struct SimulationEngine {
    pub world: RwLock<WorldState>,
    pub tag_index: RwLock<TagIndex>,
    pub config: RwLock<EngineConfig>,
    pub stats: AtomicEngineStats,
    pub llm_tx: mpsc::Sender<LLMRequest>,
    pub state: std::sync::Mutex<EngineState>,
    pub consecutive_idle: std::sync::atomic::AtomicU8,
    pub is_stepping: std::sync::atomic::AtomicBool,
    pub ws_broadcaster: BroadcastSender,
    pub llm_gateway: Arc<LLMGateway>,
}

impl SimulationEngine {
    pub fn new(
        world: WorldState,
        config: EngineConfig,
        llm_tx: mpsc::Sender<LLMRequest>,
        ws_broadcaster: BroadcastSender,
        llm_gateway: Arc<LLMGateway>,
    ) -> Self {
        Self {
            world: RwLock::new(world),
            tag_index: RwLock::new(TagIndex::default()),
            config: RwLock::new(config),
            stats: AtomicEngineStats::default(),
            llm_tx,
            state: std::sync::Mutex::new(EngineState::Paused),
            consecutive_idle: std::sync::atomic::AtomicU8::new(0),
            is_stepping: std::sync::atomic::AtomicBool::new(false),
            ws_broadcaster,
            llm_gateway,
        }
    }

    pub fn current_tick(&self) -> u64 {
        self.stats.total_ticks.load(Ordering::Relaxed)
    }

    pub fn snapshot_stats(&self) -> EngineStats {
        let now = chrono::Utc::now();
        EngineStats {
            total_ticks: self.stats.total_ticks.load(Ordering::Relaxed),
            fastforward_ticks: self.stats.fastforward_ticks.load(Ordering::Relaxed),
            detailed_ticks: self.stats.detailed_ticks.load(Ordering::Relaxed),
            total_llm_calls: self.stats.total_llm_calls.load(Ordering::Relaxed),
            llm_calls_this_step: self.stats.llm_calls_this_step.load(Ordering::Relaxed),
            characters_active: self.stats.characters_active.load(Ordering::Relaxed) as usize,
            events_triggered: self.stats.events_triggered.load(Ordering::Relaxed),
            start_time: now,
            wall_clock_elapsed: chrono::Duration::zero(),
        }
    }
}

// Re-export inner modules
pub use simulation_loop::run_simulation;
