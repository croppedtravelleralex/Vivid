use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::CouncilInput;

/// Rolling window size for the quality score tracker.
const ROLLING_WINDOW: usize = 10;

/// Retrieve or initialise the global rolling quality history.
fn quality_history() -> &'static Mutex<VecDeque<f64>> {
    static HISTORY: OnceLock<Mutex<VecDeque<f64>>> = OnceLock::new();
    HISTORY.get_or_init(|| Mutex::new(VecDeque::with_capacity(ROLLING_WINDOW)))
}

/// Multi-dimensional quality scores for the current assessment window.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityScores {
    /// How much variety exists in the event stream (0.0–1.0).
    pub event_density: f64,
    /// How well the emotional/conflict range is spread (0.0–1.0).
    pub tension_variety: f64,
    /// How many distinct characters appear in recent events (0.0–1.0).
    pub character_involvement: f64,
}

/// A full assessment record with component scores and rolling average.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentRecord {
    pub tick: u64,
    pub scores: QualityScores,
    pub blended: f64,
    pub rolling_average: f64,
}

/// Run the quality assessment on the given council input.
///
/// Scores three dimensions on 0.0–1.0 and returns a blended score.
/// The result is also recorded in a global rolling window for trend tracking.
pub fn run_assessment(input: &CouncilInput) -> f64 {
    tracing::info!(
        tick = input.tick,
        "Quality Assessor: starting assessment"
    );

    let scores = compute_scores(input);
    let blended = blend_scores(&scores);

    // Record into rolling history
    {
        let mut history = quality_history().lock().expect("quality_history mutex");
        if history.len() >= ROLLING_WINDOW {
            history.pop_front();
        }
        history.push_back(blended);
    }

    let rolling_avg = rolling_average();
    let record = AssessmentRecord {
        tick: input.tick,
        scores,
        blended,
        rolling_average: rolling_avg,
    };

    tracing::info!(
        tick = input.tick,
        blended = format!("{:.4}", record.blended),
        rolling_average = format!("{:.4}", record.rolling_average),
        "Quality Assessor: assessment complete"
    );

    record.blended
}

/// Compute the three component scores from the input.
fn compute_scores(input: &CouncilInput) -> QualityScores {
    let event_density = compute_event_density(input);
    let tension_variety = compute_tension_variety(input);
    let character_involvement = compute_character_involvement(input);

    QualityScores {
        event_density,
        tension_variety,
        character_involvement,
    }
}

/// Event density: ratio of recent events to elapsed ticks, capped and
/// normalised to 0.0–1.0.
fn compute_event_density(input: &CouncilInput) -> f64 {
    if input.tick == 0 || input.recent_events.is_empty() {
        return 0.0;
    }
    let raw_density = input.recent_events.len() as f64 / input.tick as f64;
    // Scale so that 0.05 events/tick ≈ 0.5, 0.1 events/tick ≈ 1.0
    (raw_density * 10.0).clamp(0.0, 1.0)
}

/// Tension variety: estimate how many distinct emotional/conflict tones
/// appear in recent events, normalised.
fn compute_tension_variety(input: &CouncilInput) -> f64 {
    if input.recent_events.is_empty() {
        return 0.0;
    }

    let tones = [
        "hope",
        "fear",
        "tension",
        "relief",
        "grief",
        "joy",
        "anger",
        "melancholy",
    ];

    let unique_tones: usize = tones
        .iter()
        .filter(|tone| {
            input
                .recent_events
                .iter()
                .any(|e| e.to_lowercase().contains(*tone))
        })
        .count();

    // Normalise: 0 tones → 0.0, 4+ tones → 1.0
    (unique_tones as f64 / 4.0).clamp(0.0, 1.0)
}

/// Character involvement: estimate how many distinct characters participate
/// in recent events, normalised.
fn compute_character_involvement(input: &CouncilInput) -> f64 {
    if input.recent_events.is_empty() || input.character_count == 0 {
        return 0.0;
    }

    let mut unique_chars: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for event in &input.recent_events {
        // Simple heuristic: capitalised words that appear in multiple
        // events are likely character names.
        for word in event.split_whitespace() {
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric());
            if !cleaned.is_empty()
                && cleaned.starts_with(|c: char| c.is_uppercase())
                && cleaned.len() > 1
            {
                unique_chars.insert(cleaned.to_string());
            }
        }
    }

    // Cap at character_count; normalise so N/3 characters → 0.7
    let distinct = unique_chars.len().min(input.character_count);
    let ratio = distinct as f64 / input.character_count.max(1) as f64;
    (ratio * 1.5).clamp(0.0, 1.0)
}

/// Blend the three scores into a single 0.0–1.0 score.
///
/// Weights: density=0.35, variety=0.35, involvement=0.30
fn blend_scores(scores: &QualityScores) -> f64 {
    scores.event_density * 0.35
        + scores.tension_variety * 0.35
        + scores.character_involvement * 0.30
}

/// Return the rolling average of the last N scores.
pub fn rolling_average() -> f64 {
    let history = quality_history()
        .lock()
        .expect("quality_history mutex");

    if history.is_empty() {
        return 0.0;
    }
    history.iter().sum::<f64>() / history.len() as f64
}

/// Reset the rolling history (useful for testing).
pub fn reset_history() {
    let mut history = quality_history()
        .lock()
        .expect("quality_history mutex");
    history.clear();
}

/// Return the number of records in the rolling history.
pub fn history_len() -> usize {
    let history = quality_history()
        .lock()
        .expect("quality_history mutex");
    history.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::council::CouncilInput;

    fn sample_input() -> CouncilInput {
        CouncilInput {
            tick: 100,
            character_count: 5,
            recent_events: vec![
                "Zhang Yuan discovered a hidden cache — a surge of hope".into(),
                "Lin Shuang's anger boiled over during the argument".into(),
                "Old Sun felt a deep melancholy watching the sunset".into(),
                "The group faced the Blood Hand Gang with rising tension".into(),
            ],
            active_threads: vec!["Blood Hand Gang threat".into()],
        }
    }

    #[test]
    fn test_run_assessment_returns_score() {
        reset_history();
        let input = sample_input();
        let score = run_assessment(&input);
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_rolling_average_accumulates() {
        reset_history();
        let input = sample_input();

        for _ in 0..5 {
            run_assessment(&input);
        }

        let avg = rolling_average();
        assert!(avg > 0.0);
        assert!(history_len() <= ROLLING_WINDOW);
    }

    #[test]
    fn test_rolling_window_capped() {
        reset_history();
        let input = sample_input();

        for _ in 0..ROLLING_WINDOW + 5 {
            run_assessment(&input);
        }

        assert!(history_len() <= ROLLING_WINDOW, "history should be capped at {}", ROLLING_WINDOW);
    }

    #[test]
    fn test_empty_input_scores_zero() {
        reset_history();
        let input = CouncilInput {
            tick: 0,
            character_count: 0,
            recent_events: vec![],
            active_threads: vec![],
        };
        let score = run_assessment(&input);
        assert!(score < 0.1);
    }

    #[test]
    fn test_event_density_scaling() {
        let input = CouncilInput {
            tick: 200,
            character_count: 3,
            recent_events: vec!["event a".into(), "event b".into()],
            active_threads: vec![],
        };
        let density = compute_event_density(&input);
        // 2 events / 200 ticks = 0.01 → 0.01 * 10 = 0.1
        assert!((density - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_tension_variety_detects_multiple_tones() {
        let input = CouncilInput {
            tick: 50,
            character_count: 3,
            recent_events: vec![
                "hope and joy filled the camp".into(),
                "fear and anger spread quickly".into(),
                "a deep melancholy settled in".into(),
            ],
            active_threads: vec![],
        };
        let variety = compute_tension_variety(&input);
        assert!(variety > 0.5);
    }
}
