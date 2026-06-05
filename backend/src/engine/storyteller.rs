use std::collections::{HashMap, VecDeque};
use rand::Rng;
use tracing::info;

/// Controls story pacing — when to inject action, rest, tension peaks.
///
/// The Storyteller manages narrative rhythm by tracking tension levels,
/// enforcing recovery periods after major events, detecting boredom/stalemate,
/// and adapting event probabilities to maintain variety.
pub struct Storyteller {
    pub tick: u64,
    pub tension: f64,
    pub target_tension: f64,
    pub last_event_tick: u64,
    pub event_cooldown: u64,
    pub tension_decay_rate: f64,
    pub story_phase: StoryPhase,
    pub recovery: RecoveryManager,
    pub boredom: BoredomDetector,
    pub recent_event_types: VecDeque<String>,
    pub adaptation: HashMap<String, f64>,
}

/// Narrative phase derived from current tension level.
#[derive(Debug, Clone, PartialEq)]
pub enum StoryPhase {
    /// Low tension, character moments, exploration.
    Calm,
    /// Tension building toward a crisis.
    Rising,
    /// Crisis point — maximum narrative pressure.
    Peak,
    /// Resolution after a peak.
    Falling,
    /// Back to calm after full resolution.
    Reset,
}

impl Storyteller {
    /// Create a new Storyteller with default pacing parameters.
    pub fn new() -> Self {
        Self {
            tick: 0,
            tension: 0.0,
            target_tension: 0.2,
            last_event_tick: 0,
            event_cooldown: 5,
            tension_decay_rate: 0.02,
            story_phase: StoryPhase::Calm,
            recovery: RecoveryManager::new(),
            boredom: BoredomDetector::new(),
            recent_event_types: VecDeque::new(),
            adaptation: HashMap::new(),
        }
    }

    /// Advance one tick. Adjusts tension toward target, updates recovery and phase.
    pub fn update(&mut self) {
        self.tick += 1;
        self.recovery.tick();
        self.boredom.tick_quiet();

        // Move tension toward target
        if (self.tension - self.target_tension).abs() < 0.001 {
            // already at target — no-op
        } else if self.tension < self.target_tension {
            self.tension = (self.tension + 0.05).min(self.target_tension);
        } else {
            self.tension = (self.tension - self.tension_decay_rate).max(self.target_tension);
        }

        // Determine story phase from tension level
        let new_phase = match self.tension {
            t if t < 0.2 => StoryPhase::Calm,
            t if t < 0.5 => StoryPhase::Rising,
            t if t < 0.8 => StoryPhase::Peak,
            t if t < 0.95 => StoryPhase::Falling,
            _ => StoryPhase::Reset,
        };

        if new_phase != self.story_phase {
            info!(
                "storyteller: phase transition {:?} -> {:?} at tick {} (tension={:.3})",
                self.story_phase, new_phase, self.tick, self.tension
            );
            self.story_phase = new_phase;
        }
    }

    /// Returns `true` if a new event should be injected this tick.
    ///
    /// Considers cooldown (minimum ticks since last event) and recovery
    /// period (probability modifier after major events).
    pub fn should_inject_event(&self) -> bool {
        if self.tick < self.last_event_tick + self.event_cooldown {
            return false;
        }
        if !self.recovery.event_allowed() {
            let mut rng = rand::thread_rng();
            let modifier = self.recovery.event_probability_modifier();
            return rng.gen::<f64>() < modifier;
        }
        true
    }

    /// Record that an event was triggered. Raises tension and starts cooldown.
    pub fn on_event_triggered(&mut self) {
        self.tension = (self.tension + 0.15).min(1.0);
        self.last_event_tick = self.tick;
        self.boredom.reset();

        let severity = if self.tension > 0.7 {
            "major"
        } else if self.tension > 0.4 {
            "normal"
        } else {
            "minor"
        };
        self.recovery.trigger_recovery(severity);
        info!(
            "storyteller: event triggered at tick {}, tension={:.3}, phase={:?}",
            self.tick, self.tension, self.story_phase
        );
    }

    /// Set the target tension level (clamped to 0.0–1.0).
    pub fn set_target(&mut self, target: f64) {
        self.target_tension = target.clamp(0.0, 1.0);
        info!("storyteller: target_tension set to {:.3}", self.target_tension);
    }

    /// Register that an event type occurred (used for novelty/adaptation tracking).
    pub fn record_event_type(&mut self, event_type: &str) {
        self.recent_event_types.push_back(event_type.to_string());
        if self.recent_event_types.len() > 20 {
            self.recent_event_types.pop_front();
        }
        let entry = self.adaptation.entry(event_type.to_string()).or_insert(0.0);
        *entry += 0.1;
    }

    /// Delegate boredom check to the internal BoredomDetector.
    pub fn is_bored(&self, quiet_ticks: u64, state_delta: f64, graph_delta: f64) -> bool {
        self.boredom.is_bored(quiet_ticks, state_delta, graph_delta)
    }
}

impl Default for Storyteller {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// RecoveryManager
// ---------------------------------------------------------------------------

/// Manages the quiet period after significant events.
///
/// During recovery, new events have reduced probability that gradually
/// returns to normal following an ease-out cosine curve.
pub struct RecoveryManager {
    pub current: Option<RecoveryPeriod>,
}

/// A single recovery period triggered by an event of a given severity.
#[derive(Debug, Clone)]
pub struct RecoveryPeriod {
    pub reason: String,
    pub remaining_ticks: u64,
    pub max_ticks: u64,
}

impl RecoveryManager {
    pub fn new() -> Self {
        Self { current: None }
    }

    /// Start a recovery period. Ticks depend on severity:
    /// - critical: 48 ticks
    /// - major: 24 ticks
    /// - normal: 8 ticks
    /// - minor/other: 3 ticks
    pub fn trigger_recovery(&mut self, severity: &str) {
        let ticks = match severity {
            "critical" => 48,
            "major" => 24,
            "normal" => 8,
            _ => 3,
        };
        self.current = Some(RecoveryPeriod {
            reason: format!("{} severity event recovery", severity),
            remaining_ticks: ticks,
            max_ticks: ticks,
        });
        info!("storyteller: recovery period started ({} ticks)", ticks);
    }

    /// Probability modifier for new events during recovery (0.0–1.0).
    ///
    /// Follows an ease-out cosine curve: starts low, accelerates toward full recovery.
    pub fn event_probability_modifier(&self) -> f64 {
        self.current.as_ref().map_or(1.0, |r| {
            let progress = 1.0 - r.remaining_ticks as f64 / r.max_ticks as f64;
            1.0 - (progress * std::f64::consts::FRAC_PI_2).cos()
        })
    }

    /// Returns `true` if events are fully allowed (no active recovery).
    pub fn event_allowed(&self) -> bool {
        self.current.is_none()
    }

    /// Advance recovery by one tick. Removes the period when expired.
    pub fn tick(&mut self) {
        if let Some(ref mut r) = self.current {
            r.remaining_ticks = r.remaining_ticks.saturating_sub(1);
            if r.remaining_ticks == 0 {
                info!("storyteller: recovery period ended");
                self.current = None;
            }
        }
    }
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// BoredomDetector
// ---------------------------------------------------------------------------

/// Detects when the simulation has fallen into a narrative rut.
///
/// Boredom is determined by three factors: consecutive quiet ticks,
/// low state change delta, and low relationship graph delta.
pub struct BoredomDetector {
    pub consecutive_quiet_ticks: u64,
}

impl BoredomDetector {
    pub fn new() -> Self {
        Self { consecutive_quiet_ticks: 0 }
    }

    /// Returns `true` if the simulation appears stuck.
    ///
    /// Scoring (threshold >= 0.7):
    /// - quiet_ticks > 10: +0.4
    /// - state_delta < 0.5: +0.3
    /// - graph_delta < 0.05: +0.3
    pub fn is_bored(&self, quiet_ticks: u64, state_delta: f64, graph_delta: f64) -> bool {
        let mut score = 0.0;
        if quiet_ticks > 10 {
            score += 0.4;
        }
        if state_delta < 0.5 {
            score += 0.3;
        }
        if graph_delta < 0.05 {
            score += 0.3;
        }
        score >= 0.7
    }

    /// Reset the quiet-tick counter (called when an event fires).
    pub fn reset(&mut self) {
        self.consecutive_quiet_ticks = 0;
    }

    /// Increment the quiet-tick counter.
    pub fn tick_quiet(&mut self) {
        self.consecutive_quiet_ticks = self.consecutive_quiet_ticks.saturating_add(1);
    }
}

impl Default for BoredomDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// BoredomBreaker
// ---------------------------------------------------------------------------

/// Generates a boredom-breaking event by finding the most reasonable escalation path.
///
/// Priority order:
/// 1. Escalate existing social tension
/// 2. Escalate a resource crisis
/// 3. Introduce an external stimulus if everyone is home
pub struct BoredomBreaker;

impl BoredomBreaker {
    /// Attempt to generate an event that breaks the current stalemate.
    pub fn break_boredom(
        has_social_tension: bool,
        has_resource_crisis: bool,
        all_at_home: bool,
        first_char_id: Option<String>,
    ) -> Option<GeneratedEvent> {
        if has_social_tension {
            return Some(GeneratedEvent {
                event_type: "social_tension_escalation".into(),
                description: "Unresolved social tension escalates into open conflict.".into(),
                severity: "normal".into(),
                participants: vec![],
            });
        }
        if has_resource_crisis {
            return Some(GeneratedEvent {
                event_type: "resource_shortage_crisis".into(),
                description: "Sustained resource consumption triggers a shortage event.".into(),
                severity: "major".into(),
                participants: vec![],
            });
        }
        if all_at_home {
            if let Some(char_id) = first_char_id {
                return Some(GeneratedEvent {
                    event_type: "external_stimulus".into(),
                    description: "A strange sound is heard in the distance.".into(),
                    severity: "minor".into(),
                    participants: vec![char_id],
                });
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// GeneratedEvent (internal)
// ---------------------------------------------------------------------------

/// A minimal event type produced internally by the Storyteller subsystem.
#[derive(Debug, Clone)]
pub struct GeneratedEvent {
    pub event_type: String,
    pub description: String,
    pub severity: String,
    pub participants: Vec<String>,
}

impl GeneratedEvent {
    /// Narrative "cost" of this event, used for threat-point consumption.
    pub fn cost(&self) -> f64 {
        match self.severity.as_str() {
            "critical" => 5.0,
            "major" => 3.0,
            "normal" => 1.5,
            _ => 0.5,
        }
    }
}
