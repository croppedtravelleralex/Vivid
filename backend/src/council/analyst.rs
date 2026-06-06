use serde::{Deserialize, Serialize};

use super::CouncilInput;

/// A single foreshadow note about a dormant or unresolved thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeshadowNote {
    /// Human-readable description of the dormant thread.
    pub description: String,
    /// How valuable revival would be (0.0–1.0).
    pub revival_value: f64,
    /// Suggested trigger condition for resurrection.
    pub suggested_trigger: Option<String>,
    /// Whether this is a completely unresolved Chekhov's gun.
    pub is_chekhov_gun: bool,
}

/// Aggregated analyst output.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalystReport {
    pub resurrection_candidates: Vec<ForeshadowNote>,
    pub chekhov_guns: Vec<ForeshadowNote>,
}

/// Keywords that signal a dormant / unresolved plot thread.
const DORMANT_KEYWORDS: &[&str] = &[
    "dormant",
    "stale",
    "archived",
    "unresolved",
    "unfinished",
    "pending",
    "forgotten",
    "abandoned",
];

/// Keywords that suggest a Chekhov's gun (mentioned but never resolved).
const CHEKHOV_KEYWORDS: &[&str] = &[
    "gun",
    "secret",
    "promise",
    "mystery",
    "unknown",
    "unanswered",
    "never explained",
    "hint",
    "foreshadow",
];

/// Run the foreshadow analysis on the given council input.
///
/// Scans active threads for dormant / archived entries and flags
/// Chekhov's guns — plot elements that were seeded but never resolved.
pub fn run_analysis(input: &CouncilInput) -> Vec<String> {
    tracing::info!(
        tick = input.tick,
        thread_count = input.active_threads.len(),
        "Foreshadow Analyst: starting foreshadow analysis"
    );

    let report = analyze_threads(input);

    let mut notes: Vec<String> = Vec::new();

    // Resurrection candidates
    for candidate in &report.resurrection_candidates {
        let trigger = candidate
            .suggested_trigger
            .as_ref()
            .map(|t| format!(" (trigger: {})", t))
            .unwrap_or_default();
        notes.push(format!(
            "[resurrect] {} — revival value: {:.2}{}",
            candidate.description, candidate.revival_value, trigger
        ));
    }

    // Chekhov's guns
    for gun in &report.chekhov_guns {
        notes.push(format!(
            "[chekhov] {} — revival value: {:.2}",
            gun.description, gun.revival_value
        ));
    }

    // Also scan recent events for planted elements that never had a payoff
    let planted_seeds = scan_recent_events_for_seeds(input);
    for seed in &planted_seeds {
        notes.push(format!("[seed] {}", seed));
    }

    tracing::info!(
        tick = input.tick,
        resurrection_candidates = report.resurrection_candidates.len(),
        chekhov_guns = report.chekhov_guns.len(),
        seeds_found = planted_seeds.len(),
        "Foreshadow Analyst: analysis complete"
    );

    notes
}

/// Analyse active threads for dormant entries and Chekhov's guns.
fn analyze_threads(input: &CouncilInput) -> AnalystReport {
    let mut report = AnalystReport::default();

    for thread in &input.active_threads {
        let lower = thread.to_lowercase();

        // Check for dormant keywords
        let is_dormant = DORMANT_KEYWORDS.iter().any(|k| lower.contains(k));
        // Check for Chekhov keywords
        let is_chekhov = CHEKHOV_KEYWORDS.iter().any(|k| lower.contains(k));

        if is_dormant {
            let suggested_trigger = generate_trigger_suggestion(thread);
            let note = ForeshadowNote {
                description: format!("Dormant thread: {}", thread),
                revival_value: compute_revival_value(thread),
                suggested_trigger,
                is_chekhov_gun: false,
            };
            report.resurrection_candidates.push(note);
        }

        if is_chekhov {
            let note = ForeshadowNote {
                description: format!("Chekhov's gun: {}", thread),
                revival_value: compute_revival_value(thread),
                suggested_trigger: None,
                is_chekhov_gun: true,
            };
            report.chekhov_guns.push(note);
        }
    }

    report
}

/// Scan recent events for elements that were planted (introduced with
/// foreshadowing language) but never surfaced in later events.
fn scan_recent_events_for_seeds(input: &CouncilInput) -> Vec<String> {
    let mut seeds = Vec::new();

    let foreshadow_phrases = [
        "would later",
        "little did",
        "as if",
        "somehow",
        "had a feeling",
        "strange",
        "odd",
        "unusual",
    ];

    for event in &input.recent_events {
        let lower = event.to_lowercase();
        let has_foreshadow = foreshadow_phrases.iter().any(|p| lower.contains(p));

        if has_foreshadow {
            seeds.push(format!(
                "Potential seed planted: \"{}\" — consider following up",
                truncate(event, 80)
            ));
        }
    }

    seeds
}

/// Generate a context-aware trigger suggestion for reviving a dormant thread.
fn generate_trigger_suggestion(thread: &str) -> Option<String> {
    let lower = thread.to_lowercase();

    if lower.contains("threat") || lower.contains("gang") || lower.contains("enemy") {
        Some("When group resources fall below 20% or a territory boundary is crossed".into())
    } else if lower.contains("secret") || lower.contains("past") || lower.contains("mystery") {
        Some("When trust between involved characters reaches a threshold (e.g. 7/10)".into())
    } else if lower.contains("promise") || lower.contains("ally") || lower.contains("friend") {
        Some("When the group enters a related location or faces a related crisis".into())
    } else {
        Some("When a character is isolated or under stress".into())
    }
}

/// Compute a revival-value score (0.0–1.0) based on thread characteristics.
fn compute_revival_value(thread: &str) -> f64 {
    let lower = thread.to_lowercase();

    let mut value: f64 = 0.5;

    // Increase for threads tied to high-impact categories
    if lower.contains("threat") || lower.contains("danger") {
        value += 0.2;
    }
    if lower.contains("secret") || lower.contains("mystery") {
        value += 0.15;
    }
    if lower.contains("character") || lower.contains("relationship") {
        value += 0.1;
    }

    // Decrease for minor or resolved-sounding threads
    if lower.contains("minor") || lower.contains("trivial") {
        value -= 0.2;
    }

    value.clamp(0.0, 1.0)
}

/// Truncate a string to at most `max_len` characters, appending "..." if
/// truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::council::CouncilInput;

    fn sample_input() -> CouncilInput {
        CouncilInput {
            tick: 150,
            character_count: 5,
            recent_events: vec![
                "Zhang Yuan found a strange map with odd markings".into(),
            ],
            active_threads: vec![
                "Blood Hand Gang threat (dormant)".into(),
                "Lin Shuang's mysterious past (unresolved)".into(),
                "Old Sun's promise to his daughter".into(),
            ],
        }
    }

    #[test]
    fn test_run_analysis_returns_notes() {
        let input = sample_input();
        let notes = run_analysis(&input);
        assert!(!notes.is_empty());
    }

    #[test]
    fn test_dormant_thread_detected() {
        let input = sample_input();
        let notes = run_analysis(&input);
        let has_dormant = notes.iter().any(|n| n.contains("[resurrect]"));
        assert!(has_dormant);
    }

    #[test]
    fn test_chekhov_gun_detected() {
        let input = CouncilInput {
            tick: 100,
            character_count: 3,
            recent_events: vec![],
            active_threads: vec!["The secret of the basement (mystery)".into()],
        };
        let notes = run_analysis(&input);
        let has_chekhov = notes.iter().any(|n| n.contains("[chekhov]"));
        assert!(has_chekhov);
    }

    #[test]
    fn test_seeds_detected_in_recent_events() {
        let input = CouncilInput {
            tick: 80,
            character_count: 4,
            recent_events: vec![
                "Little did they know, the basement held a dark secret.".into(),
            ],
            active_threads: vec![],
        };
        let notes = run_analysis(&input);
        let has_seed = notes.iter().any(|n| n.contains("[seed]"));
        assert!(has_seed);
    }

    #[test]
    fn test_analysis_empty_input() {
        let input = CouncilInput {
            tick: 0,
            character_count: 0,
            recent_events: vec![],
            active_threads: vec![],
        };
        let notes = run_analysis(&input);
        assert!(notes.is_empty());
    }

    #[test]
    fn test_revival_value_scoring() {
        assert!(
            compute_revival_value("major threat") > compute_revival_value("minor detail")
        );
        assert!((0.0..=1.0).contains(&compute_revival_value("anything")));
    }

    #[test]
    fn test_truncate() {
        let short = "hello";
        assert_eq!(truncate(short, 10), "hello");

        let long = "a".repeat(100);
        let truncated = truncate(&long, 20);
        assert!(truncated.len() <= 20);
        assert!(truncated.ends_with("..."));
    }
}
