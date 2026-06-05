use std::cmp::Ordering;
use std::collections::BinaryHeap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::timeline::SimTime;

// ---------------------------------------------------------------------------
// Event type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EventType {
    // Environment
    TemperatureDrop(f64),
    Blizzard,
    WaterPipeFreeze,
    // Character
    CharacterArrival(Uuid),
    CharacterDeparture(Uuid),
    CharacterInjury(Uuid),
    CharacterDiscovery(Uuid, String),
    // Social
    Conflict(Uuid, Uuid),
    Alliance(Uuid, Uuid),
    Betrayal(Uuid, Uuid),
    // Resource
    ResourceFound(String, f64),
    ResourceDepleted(String),
    // Plot
    PlotAdvance(String),
    // Narrative system (doc 12)
    PromiseBroken(Uuid, Uuid, String),
    Reminder(Uuid, String),
    ExternalStimulus(String),
    StressCascade(Uuid, Vec<Uuid>),
    ResourceConflict(String, Uuid, Uuid),
    // Extension
    #[serde(skip)]
    Custom(String),
}

impl EventType {
    /// Human-readable API name (snake_case).
    pub fn api_name(&self) -> &'static str {
        match self {
            Self::TemperatureDrop(_) => "temperature_drop",
            Self::Blizzard => "blizzard",
            Self::WaterPipeFreeze => "water_pipe_freeze",
            Self::CharacterArrival(_) => "character_arrival",
            Self::CharacterDeparture(_) => "character_departure",
            Self::CharacterInjury(_) => "character_injury",
            Self::CharacterDiscovery(_, _) => "character_discovery",
            Self::Conflict(_, _) => "conflict",
            Self::Alliance(_, _) => "alliance",
            Self::Betrayal(_, _) => "betrayal",
            Self::ResourceFound(_, _) => "resource_found",
            Self::ResourceDepleted(_) => "resource_depleted",
            Self::PlotAdvance(_) => "plot_advance",
            Self::PromiseBroken(_, _, _) => "promise_broken",
            Self::Reminder(_, _) => "reminder",
            Self::ExternalStimulus(_) => "external_stimulus",
            Self::StressCascade(_, _) => "stress_cascade",
            Self::ResourceConflict(_, _, _) => "resource_conflict",
            Self::Custom(_) => "custom",
        }
    }
}

// ---------------------------------------------------------------------------
// Event priority & severity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    Major,
    Normal,
    Minor,
}

// ---------------------------------------------------------------------------
// Event source
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventSource {
    Scheduled,
    Conditional,
    Cascade,
    Manual,
    System,
}

// ---------------------------------------------------------------------------
// Scheduled event
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub id: Uuid,
    pub trigger_time: SimTime,
    pub event_type: EventType,
    pub title: String,
    pub description: String,
    pub participants: Vec<Uuid>,
    pub priority: EventPriority,
    pub severity: Severity,
    pub source: EventSource,
    pub one_shot: bool,
    pub fired: bool,
}

impl Eq for ScheduledEvent {}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .trigger_time
            .cmp(&self.trigger_time)
            .then_with(|| other.priority.cmp(&self.priority))
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ---------------------------------------------------------------------------
// Condition trigger
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionTrigger {
    pub id: String,
    pub condition: TriggerCondition,
    pub effect: TriggerEffect,
    pub one_shot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub and: Option<Vec<TriggerClause>>,
    pub or: Option<Vec<TriggerClause>>,
    pub variable: Option<String>,
    pub op: Option<String>,
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerClause {
    pub variable: String,
    pub op: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEffect {
    #[serde(rename = "type")]
    pub effect_type: String,
    pub description: String,
    pub affected_locations: Option<Vec<String>>,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Event queue (BinaryHeap)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EventQueue {
    pub scheduled: BinaryHeap<ScheduledEvent>,
    pub condition_triggers: Vec<ConditionTrigger>,
    pub historical: Vec<HistoricalEvent>,
    pub max_historical: usize,
}

impl Default for EventQueue {
    fn default() -> Self {
        Self {
            scheduled: BinaryHeap::new(),
            condition_triggers: vec![],
            historical: vec![],
            max_historical: 50_000,
        }
    }
}

impl EventQueue {
    pub fn push(&mut self, event: ScheduledEvent) {
        self.scheduled.push(event);
    }

    pub fn pop_due(&mut self, current_time: &SimTime) -> Option<ScheduledEvent> {
        if let Some(next) = self.scheduled.peek() {
            if next.trigger_time <= *current_time {
                return self.scheduled.pop();
            }
        }
        None
    }

    pub fn peek_due(&self, current_time: &SimTime) -> Option<&ScheduledEvent> {
        self.scheduled
            .peek()
            .filter(|e| e.trigger_time <= *current_time)
    }

    pub fn archive(&mut self, event: HistoricalEvent) {
        self.historical.push(event);
        if self.historical.len() > self.max_historical {
            self.historical.remove(0);
        }
    }
}

// ---------------------------------------------------------------------------
// Historical event record
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEvent {
    pub id: Uuid,
    pub tick: u64,
    pub sim_time: NaiveDateTime,
    pub event_type: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub participants: Vec<String>,
    pub location_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Event summary for WS broadcast
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub event_id: Uuid,
    pub event_type: String,
    pub title: String,
    pub severity: String,
}
