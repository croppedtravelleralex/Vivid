use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

#[derive(Clone)]
struct FallbackStrategy {
    name: String,
    capability: u8, // 0=basic, 9=full
    cost_multiplier: f64,
}

pub struct FallbackChainLayer {
    strategies: Vec<FallbackStrategy>,
    pub max_chain_length: usize,
}

impl FallbackChainLayer {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                FallbackStrategy {
                    name: "primary_llm".to_string(),
                    capability: 9,
                    cost_multiplier: 1.0,
                },
                FallbackStrategy {
                    name: "compressed_llm".to_string(),
                    capability: 7,
                    cost_multiplier: 0.5,
                },
                FallbackStrategy {
                    name: "rule_engine".to_string(),
                    capability: 4,
                    cost_multiplier: 0.05,
                },
                FallbackStrategy {
                    name: "default_response".to_string(),
                    capability: 1,
                    cost_multiplier: 0.0,
                },
            ],
            max_chain_length: 3,
        }
    }

    fn required_capability(priority: u8) -> u8 {
        match priority {
            0 => 0,
            1..=3 => 3,
            4..=6 => 5,
            7..=8 => 8,
            9 => 9,
            _ => 5,
        }
    }
}

impl OptimizationLayer for FallbackChainLayer {
    fn name(&self) -> &'static str {
        "L15:FallbackChain"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let needed = Self::required_capability(request.priority);

        // Build fallback chain: strategies sorted by capability that meet the need
        let mut chain: Vec<String> = Vec::new();
        let mut best_cost = f64::MAX;
        let mut best_strategy: Option<String> = None;

        for strategy in &self.strategies {
            if strategy.capability >= needed {
                chain.push(strategy.name.clone());
                if strategy.cost_multiplier < best_cost && strategy.capability >= needed {
                    best_cost = strategy.cost_multiplier;
                    best_strategy = Some(strategy.name.clone());
                }
            }
        }

        // Limit chain length
        if chain.len() > self.max_chain_length {
            chain.truncate(self.max_chain_length);
        }

        result.fallback_chain = chain;

        if let Some(ref best) = best_strategy {
            // Tag prompt with selected strategy
            let tag = format!("\n[FALLBACK_STRATEGY: {}]", best);
            request.prompt.push_str(&tag);

            tracing::info!(
                "[L15] Fallback chain for {} (needed={}, best={}, chain_len={})",
                request.char_name,
                needed,
                best,
                result.fallback_chain.len()
            );

            // If best strategy is not primary_llm, note significant savings
            if best != "primary_llm" {
                let savings = if best == "default_response" {
                    request.prompt.len() as u32 / 4 * 9 / 10
                } else {
                    request.prompt.len() as u32 / 4 / 2
                };
                result.tokens_saved += savings;
            }
        }
    }
}
