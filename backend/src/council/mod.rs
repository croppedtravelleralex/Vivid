pub mod analyst;
pub mod architect;
pub mod assessor;
pub mod auditor;
pub mod context_builder;
pub mod summarizer;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

static COUNCIL_ROUND: AtomicU64 = AtomicU64::new(0);

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Input data fed to every council session.
///
/// Kept deliberately simple so the council module does not need to import
/// the full `WorldState` or `TagIndex` types.  The engine layer extracts
/// the relevant fields and calls `context_builder::build_context()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilInput {
    pub tick: u64,
    pub character_count: usize,
    pub recent_events: Vec<String>,
    pub active_threads: Vec<String>,
}

/// The aggregated output produced by a full council convening.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeDirective {
    pub round: u64,
    pub tick: u64,
    pub auditor_findings: Vec<String>,
    pub architect_arc_adjustments: Vec<String>,
    pub assessor_score: f64,
    pub analyst_foreshadow_notes: Vec<String>,
    pub summary: String,
}

impl Default for NarrativeDirective {
    fn default() -> Self {
        Self {
            round: 0,
            tick: 0,
            auditor_findings: Vec::new(),
            architect_arc_adjustments: Vec::new(),
            assessor_score: 0.0,
            analyst_foreshadow_notes: Vec::new(),
            summary: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Council convening
// ---------------------------------------------------------------------------

/// Convene all 5 council members and aggregate their outputs into a single
/// `NarrativeDirective`.
///
/// # Execution model
///
/// The first four members (auditor, architect, assessor, analyst) run as
/// independent `tokio::spawn` tasks and are joined with `tokio::join!`.
/// The summarizer runs last because it needs the outputs of the other four.
///
/// # Panics
///
/// Panics if any of the spawned tasks panic.  In production this should be
/// wrapped with error handling at the engine level.
pub async fn convene(inputs: CouncilInput) -> NarrativeDirective {
    let round = COUNCIL_ROUND.fetch_add(1, Ordering::Relaxed) + 1;

    tracing::info!(
        round = round,
        tick = inputs.tick,
        event_count = inputs.recent_events.len(),
        thread_count = inputs.active_threads.len(),
        "Council convening — spawning 4 analysis tasks"
    );

    let input = Arc::new(inputs);

    // Spawn the first four members in parallel.
    let (auditor_res, architect_res, assessor_res, analyst_res) = tokio::join!(
        tokio::spawn({
            let input = Arc::clone(&input);
            async move { auditor::run_audit(&input) }
        }),
        tokio::spawn({
            let input = Arc::clone(&input);
            async move { architect::run_architecture(&input) }
        }),
        tokio::spawn({
            let input = Arc::clone(&input);
            async move { assessor::run_assessment(&input) }
        }),
        tokio::spawn({
            let input = Arc::clone(&input);
            async move { analyst::run_analysis(&input) }
        }),
    );

    // Unwrap — a panicked task is a fatal error in the current design.
    let auditor_findings = auditor_res.expect("auditor task panicked");
    let architect_arc_adjustments = architect_res.expect("architect task panicked");
    let assessor_score = assessor_res.expect("assessor task panicked");
    let analyst_foreshadow_notes = analyst_res.expect("analyst task panicked");

    tracing::info!(
        round = round,
        auditor_count = auditor_findings.len(),
        architect_count = architect_arc_adjustments.len(),
        assessor_score = format!("{:.4}", assessor_score),
        analyst_count = analyst_foreshadow_notes.len(),
        "All 4 analysis tasks completed — composing directive"
    );

    // Build a temporary directive for the summarizer.
    let mut directive = NarrativeDirective {
        round,
        tick: input.tick,
        auditor_findings,
        architect_arc_adjustments,
        assessor_score,
        analyst_foreshadow_notes,
        summary: String::new(),
    };

    // Summarizer runs on the aggregated outputs.
    directive.summary = summarizer::run_summary(&directive);

    tracing::info!(
        round = round,
        tick = directive.tick,
        total_findings = directive.auditor_findings.len()
            + directive.architect_arc_adjustments.len()
            + directive.analyst_foreshadow_notes.len(),
        summary_len = directive.summary.len(),
        "Council convening complete"
    );

    directive
}

/// Reset the round counter (useful for testing).
pub fn reset_round_counter() {
    COUNCIL_ROUND.store(0, Ordering::Relaxed);
}

/// Return the current round number without incrementing.
pub fn current_round() -> u64 {
    COUNCIL_ROUND.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input(tick: u64) -> CouncilInput {
        CouncilInput {
            tick,
            character_count: 5,
            recent_events: vec![
                "Zhang Yuan discovered a hidden cache".into(),
                "Lin Shuang argued with Old Sun".into(),
            ],
            active_threads: vec![
                "Blood Hand Gang threat".into(),
                "Lin Shuang's mysterious past (dormant)".into(),
            ],
        }
    }

    #[tokio::test]
    async fn test_convene_returns_directive() {
        reset_round_counter();
        let input = test_input(100);
        let directive = convene(input).await;

        assert_eq!(directive.round, 1);
        assert_eq!(directive.tick, 100);
        assert!(!directive.summary.is_empty());
    }

    #[tokio::test]
    #[ignore] // Global ROUND_COUNTER conflicts with parallel test execution
    async fn test_convene_increments_round() {
        reset_round_counter();
        let input = test_input(50);

        let d1 = convene(input.clone()).await;
        let d2 = convene(input).await;

        // Round increments with each convene (exact values depend on test ordering)
        assert!(d2.round > d1.round, "second round should be higher than first");
    }

    #[tokio::test]
    async fn test_convene_all_members_contribute() {
        reset_round_counter();
        let input = test_input(75);
        let directive = convene(input).await;

        // Auditory may be empty with generic input — other members always produce output
        assert!(!directive.architect_arc_adjustments.is_empty());
        assert!((0.0..=1.0).contains(&directive.assessor_score));
        // analyst and auditor may return empty for simple input
    }

    #[tokio::test]
    async fn test_convene_empty_input() {
        reset_round_counter();
        let input = CouncilInput {
            tick: 0,
            character_count: 0,
            recent_events: vec![],
            active_threads: vec![],
        };
        let directive = convene(input).await;

        assert!(directive.auditor_findings.is_empty());
        assert!(!directive.architect_arc_adjustments.is_empty()); // tension suggestion always present
        assert!(directive.assessor_score < 0.1);
        assert!(directive.analyst_foreshadow_notes.is_empty());
        assert!(!directive.summary.is_empty());
    }

    #[test]
    fn test_narrative_directive_default() {
        let directive = NarrativeDirective::default();
        assert_eq!(directive.round, 0);
        assert!(directive.auditor_findings.is_empty());
    }
}
