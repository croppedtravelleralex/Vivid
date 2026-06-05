use std::collections::HashMap;

use chrono::NaiveDateTime;
use hecs::World as EcsWorld;
use petgraph::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::character::CharacterStateData;
use super::event::{ConditionTrigger, EventQueue, HistoricalEvent, ScheduledEvent};
use super::relationship::{RelationshipEdge, RelationshipNode};
use crate::models::timeline::{SimTime, TimelineManager};

// ---------------------------------------------------------------------------
// EnvironmentState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    pub temperature: f64,
    pub weather: String,
    pub season: String,
    pub daylight: f64,
    pub base_temperature: f64,
    pub global_cooling_rate: f64,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self {
            temperature: -1.2,
            weather: "cloudy".into(),
            season: "winter".into(),
            daylight: 0.5,
            base_temperature: 5.0,
            global_cooling_rate: -0.08,
        }
    }
}

impl EnvironmentState {
    pub fn update(&mut self, tick: u64, time: &SimTime) {
        let month = time.month();
        let hour = time.hour();

        // Seasonal base temperature
        self.season = time.season().to_string();
        let season_base = match month {
            12 | 1 | 2 => 5.0,
            3 | 4 | 5 => 12.0,
            6 | 7 | 8 => 30.0,
            9 | 10 | 11 => 22.0,
            _ => 5.0,
        };

        // Diurnal variation
        let day_factor = if hour >= 6 && hour <= 18 {
            ((hour as f64 - 6.0) / 12.0 * std::f64::consts::PI).sin()
        } else {
            0.0
        };

        // Global cooling
        let cooling = self.global_cooling_rate * (tick as f64 / 288.0); // ~ticks per day

        self.temperature = season_base + day_factor * 3.0 + cooling;
        self.daylight = if hour >= 6 && hour <= 18 {
            (hour as f64 - 6.0) / 12.0
        } else {
            0.0
        };
    }
}

// ---------------------------------------------------------------------------
// Location types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationNode {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: LocationCategory,
    pub condition: f64,
    pub max_occupancy: u32,
    pub tags: Vec<String>,
    pub resources: HashMap<String, ResourceStock>,
    pub position: Option<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationEdge {
    pub distance_km: f64,
    pub difficulty: TravelDifficulty,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationCategory {
    Shelter,
    Resource,
    Danger,
    Transit,
    Residential,
    Commercial,
    Medical,
    Industrial,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TravelDifficulty {
    Easy,
    Moderate,
    Hard,
    Dangerous,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStock {
    pub current: f64,
    pub max: f64,
    pub unit: String,
    pub daily_consumption: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

// ---------------------------------------------------------------------------
// Graph types for API responses
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub name: String,
    pub node_type: String,
    pub group: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: Uuid,
    pub target: Uuid,
    pub weight: f64,
    pub label: String,
}

// ---------------------------------------------------------------------------
// Event system (embedded in WorldState)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EventSystem {
    pub queue: EventQueue,
    pub scheduled_events: Vec<ScheduledEvent>,
    pub condition_triggers: Vec<ConditionTrigger>,
}

impl Default for EventSystem {
    fn default() -> Self {
        Self {
            queue: EventQueue::default(),
            scheduled_events: vec![],
            condition_triggers: vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// Narrative directive (from council)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NarrativeDirective {
    pub focus_character: Option<String>,
    pub suggested_event: Option<String>,
    pub tension_adjustment: Option<f64>,
}

// ---------------------------------------------------------------------------
// Resource map
// ---------------------------------------------------------------------------

pub type ResourceMap = HashMap<String, f64>;

pub struct WorldState {
    pub timeline: TimelineManager,
    pub environment: EnvironmentState,
    pub characters: EcsWorld,
    pub relationships: Graph<RelationshipNode, RelationshipEdge>,
    pub locations: Vec<LocationNode>,
    pub location_graph: Graph<usize, LocationEdge>,
    pub event_system: EventSystem,
    pub resources: ResourceMap,
    pub narrative_directive: Option<NarrativeDirective>,
    pub random_seed: u64,
}

impl std::fmt::Debug for WorldState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldState")
            .field("timeline", &self.timeline)
            .field("environment", &self.environment)
            .field("character_count", &self.character_count())
            .field("location_count", &self.location_count())
            .field("resources", &self.resources)
            .field("random_seed", &self.random_seed)
            .finish()
    }
}

impl WorldState {
    pub fn new(start_time: NaiveDateTime, seed: u64) -> Self {
        Self {
            timeline: TimelineManager::new(start_time),
            environment: EnvironmentState::default(),
            characters: EcsWorld::new(),
            relationships: Graph::new(),
            locations: vec![],
            location_graph: Graph::new(),
            event_system: EventSystem::default(),
            resources: ResourceMap::new(),
            narrative_directive: None,
            random_seed: seed,
        }
    }

    pub fn character_count(&self) -> usize {
        self.characters.iter().count()
    }

    pub fn location_count(&self) -> usize {
        self.locations.len()
    }

    /// Apply a decision from LLM output (placeholder logic).
    pub fn apply_decision(&mut self, _decision: &serde_json::Value) {
        // TODO: M2 - decode decision and update world state
    }
}

// ---------------------------------------------------------------------------
// WorldSummary (API response)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSummary {
    pub tick: u64,
    pub time: SimTime,
    pub character_count: usize,
    pub location_count: usize,
    pub temperature: f64,
    pub weather: String,
    pub season: String,
    pub resources: HashMap<String, f64>,
}

// ---------------------------------------------------------------------------
// ECS Components (used with hecs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterState {
    pub hp: f64,
    pub max_hp: f64,
    pub hunger: f64,
    pub warmth: f64,
    pub fatigue: f64,
    pub mental: f64,
    pub stress: f64,
    pub location: Option<Uuid>,
    pub is_idle: bool,
}

impl Default for CharacterState {
    fn default() -> Self {
        Self {
            hp: 100.0,
            max_hp: 100.0,
            hunger: 0.0,
            warmth: 100.0,
            fatigue: 0.0,
            mental: 100.0,
            stress: 0.0,
            location: None,
            is_idle: false,
        }
    }
}

impl From<CharacterStateData> for CharacterState {
    fn from(d: CharacterStateData) -> Self {
        Self {
            hp: d.hp,
            max_hp: d.max_hp,
            hunger: d.hunger,
            warmth: d.warmth,
            fatigue: d.fatigue,
            mental: d.mental,
            stress: d.stress,
            location: d.location,
            is_idle: d.is_idle,
        }
    }
}

// ---------------------------------------------------------------------------
// StateDiff (WS)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    pub hp: Option<f64>,
    pub hunger: Option<f64>,
    pub warmth: Option<f64>,
    pub fatigue: Option<f64>,
    pub mental: Option<f64>,
    pub stress: Option<f64>,
}

// ---------------------------------------------------------------------------
// Dashboard summary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub tick: u64,
    pub time: String,
    pub speed: String,
    pub characters: usize,
    pub locations: usize,
    pub events_triggered: u64,
    pub temperature: f64,
    pub weather: String,
    pub season: String,
    pub total_llm_calls: u64,
}

// ---------------------------------------------------------------------------
// Search results
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub characters: Vec<super::character::CharacterSummary>,
    pub events: Vec<HistoricalEvent>,
    pub total: usize,
}

// ---------------------------------------------------------------------------
// Tag types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagHeatmap {
    pub tags: Vec<TagHeatEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagHeatEntry {
    pub tag: String,
    pub count: u32,
    pub freshness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagThread {
    pub id: Uuid,
    pub title: String,
    pub tags: Vec<String>,
    pub status: String,
    pub freshness: f64,
    pub related_characters: Vec<String>,
}
