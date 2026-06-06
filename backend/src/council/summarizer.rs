use serde::{Deserialize, Serialize};

use super::NarrativeDirective;

/// A structured summary record produced by the executive summarizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilSummary {
    pub paragraph: String,
    pub finding_count: usize,
    pub dominant_theme: String,
    pub urgency_level: UrgencyLevel,
}

/// How urgently the council's findings should be actioned.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UrgencyLevel {
    /// Everything is on track — minor tweaks only.
    Low,
    /// Some issues need attention in the coming window.
    Medium,
    /// Actionable issues require prompt follow-through.
    High,
    /// Critical misalignment — needs immediate intervention.
    Critical,
}

/// Produce a one-paragraph executive summary from the council's findings.
///
/// The summarizer examines the directive produced by the first four members
/// (auditor, architect, assessor, analyst) and distills them into a single
/// coherent paragraph with an urgency rating.
pub fn run_summary(directive: &NarrativeDirective) -> String {
    tracing::info!(
        round = directive.round,
        tick = directive.tick,
        "Executive Summarizer: composing narrative directive summary"
    );

    let paragraph = compose_paragraph(directive);
    let _summary = CouncilSummary {
        paragraph: paragraph.clone(),
        finding_count: total_findings(directive),
        dominant_theme: identify_dominant_theme(directive),
        urgency_level: classify_urgency(directive),
    };

    tracing::info!(
        round = directive.round,
        tick = directive.tick,
        finding_count = _summary.finding_count,
        urgency = format!("{:?}", _summary.urgency_level),
        "Executive Summarizer: summary complete"
    );

    paragraph
}

/// Compose a readable one-paragraph summary from the combined directive.
fn compose_paragraph(directive: &NarrativeDirective) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Opening — round and tick context
    parts.push(format!(
        "Council Round {} at tick {}.",
        directive.round, directive.tick
    ));

    // Auditor
    if !directive.auditor_findings.is_empty() {
        parts.push(format!(
            "The continuity auditor identified {} issue{}: {}.",
            directive.auditor_findings.len(),
            if directive.auditor_findings.len() == 1 {
                ""
            } else {
                "s"
            },
            directive.auditor_findings.first().cloned().unwrap_or_default()
        ));
        if directive.auditor_findings.len() > 1 {
            parts.push(format!(
                "Additionally, {} other matter{} flagged.",
                directive.auditor_findings.len() - 1,
                if directive.auditor_findings.len() == 2 {
                    " was"
                } else {
                    "s were"
                }
            ));
        }
    } else {
        parts.push("No continuity issues detected.".into());
    }

    // Architect
    if !directive.architect_arc_adjustments.is_empty() {
        parts.push(format!(
            "The arc architect recommends {} arc adjustment{} to maintain narrative momentum.",
            directive.architect_arc_adjustments.len(),
            if directive.architect_arc_adjustments.len() == 1 {
                ""
            } else {
                "s"
            }
        ));
    }

    // Assessor
    let score_desc = match directive.assessor_score {
        s if s >= 0.8 => "excellent",
        s if s >= 0.6 => "good",
        s if s >= 0.4 => "adequate",
        s if s >= 0.2 => "concerning",
        _ => "poor",
    };
    parts.push(format!(
        "Narrative quality is assessed at {:.2} ({}) on a 0–1 scale.",
        directive.assessor_score, score_desc
    ));

    // Analyst
    if !directive.analyst_foreshadow_notes.is_empty() {
        let dormant_count = directive
            .analyst_foreshadow_notes
            .iter()
            .filter(|n| n.contains("[resurrect]"))
            .count();
        let chekhov_count = directive
            .analyst_foreshadow_notes
            .iter()
            .filter(|n| n.contains("[chekhov]"))
            .count();

        if dormant_count > 0 || chekhov_count > 0 {
            let mut threads = Vec::new();
            if dormant_count > 0 {
                threads.push(format!("{} dormant thread{} ready for revival", dormant_count, if dormant_count == 1 { "" } else { "s" }));
            }
            if chekhov_count > 0 {
                threads.push(format!("{} unresolved Chekhov's gun{}", chekhov_count, if chekhov_count == 1 { "" } else { "s" }));
            }
            parts.push(format!(
                "Foreshadow analysis found {}.",
                threads.join(" and ")
            ));
        }
    }

    // Combine into one paragraph
    parts.join(" ")
}

/// Count total findings across all council members.
fn total_findings(directive: &NarrativeDirective) -> usize {
    directive.auditor_findings.len()
        + directive.architect_arc_adjustments.len()
        + directive.analyst_foreshadow_notes.len()
}

/// Identify the dominant theme from the directive outputs.
fn identify_dominant_theme(directive: &NarrativeDirective) -> String {
    let issues = directive.auditor_findings.len();
    let adjustments = directive.architect_arc_adjustments.len();
    let notes = directive.analyst_foreshadow_notes.len();
    let score = directive.assessor_score;

    if issues > adjustments && issues > notes {
        "continuity".into()
    } else if adjustments > issues && adjustments > notes {
        "arc_planning".into()
    } else if notes > issues && notes > adjustments {
        "foreshadow_tracking".into()
    } else if score < 0.4 {
        "quality_concern".into()
    } else {
        "balanced".into()
    }
}

/// Classify urgency based on findings and quality score.
fn classify_urgency(directive: &NarrativeDirective) -> UrgencyLevel {
    let auditor_high_severity = directive
        .auditor_findings
        .iter()
        .filter(|f| {
            // Heuristic: findings mentioning "critical" or "severe" are high
            let lower = f.to_lowercase();
            lower.contains("critical") || lower.contains("severe") || lower.contains("fix")
        })
        .count();

    if auditor_high_severity >= 2 || directive.assessor_score < 0.2 {
        UrgencyLevel::Critical
    } else if auditor_high_severity >= 1 || directive.assessor_score < 0.4 {
        UrgencyLevel::High
    } else if directive.assessor_score < 0.6 || !directive.analyst_foreshadow_notes.is_empty() {
        UrgencyLevel::Medium
    } else {
        UrgencyLevel::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::council::NarrativeDirective;

    fn sample_directive() -> NarrativeDirective {
        NarrativeDirective {
            round: 3,
            tick: 150,
            auditor_findings: vec![
                "[char] Zhang Yuan mood swing without cause (severity: 0.40)".into(),
            ],
            architect_arc_adjustments: vec![
                "[tension] Suggested target tension: 0.65".into(),
            ],
            assessor_score: 0.62,
            analyst_foreshadow_notes: vec![
                "[resurrect] Dormant thread: Blood Hand Gang threat — revival value: 0.70".into(),
            ],
            summary: String::new(),
        }
    }

    #[test]
    fn test_run_summary_returns_string() {
        let directive = sample_directive();
        let summary = run_summary(&directive);
        assert!(!summary.is_empty());
        assert!(summary.contains("Council Round 3"));
        assert!(summary.contains("tick 150"));
    }

    #[test]
    fn test_summary_contains_quality_assessment() {
        let directive = sample_directive();
        let summary = run_summary(&directive);
        assert!(summary.contains("0.62"));
    }

    #[test]
    fn test_summary_contains_foreshadow_info() {
        let directive = sample_directive();
        let summary = run_summary(&directive);
        assert!(summary.contains("dormant thread") || summary.contains("Chekhov"));
    }

    #[test]
    fn test_summary_empty_directive() {
        let directive = NarrativeDirective::default();
        let summary = run_summary(&directive);
        assert!(!summary.is_empty());
        assert!(summary.contains("No continuity issues"));
    }

    #[test]
    fn test_urgency_classification() {
        let mut directive = sample_directive();

        directive.assessor_score = 0.15;
        assert_eq!(classify_urgency(&directive), UrgencyLevel::Critical);

        directive.assessor_score = 0.35;
        directive.auditor_findings.clear();
        assert_eq!(classify_urgency(&directive), UrgencyLevel::High);

        directive.assessor_score = 0.55;
        assert_eq!(classify_urgency(&directive), UrgencyLevel::Medium);

        directive.assessor_score = 0.85;
        directive.analyst_foreshadow_notes.clear();
        assert_eq!(classify_urgency(&directive), UrgencyLevel::Low);
    }

    #[test]
    fn test_dominant_theme_identification() {
        let directive = sample_directive();
        let theme = identify_dominant_theme(&directive);
        assert!(!theme.is_empty());
    }

    #[test]
    fn test_total_findings_count() {
        let directive = sample_directive();
        assert_eq!(total_findings(&directive), 3);
    }
}
