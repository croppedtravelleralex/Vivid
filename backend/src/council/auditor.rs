use serde::{Deserialize, Serialize};

use super::CouncilInput;

/// A single inconsistency finding produced by the continuity auditor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inconsistency {
    /// Human-readable description of the issue.
    pub description: String,
    /// Severity between 0.0 (trivial) and 1.0 (critical).
    pub severity: f64,
    /// Optional suggested fix.
    pub suggested_fix: Option<String>,
}

/// Aggregated audit report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditReport {
    pub character_inconsistencies: Vec<Inconsistency>,
    pub time_inconsistencies: Vec<Inconsistency>,
    pub relationship_inconsistencies: Vec<Inconsistency>,
    pub overall_coherence: f64,
}

/// Run the continuity audit on the given council input.
///
/// Checks three dimensions:
/// 1. **Character consistency** — are there sudden swings without cause?
/// 2. **Time consistency** — are events in a sensible order?
/// 3. **Relationship consistency** — does trust change without interaction?
pub fn run_audit(input: &CouncilInput) -> Vec<String> {
    tracing::info!(
        tick = input.tick,
        event_count = input.recent_events.len(),
        "Continuity Auditor: starting audit"
    );

    let report = analyze_characters(input);
    let time_issues = check_time_consistency(input);
    let relationship_issues = check_relationships(input);

    let total_issues = report.character_inconsistencies.len()
        + time_issues.len()
        + relationship_issues.len();

    let mut findings: Vec<String> = Vec::new();

    // Character findings
    for inc in &report.character_inconsistencies {
        let msg = format!(
            "[char] {} (severity: {:.2}){}",
            inc.description,
            inc.severity,
            inc.suggested_fix
                .as_ref()
                .map(|f| format!(" — fix: {}", f))
                .unwrap_or_default()
        );
        findings.push(msg);
    }

    // Time findings
    for inc in &time_issues {
        let msg = format!(
            "[time] {} (severity: {:.2})",
            inc.description, inc.severity
        );
        findings.push(msg);
    }

    // Relationship findings
    for inc in &relationship_issues {
        let msg = format!(
            "[relation] {} (severity: {:.2}){}",
            inc.description,
            inc.severity,
            inc.suggested_fix
                .as_ref()
                .map(|f| format!(" — fix: {}", f))
                .unwrap_or_default()
        );
        findings.push(msg);
    }

    tracing::info!(
        tick = input.tick,
        total_issues = total_issues,
        coherence = format!("{:.2}", report.overall_coherence),
        "Continuity Auditor: audit complete"
    );

    findings
}

/// Analyze character-state consistency from the recent event stream.
fn analyze_characters(input: &CouncilInput) -> AuditReport {
    let mut report = AuditReport::default();

    // Simple heuristic: look for rapid emotional contrasts in event descriptions.
    let positive_keywords = ["joy", "hope", "relief", "happy", "grateful", "excited"];
    let negative_keywords = [
        "anger", "fear", "despair", "grief", "rage", "panic", "horror", "sad",
    ];

    let mut mood_swings: Vec<(usize, String, String)> = Vec::new();

    for (i, event) in input.recent_events.iter().enumerate() {
        let lower = event.to_lowercase();
        let has_positive = positive_keywords.iter().any(|k| lower.contains(k));
        let has_negative = negative_keywords.iter().any(|k| lower.contains(k));

        if has_positive && has_negative {
            mood_swings.push((i, "mixed emotional signals".into(), event.clone()));
        }
    }

    // Check for repetitive character tags suggesting stagnation
    for (_, label, event) in &mood_swings {
        report.character_inconsistencies.push(Inconsistency {
            description: format!("{}: {}", label, event),
            severity: 0.5,
            suggested_fix: Some("Provide a narrative bridge for the emotional shift".into()),
        });
    }
    let mut char_mentions: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for event in &input.recent_events {
        let lower = event.to_lowercase();
        for word in lower.split_whitespace() {
            if word.len() > 2 {
                *char_mentions.entry(word.to_string()).or_insert(0) += 1;
            }
        }
    }

    // Flag characters mentioned many times without change indicators
    for (word, count) in char_mentions.iter() {
        let change_keywords = ["but", "however", "changed", "transformed", "realized"];
        let has_change = change_keywords.iter().any(|k| input.recent_events.iter().any(|e| e.to_lowercase().contains(k)));
        if *count > 3 && !has_change {
            report.character_inconsistencies.push(Inconsistency {
                description: format!(
                    "Character '{}' appears {} times without transformation indicators",
                    word, count
                ),
                severity: 0.25,
                suggested_fix: Some("Introduce a turning point event for this character".into()),
            });
        }
    }

    // Calculate overall coherence (inverse of issues)
    let total_flags = mood_swings.len() + report.character_inconsistencies.len();
    report.overall_coherence = (1.0 - (total_flags as f64 * 0.1).clamp(0.0, 0.95)).max(0.3);
    report
}

/// Check that events have a sensible temporal flow.
fn check_time_consistency(input: &CouncilInput) -> Vec<Inconsistency> {
    let mut issues = Vec::new();

    // Look for "later" / "next day" etc. appearing without anchor events
    let time_anchors = ["earlier", "later", "next day", "the next morning", "hours later"];
    let has_time_jumps = input.recent_events.iter().any(|e| {
        let lower = e.to_lowercase();
        time_anchors.iter().any(|a| lower.contains(a))
    });

    if has_time_jumps && input.recent_events.len() < 3 {
        issues.push(Inconsistency {
            description: "Time-jump language detected but too few events to establish chronology".into(),
            severity: 0.3,
            suggested_fix: Some("Add bridging events to anchor the time jump".into()),
        });
    }

    issues
}

/// Check that relationship changes are grounded in events.
fn check_relationships(_input: &CouncilInput) -> Vec<Inconsistency> {
    // With the current simplified input we primarily flag general risks.
    // Full relationship-graph analysis will require richer input (doc 13 M3).
    Vec::new()
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
                "Zhang Yuan felt a surge of hope as he found supplies, but fear crept in.".into(),
                "Lin Shuang's anger boiled over during the argument.".into(),
            ],
            active_threads: vec!["Blood Hand Gang threat".into()],
        }
    }

    #[test]
    fn test_run_audit_returns_findings() {
        let input = sample_input();
        let findings = run_audit(&input);
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_mixed_emotional_signals_detected() {
        let input = sample_input();
        let findings = run_audit(&input);
        let has_mood_issue = findings.iter().any(|f| f.contains("mixed emotional"));
        assert!(has_mood_issue, "Should flag mixed emotional signals");
    }

    #[test]
    fn test_audit_empty_input() {
        let input = CouncilInput {
            tick: 0,
            character_count: 0,
            recent_events: vec![],
            active_threads: vec![],
        };
        let findings = run_audit(&input);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_time_inconsistency_detected() {
        let input = CouncilInput {
            tick: 50,
            character_count: 3,
            recent_events: vec!["Hours later, they regrouped.".into()],
            active_threads: vec![],
        };
        let findings = run_audit(&input);
        let has_time_issue = findings.iter().any(|f| f.contains("[time]"));
        assert!(has_time_issue);
    }
}
