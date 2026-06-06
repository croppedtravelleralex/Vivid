use serde::{Deserialize, Serialize};

use super::CouncilInput;

/// Describes where a character currently sits in their narrative arc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterArcState {
    pub character_name: String,
    pub current_stage: ArcStage,
    pub ticks_since_last_event: u64,
    pub target_tension: f64,
}

/// Standard narrative arc stages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArcStage {
    /// Character is in a static / everyday state.
    Normal,
    /// Inciting incident — something disrupts the status quo.
    IncitingIncident,
    /// Rising action — tension builds.
    RisingAction,
    /// Climax — the peak of conflict.
    Climax,
    /// Falling action — consequences play out.
    FallingAction,
    /// Resolution — new equilibrium.
    Resolution,
    /// Character arcs that are dormant / waiting for screen time.
    Dormant,
}

impl ArcStage {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            ArcStage::Normal => "normal",
            ArcStage::IncitingIncident => "inciting_incident",
            ArcStage::RisingAction => "rising_action",
            ArcStage::Climax => "climax",
            ArcStage::FallingAction => "falling_action",
            ArcStage::Resolution => "resolution",
            ArcStage::Dormant => "dormant",
        }
    }
}

/// A suggested adjustment to narrative direction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcAdjustment {
    pub character: String,
    pub suggestion: String,
    pub urgency: f64, // 0.0–1.0
}

/// Complete architectural directive produced by the arc architect.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArcDirective {
    pub adjustments: Vec<ArcAdjustment>,
    pub suggested_target_tension: Option<f64>,
    pub pacing_note: Option<String>,
}

/// Run the arc architect analysis.
///
/// Evaluates each thread as a proxy for character arcs, determines stage
/// based on recency of activity, and suggests tension adjustments.
pub fn run_architecture(input: &CouncilInput) -> Vec<String> {
    tracing::info!(
        tick = input.tick,
        thread_count = input.active_threads.len(),
        "Arc Architect: starting narrative arc analysis"
    );

    let directive = analyze_arcs(input);

    let mut adjustments: Vec<String> = directive
        .adjustments
        .iter()
        .map(|adj| {
            format!(
                "[arc] {} — {} (urgency: {:.2})",
                adj.character, adj.suggestion, adj.urgency
            )
        })
        .collect();

    if let Some(tension) = directive.suggested_target_tension {
        adjustments.push(format!(
            "[tension] Suggested target tension: {:.2}",
            tension
        ));
    }

    if let Some(pacing) = &directive.pacing_note {
        adjustments.push(format!("[pacing] {}", pacing));
    }

    tracing::info!(
        tick = input.tick,
        adjustment_count = adjustments.len(),
        "Arc Architect: analysis complete"
    );

    adjustments
}

/// Core analysis logic — maps active threads to arc stages and derives
/// adjustments.
fn analyze_arcs(input: &CouncilInput) -> ArcDirective {
    let mut directive = ArcDirective::default();

    // Treat each active thread as a character arc proxy.
    for thread in &input.active_threads {
        let stage = classify_arc_stage(thread, input);
        let urgency = compute_urgency(&stage, input.tick);

        let suggestion = match &stage {
            ArcStage::Dormant => format!(
                "Thread '{}' is dormant — consider reviving with a triggering event",
                thread
            ),
            ArcStage::IncitingIncident => format!(
                "Thread '{}' at inciting incident — escalate toward rising action",
                thread
            ),
            ArcStage::RisingAction => format!(
                "Thread '{}' in rising action — maintain momentum, introduce complications",
                thread
            ),
            ArcStage::Climax => format!(
                "Thread '{}' approaching climax — resolve key conflicts",
                thread
            ),
            ArcStage::FallingAction => format!(
                "Thread '{}' in falling action — show consequences, tie off loose ends",
                thread
            ),
            ArcStage::Resolution => format!(
                "Thread '{}' resolved — consider archiving or seeding a follow-up",
                thread
            ),
            ArcStage::Normal => format!(
                "Thread '{}' in steady state — inject minor disruption to maintain interest",
                thread
            ),
        };

        directive.adjustments.push(ArcAdjustment {
            character: thread.clone(),
            suggestion,
            urgency,
        });
    }

    // Suggest target tension based on event volume
    let density = if input.tick > 0 {
        input.recent_events.len() as f64 / input.tick as f64
    } else {
        0.0
    };

    if density < 0.02 {
        directive.suggested_target_tension = Some(0.65);
        directive.pacing_note = Some(
            "Event density is low — consider accelerating the narrative pace".into(),
        );
    } else if density > 0.2 {
        directive.suggested_target_tension = Some(0.40);
        directive.pacing_note = Some(
            "Event density is high — consider a breather sequence".into(),
        );
    } else {
        directive.suggested_target_tension = Some(0.55);
    }

    directive
}

/// Classify a thread into an arc stage based on heuristics.
fn classify_arc_stage(thread: &str, input: &CouncilInput) -> ArcStage {
    let lower = thread.to_lowercase();

    // Check against keyword indicators for each stage
    if lower.contains("dormant")
        || lower.contains("stale")
        || lower.contains("unresolved")
    {
        return ArcStage::Dormant;
    }
    if lower.contains("climax")
        || lower.contains("showdown")
        || lower.contains("final")
    {
        return ArcStage::Climax;
    }
    if lower.contains("begin")
        || lower.contains("start")
        || lower.contains("discover")
        || lower.contains("arrive")
    {
        return ArcStage::IncitingIncident;
    }
    if lower.contains("resolve")
        || lower.contains("recover")
        || lower.contains("rebuild")
    {
        return ArcStage::Resolution;
    }
    if lower.contains("fallout")
        || lower.contains("consequence")
        || lower.contains("aftermath")
    {
        return ArcStage::FallingAction;
    }

    // If the thread hasn't been mentioned in recent events, mark as dormant.
    let recently_mentioned = input
        .recent_events
        .iter()
        .any(|e| e.to_lowercase().contains(&lower));

    if !recently_mentioned {
        return ArcStage::Dormant;
    }

    // Default to rising action if we see recent activity.
    ArcStage::RisingAction
}

/// Compute urgency based on arc stage and elapsed ticks.
fn compute_urgency(stage: &ArcStage, _tick: u64) -> f64 {
    match stage {
        ArcStage::Climax => 0.9,
        ArcStage::RisingAction => 0.7,
        ArcStage::IncitingIncident => 0.6,
        ArcStage::Dormant => 0.5,
        ArcStage::FallingAction => 0.4,
        ArcStage::Resolution => 0.3,
        ArcStage::Normal => 0.2,
    }
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
                "Zhang Yuan discovered a hidden cache of supplies".into(),
                "The group argued about their next move".into(),
            ],
            active_threads: vec![
                "Blood Hand Gang threat".into(),
                "Lin Shuang's mysterious past (dormant)".into(),
            ],
        }
    }

    #[test]
    fn test_run_architecture_returns_adjustments() {
        let input = sample_input();
        let adjustments = run_architecture(&input);
        assert!(!adjustments.is_empty());
    }

    #[test]
    fn test_tension_adjustment_included() {
        let input = sample_input();
        let adjustments = run_architecture(&input);
        let has_tension = adjustments.iter().any(|a| a.contains("[tension]"));
        assert!(has_tension);
    }

    #[test]
    fn test_dormant_stage_detected() {
        let input = sample_input();
        let adjustments = run_architecture(&input);
        let dormant_adjustments: Vec<&String> = adjustments
            .iter()
            .filter(|a| a.contains("dormant"))
            .collect();
        assert!(!dormant_adjustments.is_empty());
    }

    #[test]
    fn test_empty_input_produces_baseline() {
        let input = CouncilInput {
            tick: 0,
            character_count: 0,
            recent_events: vec![],
            active_threads: vec![],
        };
        let adjustments = run_architecture(&input);
        // Even empty input should produce a tension suggestion
        assert!(adjustments.iter().any(|a| a.contains("[tension]")));
    }

    #[test]
    fn test_arc_stage_classification() {
        let input = sample_input();
        assert_eq!(
            classify_arc_stage("the final showdown approaches", &input),
            ArcStage::Climax
        );
        assert_eq!(
            classify_arc_stage("a new discovery", &input),
            ArcStage::IncitingIncident
        );
        assert_eq!(
            classify_arc_stage("rebuilding the shelter", &input),
            ArcStage::Resolution
        );
    }
}
