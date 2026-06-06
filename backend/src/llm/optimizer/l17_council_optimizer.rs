use std::collections::HashMap;
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

/// Cross-council state: tracks narrative coherence across multiple characters
struct CouncilState {
    council_id: String,
    last_narrative_tick: u64,
    narrative_summary: String,
    member_characters: Vec<String>,
    coherence_score: f64,
}

pub struct CouncilOptimizerLayer {
    councils: Mutex<HashMap<String, CouncilState>>,
    pub max_councils: usize,
    pub coherence_threshold: f64,
}

impl CouncilOptimizerLayer {
    pub fn new() -> Self {
        Self {
            councils: Mutex::new(HashMap::new()),
            max_councils: 10,
            coherence_threshold: 0.7,
        }
    }

    fn determine_council(request: &LLMOptimizationRequest) -> String {
        // Group by phase category for narrative coherence
        let category = match request.phase.as_str() {
            "greeting" | "response" | "answer" | "question" | "dialogue" => {
                "dialogue_council"
            }
            "action" | "reaction" | "combat_action" | "combat_start" => "action_council",
            "observation" | "reflection" | "discovery" | "exploration" => "cognition_council",
            "trade" | "trade_complete" | "inventory" => "economy_council",
            _ => "general_council",
        };
        format!("{}:{}", category, request.char_name.chars().next().unwrap_or('?'))
    }

    fn update_coherence(
        council: &mut CouncilState,
        request: &LLMOptimizationRequest,
        result: &LLMOptimizationResult,
    ) {
        // Coherence increases if we have cache hits (consistent responses)
        if result.cache_hit {
            council.coherence_score =
                (council.coherence_score + 0.05).min(1.0);
        }

        // Coherence decreases on priority mismatches
        if result.priority > 0 && result.priority < 3 {
            council.coherence_score =
                (council.coherence_score - 0.02).max(0.0);
        }

        // Narrative continuity bonus
        if council.last_narrative_tick > 0
            && request.created_at > council.last_narrative_tick
            && request.created_at - council.last_narrative_tick < 5
        {
            council.coherence_score =
                (council.coherence_score + 0.01).min(1.0);
        }

        council.last_narrative_tick = request.created_at;
    }
}

impl OptimizationLayer for CouncilOptimizerLayer {
    fn name(&self) -> &'static str {
        "L17:CouncilOptimizer"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let council_id = Self::determine_council(request);
        let mut councils = self.councils.lock().unwrap();

        // Evict excess councils (oldest first)
        while councils.len() >= self.max_councils {
            if let Some(oldest_id) = councils
                .iter()
                .min_by_key(|(_, c)| c.last_narrative_tick)
                .map(|(k, _)| k.clone())
            {
                councils.remove(&oldest_id);
            }
        }

        let council = councils.entry(council_id.clone()).or_insert(CouncilState {
            council_id: council_id.clone(),
            last_narrative_tick: request.created_at,
            narrative_summary: String::new(),
            member_characters: Vec::new(),
            coherence_score: 0.5, // start neutral
        });

        // Track members
        if !council.member_characters.contains(&request.char_name) {
            council.member_characters.push(request.char_name.clone());
        }

        // Update coherence metrics
        Self::update_coherence(council, request, result);

        // Append narrative summary for cross-council coherence
        let coherence_note = if council.coherence_score >= self.coherence_threshold {
            format!(
                "\n[COUNCIL {}] coherence={:.2}, members={}, last_tick={}",
                council_id,
                council.coherence_score,
                council.member_characters.len(),
                council.last_narrative_tick
            )
        } else {
            format!(
                "\n[COUNCIL {} WARN] coherence={:.2} below threshold",
                council_id, council.coherence_score
            )
        };

        request.prompt.push_str(&coherence_note);

        tracing::info!(
            "[L17] Council '{}' for {} (coherence={:.2}, members={})",
            council_id,
            request.char_name,
            council.coherence_score,
            council.member_characters.len()
        );
    }
}
