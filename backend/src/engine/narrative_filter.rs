use std::collections::VecDeque;
use tracing::info;

/// Filters generated events to keep only narratively interesting ones.
///
/// Each event is scored by novelty, impact, surprise, and diversity.
/// Per tick, only the top-K events are emitted to the frontend.
pub struct NarrativeFilter {
    pub min_impact_threshold: f64,
    pub max_events_per_tick: usize,
    pub novelty_window: usize,
    pub recent_event_types: VecDeque<String>,
    pub boredom: BoredomFilter,
}

/// An event paired with its narrative score.
#[derive(Debug, Clone)]
pub struct ScoredEvent {
    pub event_type: String,
    pub participants: Vec<String>,
    pub magnitude: f64,
    pub is_novel: bool,
    pub score: f64,
}

impl NarrativeFilter {
    /// Create a new narrative filter.
    pub fn new(
        min_impact_threshold: f64,
        max_events_per_tick: usize,
        novelty_window: usize,
    ) -> Self {
        Self {
            min_impact_threshold,
            max_events_per_tick,
            novelty_window,
            recent_event_types: VecDeque::with_capacity(novelty_window + 10),
            boredom: BoredomFilter::new(),
        }
    }

    /// Score a single event by narrative value (0.0 = boring, 1.0 = must-include).
    ///
    /// Factors:
    /// - novelty: how different from recent event types (weight 2.0)
    /// - impact: magnitude-based significance (weight 1.0)
    /// - surprise: inverse of probability (weight 1.5)
    pub fn score_event(
        &self,
        event_type: &str,
        _participants: &[String],
        magnitude: f64,
        is_novel: bool,
    ) -> f64 {
        let novelty_score = self.calculate_novelty(event_type) * 2.0;
        let impact_score = self.calculate_impact(magnitude) * 1.0;
        let surprise_score = self.calculate_surprise(0.3) * 1.5;

        let mut total = novelty_score + impact_score + surprise_score;

        // Bonus for novel events
        if is_novel {
            total += 0.5;
        }

        // Bonus for significant impact
        if magnitude > 0.5 {
            total += 0.3;
        }

        total
    }

    /// Filter a batch of scored events, returning only the top-scoring ones
    /// that pass the boredom filter and impact threshold.
    pub fn filter(&mut self, events: Vec<ScoredEvent>) -> Vec<ScoredEvent> {
        let mut passed = Vec::new();

        for event in events {
            // Skip events below the impact threshold
            if event.magnitude < self.min_impact_threshold {
                continue;
            }

            // Skip events that the boredom filter suppresses
            if self.boredom.should_suppress(&event.event_type) {
                continue;
            }

            passed.push(event);
        }

        // Track recent types
        for event in &passed {
            self.recent_event_types.push_back(event.event_type.clone());
            if self.recent_event_types.len() > self.novelty_window {
                self.recent_event_types.pop_front();
            }
        }

        // Score and sort
        for event in &mut passed {
            event.score = self.score_event(
                &event.event_type,
                &event.participants,
                event.magnitude,
                event.is_novel,
            );
        }

        passed.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Trim to max per tick
        let filtered: Vec<ScoredEvent> = passed.into_iter().take(self.max_events_per_tick).collect();

        if !filtered.is_empty() {
            info!(
                "narrative_filter: {} events passed filter (top score={:.3})",
                filtered.len(),
                filtered.first().map(|e| e.score).unwrap_or(0.0)
            );
        }

        filtered
    }

    /// Calculate novelty score based on recent event type frequency.
    fn calculate_novelty(&self, event_type: &str) -> f64 {
        let recent_count = self
            .recent_event_types
            .iter()
            .filter(|t| *t == event_type)
            .count();

        if recent_count == 0 {
            return 1.0; // completely novel
        }

        let window = self.novelty_window.max(1);
        let frequency = recent_count as f64 / window as f64;
        (1.0 - frequency).max(0.0)
    }

    /// Calculate impact score from event magnitude.
    fn calculate_impact(&self, magnitude: f64) -> f64 {
        magnitude.clamp(0.0, 1.0)
    }

    /// Calculate surprise score from base probability (low prob = high surprise).
    fn calculate_surprise(&self, probability: f64) -> f64 {
        (1.0 - probability.clamp(0.0, 1.0)).max(0.0)
    }
}

impl Default for NarrativeFilter {
    fn default() -> Self {
        Self::new(0.1, 5, 20)
    }
}

// ---------------------------------------------------------------------------
// BoredomFilter
// ---------------------------------------------------------------------------

/// Suppresses repetitive events by tracking the last N event types.
///
/// If the same event type appears too frequently in the recent window,
/// it is suppressed to avoid narrative fatigue.
pub struct BoredomFilter {
    pub recent_types: VecDeque<String>,
    pub max_history: usize,
    pub suppression_threshold: usize,
}

impl BoredomFilter {
    pub fn new() -> Self {
        Self {
            recent_types: VecDeque::with_capacity(20),
            max_history: 20,
            suppression_threshold: 3,
        }
    }

    /// Check if an event type should be suppressed.
    ///
    /// Suppresses when the same type appears >= `suppression_threshold`
    /// times in the last `max_history` events.
    pub fn should_suppress(&mut self, event_type: &str) -> bool {
        self.recent_types.push_back(event_type.to_string());
        if self.recent_types.len() > self.max_history {
            self.recent_types.pop_front();
        }

        let recent_5: Vec<&str> = self
            .recent_types
            .iter()
            .rev()
            .take(5)
            .map(|s| s.as_str())
            .collect();

        let same_count = recent_5
            .iter()
            .filter(|t| *t == &event_type)
            .count();

        let suppressed = same_count >= self.suppression_threshold;
        if suppressed {
            info!(
                "narrative_filter: suppressing '{}' ({} of last 5)",
                event_type, same_count
            );
        }

        suppressed
    }

    /// Reset the filter (e.g. at phase boundary).
    pub fn reset(&mut self) {
        self.recent_types.clear();
    }
}

impl Default for BoredomFilter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// TOP-K selection (free function)
// ---------------------------------------------------------------------------

/// Generic narrative value scoring for an event, used by `select_top_k`.
pub fn narrative_value(
    novelty: f64,
    unexpectedness: f64,
    impact_scope: f64,
    cascade_potential: f64,
    character_importance: f64,
    emotional_impact: f64,
) -> f64 {
    novelty * 2.0
        + unexpectedness * 1.5
        + impact_scope * 1.0
        + cascade_potential * 0.8
        + character_importance * 1.2
        + emotional_impact * 0.5
}

/// Select the top-K events from a list by narrative value score.
///
/// Events must implement the `HasNarrativeValue` trait.
pub fn select_top_k<T: HasNarrativeValue>(
    all_events: Vec<T>,
    k: usize,
) -> Vec<T> {
    let mut scored: Vec<(f64, T)> = all_events
        .into_iter()
        .map(|e| {
            let score = e.narrative_score();
            (score, e)
        })
        .collect();

    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_score = scored.first().map(|(s, _)| *s);

    let result: Vec<T> = scored.into_iter().take(k).map(|(_, e)| e).collect();

    if let Some(ts) = top_score {
        info!(
            "narrative_filter: top-K selected {} events (top score={:.3})",
            result.len(),
            ts
        );
    }

    result
}

/// Trait for types that can be scored by the narrative value system.
pub trait HasNarrativeValue {
    fn narrative_score(&self) -> f64;
}

/// Simple event wrapper that implements `HasNarrativeValue`.
#[derive(Debug, Clone)]
pub struct NarrativeEvent {
    pub event_type: String,
    pub novelty: f64,
    pub unexpectedness: f64,
    pub impact_scope: f64,
    pub cascade_potential: f64,
    pub character_importance: f64,
    pub emotional_impact: f64,
}

impl HasNarrativeValue for NarrativeEvent {
    fn narrative_score(&self) -> f64 {
        narrative_value(
            self.novelty,
            self.unexpectedness,
            self.impact_scope,
            self.cascade_potential,
            self.character_importance,
            self.emotional_impact,
        )
    }
}
