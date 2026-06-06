use std::collections::HashMap;
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

struct PredictionPattern {
    phase: String,
    next_expected: String,
    probability: f64,
    pregenerated: Option<String>,
}

pub struct PreemptiveSynthesisLayer {
    patterns: Mutex<HashMap<String, Vec<PredictionPattern>>>,
    pub min_probability: f64,
}

impl PreemptiveSynthesisLayer {
    pub fn new() -> Self {
        Self {
            patterns: Mutex::new(HashMap::new()),
            min_probability: 0.6,
        }
    }

    fn predict_next_phase(current: &str) -> Option<&'static str> {
        match current {
            "greeting" => Some("response"),
            "action" => Some("reaction"),
            "observation" => Some("reflection"),
            "question" => Some("answer"),
            "combat_start" => Some("combat_action"),
            "trade" => Some("trade_complete"),
            "exploration" => Some("discovery"),
            _ => None,
        }
    }
}

impl OptimizationLayer for PreemptiveSynthesisLayer {
    fn name(&self) -> &'static str {
        "L8:PreemptiveSynthesis"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let mut patterns = self.patterns.lock().unwrap();

        // Check if this matches a predicted next phase
        let char_patterns = patterns.entry(request.char_name.clone()).or_default();

        let mut matched = false;
        char_patterns.retain(|p| {
            if p.next_expected == request.phase && p.probability >= self.min_probability {
                matched = true;
                if let Some(ref pregen) = p.pregenerated {
                    result.cache_hit = true;
                    result.prompt_rewritten = pregen.clone();
                    result.tokens_saved = request.prompt.len() as u32 / 2;
                    tracing::info!(
                        "[L8] Pre-synthesized response for {} phase={} (p={:.2})",
                        request.char_name,
                        request.phase,
                        p.probability
                    );
                }
                false // remove consumed prediction
            } else {
                true // keep others
            }
        });

        if !matched {
            // Predict next phase and create placeholder
            if let Some(next) = Self::predict_next_phase(&request.phase) {
                char_patterns.push(PredictionPattern {
                    phase: request.phase.clone(),
                    next_expected: next.to_string(),
                    probability: self.min_probability,
                    pregenerated: None,
                });
                tracing::info!(
                    "[L8] Registered prediction: {} -> {} for {}",
                    request.phase,
                    next,
                    request.char_name
                );
            }
        }
    }
}
