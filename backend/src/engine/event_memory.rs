use std::collections::HashSet;
use rand::Rng;
use tracing::info;

/// Tracks events that have occurred and their "activation" level over time.
///
/// Activation follows an Ebbinghaus-style forgetting curve: recent events
/// are highly active and can trigger cascades; old events fade toward zero.
pub struct EventMemory {
    pub events: Vec<EventRecord>,
    pub max_events: usize,
}

/// A recorded event with its current activation level.
#[derive(Debug, Clone)]
pub struct EventRecord {
    pub id: String,
    pub tick: u64,
    pub event_type: String,
    pub title: String,
    pub participants: Vec<String>,
    pub activation: f64,
    pub last_recalled_tick: u64,
}

impl EventMemory {
    /// Create a new event memory store.
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::with_capacity(max_events.min(100)),
            max_events,
        }
    }

    /// Record a new event with activation starting at 1.0.
    pub fn record(
        &mut self,
        id: &str,
        tick: u64,
        event_type: &str,
        title: &str,
        participants: &[String],
    ) {
        if self.events.len() >= self.max_events {
            // Evict the oldest event
            self.events.remove(0);
        }

        self.events.push(EventRecord {
            id: id.to_string(),
            tick,
            event_type: event_type.to_string(),
            title: title.to_string(),
            participants: participants.to_vec(),
            activation: 1.0,
            last_recalled_tick: tick,
        });

        info!(
            "event_memory: recorded '{}' (type={}) at tick {}",
            title, event_type, tick
        );
    }

    /// Return all events whose activation is above the given threshold.
    pub fn recall(&self, min_activation: f64) -> Vec<&EventRecord> {
        self.events
            .iter()
            .filter(|e| e.activation >= min_activation)
            .collect()
    }

    /// Apply decay to all events based on the Ebbinghaus forgetting curve.
    ///
    /// `decay_rate` is the half-life factor: higher = faster forgetting.
    /// Activation = e^(-age / adjusted_half_life)
    pub fn decay(&mut self, current_tick: u64, decay_rate: f64) {
        for event in &mut self.events {
            let age = current_tick.saturating_sub(event.tick);
            if age == 0 {
                event.activation = 1.0;
                continue;
            }

            let adjusted = 100.0 * (1.0 + decay_rate);
            event.activation = (-(age as f64) / adjusted).exp();
        }

        // Remove fully decayed events (activation < 0.01)
        self.events.retain(|e| e.activation >= 0.01);

        // Keep within max_events bound
        while self.events.len() > self.max_events {
            self.events.remove(0);
        }
    }

    /// Refresh (boost) an event's activation, e.g. when it's mentioned again.
    pub fn refresh(&mut self, id: &str, tick: u64) {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == id) {
            event.activation = (event.activation + 1.0).min(1.0);
            event.last_recalled_tick = tick;
            info!("event_memory: refreshed '{}'", id);
        }
    }

    /// Check if an event exists by ID.
    pub fn has_event(&self, id: &str) -> bool {
        self.events.iter().any(|e| e.id == id)
    }

    /// Get a specific event by ID.
    pub fn get(&self, id: &str) -> Option<&EventRecord> {
        self.events.iter().find(|e| e.id == id)
    }
}

impl Default for EventMemory {
    fn default() -> Self {
        Self::new(5000)
    }
}

// ---------------------------------------------------------------------------
// ForgettingCurve
// ---------------------------------------------------------------------------

/// Ebbinghaus-style forgetting curve for computing event activation.
///
/// Activation decays exponentially with age, adjusted by event poignancy
/// (more poignant events are remembered longer).
pub struct ForgettingCurve {
    pub half_life_ticks: u64,
    pub importance_factor: f64,
}

impl ForgettingCurve {
    pub fn new(half_life_ticks: u64, importance_factor: f64) -> Self {
        Self {
            half_life_ticks,
            importance_factor,
        }
    }

    /// Compute activation (0.0–1.0) for an event at the current tick.
    ///
    /// A freshly occurred event has activation 1.0. Older events decay
    /// exponentially based on age and the event's poignancy.
    pub fn activation(&self, occurred_at_tick: u64, poignancy: f64, current_tick: u64) -> f64 {
        let age = current_tick.saturating_sub(occurred_at_tick);
        if age == 0 {
            return 1.0;
        }

        let adjusted = self.half_life_ticks as f64 * (1.0 + poignancy * self.importance_factor);
        (-(age as f64) / adjusted).exp()
    }

    /// Filter a slice of events, returning only those with activation >= 0.2.
    pub fn active_events<'a>(
        &self,
        events: &'a [HistoricalEvent],
        current_tick: u64,
    ) -> Vec<&'a HistoricalEvent> {
        events
            .iter()
            .filter(|e| self.activation(e.occurred_at_tick, e.poignancy, current_tick) >= 0.2)
            .collect()
    }
}

impl Default for ForgettingCurve {
    fn default() -> Self {
        Self::new(100, 1.5)
    }
}

/// Minimal historical event record for use with the forgetting curve.
#[derive(Debug, Clone)]
pub struct HistoricalEvent {
    pub id: String,
    pub occurred_at_tick: u64,
    pub poignancy: f64,
    pub severity: String,
}

// ---------------------------------------------------------------------------
// StatuteOfLimitations
// ---------------------------------------------------------------------------

/// Locks events after a time limit so they no longer produce cascades.
///
/// The limit is `base_ticks + severity_extension` where severity_extension
/// scales with event severity (critical = longest).
pub struct StatuteOfLimitations {
    pub base_ticks: u64,
    pub locked: HashSet<String>,
}

impl StatuteOfLimitations {
    pub fn new(base_ticks: u64) -> Self {
        Self {
            base_ticks,
            locked: HashSet::new(),
        }
    }

    /// Check an event and lock it if it has exceeded its statute of limitations.
    ///
    /// Returns `true` if the event was newly locked.
    pub fn check_and_lock(
        &mut self,
        event_id: &str,
        occurred_at_tick: u64,
        severity: &str,
        current_tick: u64,
    ) -> bool {
        let extension = match severity {
            "critical" => 1000,
            "major" => 500,
            "normal" => 200,
            _ => 50,
        };
        let limit = self.base_ticks + extension;

        if current_tick.saturating_sub(occurred_at_tick) >= limit {
            self.locked.insert(event_id.to_string());
            info!(
                "event_memory: locked event '{}' (statute expired at tick {})",
                event_id,
                occurred_at_tick + limit
            );
            true
        } else {
            false
        }
    }

    /// Returns `false` if the event has been locked.
    pub fn can_cascade(&self, event_id: &str) -> bool {
        !self.locked.contains(event_id)
    }

    /// Remove an event from the locked set (if it needs to be re-activated).
    pub fn unlock(&mut self, event_id: &str) {
        self.locked.remove(event_id);
    }
}

impl Default for StatuteOfLimitations {
    fn default() -> Self {
        Self::new(300)
    }
}

// ---------------------------------------------------------------------------
// UnfinishedBusiness
// ---------------------------------------------------------------------------

/// Tracks pending promises, unfulfilled commitments, and unresolved matters.
///
/// Each pending item has a per-tick probability of being recalled.
/// If a deadline passes without resolution, a "promise broken" event triggers.
pub struct UnfinishedBusiness {
    pub pending: Vec<PendingPromise>,
}

/// A promise or commitment that has not yet been fulfilled.
#[derive(Debug, Clone)]
pub struct PendingPromise {
    pub id: String,
    pub promiser: String,
    pub promisee: String,
    pub content: String,
    pub deadline_tick: Option<u64>,
    pub reminder_prob: f64,
    pub tick_created: u64,
}

impl UnfinishedBusiness {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    /// Add a new pending promise.
    pub fn add_promise(
        &mut self,
        id: &str,
        promiser: &str,
        promisee: &str,
        content: &str,
        deadline_tick: Option<u64>,
        reminder_prob: f64,
        tick: u64,
    ) {
        self.pending.push(PendingPromise {
            id: id.to_string(),
            promiser: promiser.to_string(),
            promisee: promisee.to_string(),
            content: content.to_string(),
            deadline_tick,
            reminder_prob,
            tick_created: tick,
        });
        info!(
            "event_memory: new promise '{}' from {} to {}",
            content, promiser, promisee
        );
    }

    /// Process one tick. Returns any events generated (reminders or broken promises).
    pub fn tick(&mut self, current_tick: u64) -> Vec<UnfinishedEvent> {
        let mut events = Vec::new();
        let mut rng = rand::thread_rng();
        let mut to_remove = Vec::new();

        for (i, item) in self.pending.iter().enumerate() {
            // Check deadline
            if let Some(deadline) = item.deadline_tick {
                if current_tick > deadline {
                    events.push(UnfinishedEvent {
                        id: item.id.clone(),
                        event_type: "promise_broken".to_string(),
                        description: format!(
                            "{} failed to fulfill their promise to {}: {}",
                            item.promiser, item.promisee, item.content
                        ),
                        participants: vec![item.promiser.clone(), item.promisee.clone()],
                    });
                    to_remove.push(i);
                    info!(
                        "event_memory: promise broken '{}' by {} (deadline passed)",
                        item.content, item.promiser
                    );
                    continue;
                }
            }

            // Probability reminder
            if rng.gen::<f64>() < item.reminder_prob {
                events.push(UnfinishedEvent {
                    id: item.id.clone(),
                    event_type: "reminder".to_string(),
                    description: format!(
                        "{} is reminded that {} hasn't fulfilled their promise: {}",
                        item.promisee, item.promiser, item.content
                    ),
                    participants: vec![item.promisee.clone()],
                });
            }
        }

        // Remove fulfilled/expired promises (in reverse order)
        for &i in to_remove.iter().rev() {
            if i < self.pending.len() {
                self.pending.swap_remove(i);
            }
        }

        events
    }

    /// Mark a promise as fulfilled and remove it from the pending list.
    pub fn fulfill(&mut self, id: &str) -> bool {
        if let Some(pos) = self.pending.iter().position(|p| p.id == id) {
            self.pending.swap_remove(pos);
            info!("event_memory: promise '{}' fulfilled", id);
            true
        } else {
            false
        }
    }

    /// Get all pending promises for a particular character.
    pub fn promises_by(&self, char_name: &str) -> Vec<&PendingPromise> {
        self.pending
            .iter()
            .filter(|p| p.promiser == char_name)
            .collect()
    }
}

impl Default for UnfinishedBusiness {
    fn default() -> Self {
        Self::new()
    }
}

/// An event generated by the unfinished business system.
#[derive(Debug, Clone)]
pub struct UnfinishedEvent {
    pub id: String,
    pub event_type: String,
    pub description: String,
    pub participants: Vec<String>,
}
