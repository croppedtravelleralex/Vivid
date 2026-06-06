use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

struct UsageSnapshot {
    recent_tokens: Vec<u32>,
    recent_counts: Vec<u64>,
}

pub struct AdaptiveTokenLimitLayer {
    history: Mutex<UsageSnapshot>,
    pub base_token_limit: u32,
    pub max_token_limit: u32,
    pub min_token_limit: u32,
}

impl AdaptiveTokenLimitLayer {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(UsageSnapshot {
                recent_tokens: Vec::with_capacity(20),
                recent_counts: Vec::with_capacity(20),
            }),
            base_token_limit: 4096,
            max_token_limit: 8192,
            min_token_limit: 512,
        }
    }
}

impl OptimizationLayer for AdaptiveTokenLimitLayer {
    fn name(&self) -> &'static str {
        "L3:AdaptiveTokenLimit"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let mut hist = self.history.lock().unwrap();

        // Record current request
        hist.recent_tokens.push(request.budget_tokens);
        hist.recent_counts.push(1);

        // Keep sliding window of 20
        if hist.recent_tokens.len() > 20 {
            hist.recent_tokens.remove(0);
            hist.recent_counts.remove(0);
        }

        // Compute recent average usage
        let avg_tokens: u32 = if hist.recent_tokens.is_empty() {
            self.base_token_limit
        } else {
            let sum: u32 = hist.recent_tokens.iter().sum();
            sum / hist.recent_tokens.len() as u32
        };

        // Adjust limit: high priority gets more tokens
        let priority_mult = match request.priority {
            0..=3 => 0.5,
            4..=6 => 1.0,
            7..=8 => 1.5,
            9 => 2.0,
            _ => 1.0,
        };

        // Calculate adaptive limit from recent usage + priority boost
        let mut adaptive_limit =
            (avg_tokens as f64 * priority_mult).round() as u32;
        adaptive_limit = adaptive_limit.clamp(self.min_token_limit, self.max_token_limit);

        // Clamp budget_tokens to adaptive limit
        if request.budget_tokens > adaptive_limit {
            let saved = request.budget_tokens - adaptive_limit;
            request.budget_tokens = adaptive_limit;
            result.tokens_saved += saved;
            tracing::info!(
                "[L3] Clamped budget for {} from {} to {} (avg={}, pri={})",
                request.char_name,
                request.budget_tokens + saved,
                adaptive_limit,
                avg_tokens,
                request.priority
            );
        }

        // Truncate prompt if it exceeds adaptive budget
        let prompt_limit = adaptive_limit as usize * 4; // rough char estimate
        if request.prompt.len() > prompt_limit {
            request.prompt.truncate(prompt_limit);
        }
    }
}
