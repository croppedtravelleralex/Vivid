use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

pub struct PriorityBudgetingLayer {
    pub max_budget_per_char: u32,
    pub critical_threshold: u8,
}

impl PriorityBudgetingLayer {
    pub fn new() -> Self {
        Self {
            max_budget_per_char: 4000,
            critical_threshold: 7,
        }
    }
}

impl OptimizationLayer for PriorityBudgetingLayer {
    fn name(&self) -> &'static str {
        "L0:PriorityBudgeting"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        // Priority 0 = cancel
        if request.priority == 0 {
            result.should_dispatch = false;
            tracing::info!(
                "[L0] Cancelled request for {} (priority=0)",
                request.char_name
            );
            return;
        }

        // Truncate prompt to budget
        let budget = if request.priority >= self.critical_threshold {
            self.max_budget_per_char * 2
        } else {
            self.max_budget_per_char
        };

        if request.prompt.len() > budget as usize {
            let excess = request.prompt.len() - budget as usize;
            request.prompt.truncate(budget as usize);
            let tokens_est = excess as u32 / 4;
            result.tokens_saved += tokens_est;
            tracing::info!(
                "[L0] Truncated prompt for {} by {} chars (~{} tokens)",
                request.char_name,
                excess,
                tokens_est
            );
        }

        // Set result priority for downstream layers
        result.priority = request.priority;
    }
}
