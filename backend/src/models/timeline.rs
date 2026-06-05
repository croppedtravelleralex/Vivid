use chrono::{Datelike, NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};

use super::event::HistoricalEvent;

// ---------------------------------------------------------------------------
// SimTime
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SimTime {
    pub datetime: NaiveDateTime,
}

impl SimTime {
    pub fn new(datetime: NaiveDateTime) -> Self {
        Self { datetime }
    }

    pub fn advance_minutes(&mut self, minutes: u64) {
        self.datetime = self.datetime + chrono::Duration::minutes(minutes as i64);
    }

    pub fn advance_hours(&mut self, hours: u64) {
        self.datetime = self.datetime + chrono::Duration::hours(hours as i64);
    }

    pub fn hour(&self) -> u32 {
        self.datetime.hour()
    }

    pub fn month(&self) -> u32 {
        self.datetime.month()
    }

    pub fn day(&self) -> u32 {
        self.datetime.day()
    }

    pub fn season(&self) -> &'static str {
        match self.datetime.month() {
            3 | 4 | 5 => "spring",
            6 | 7 | 8 => "summer",
            9 | 10 | 11 => "autumn",
            _ => "winter",
        }
    }
}

impl std::fmt::Display for SimTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.datetime.format("%Y-%m-%d %H:%M"))
    }
}

// ---------------------------------------------------------------------------
// SimSpeed
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimSpeed {
    Paused,
    Detailed,
    FastForward,
}

impl SimSpeed {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Paused => "paused",
            Self::Detailed => "detailed",
            Self::FastForward => "fast_forward",
        }
    }
}

// ---------------------------------------------------------------------------
// TimelineManager
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineManager {
    pub tick: u64,
    pub time: SimTime,
    pub events: Vec<HistoricalEvent>,
}

impl TimelineManager {
    pub fn new(start_time: NaiveDateTime) -> Self {
        Self {
            tick: 0,
            time: SimTime::new(start_time),
            events: vec![],
        }
    }

    pub fn advance_minutes(&mut self, minutes: u64) {
        self.tick += 1;
        self.time.advance_minutes(minutes);
    }

    pub fn advance_hours(&mut self, hours: u64) {
        self.tick += 1;
        self.time.advance_hours(hours);
    }

    pub fn record_event(&mut self, event: HistoricalEvent) {
        self.events.push(event);
    }
}

// ---------------------------------------------------------------------------
// API response types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineResponse {
    pub events: Vec<HistoricalEvent>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickResult {
    pub tick: u64,
    pub time: SimTime,
    pub events_triggered: u64,
    pub llm_calls: u64,
    pub speed: SimSpeed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTransition {
    pub from: SimSpeed,
    pub to: SimSpeed,
    pub reason: String,
}
