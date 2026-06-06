use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

struct DeadlineEstimator {
    avg_response_ms: f64,
    sample_count: u64,
}

pub struct DeadlineAwareLayer {
    estimator: Mutex<DeadlineEstimator>,
    /// Base deadline in ms based on priority level
    pub base_deadlines_ms: [u64; 10],
}

impl DeadlineAwareLayer {
    pub fn new() -> Self {
        Self {
            estimator: Mutex::new(DeadlineEstimator {
                avg_response_ms: 200.0,
                sample_count: 0,
            }),
            base_deadlines_ms: [
                0,   // priority 0: cancel
                500, // priority 1: generous
                400, // priority 2
                300, // priority 3
                200, // priority 4
                150, // priority 5
                120, // priority 6
                100, // priority 7
                80,  // priority 8
                30,  // priority 9: immediate
            ],
        }
    }

    fn estimate_execution_ms(&self, prompt_len: usize) -> f64 {
        let est = self
            .estimator
            .lock()
            .unwrap()
            .avg_response_ms;
        // Scale by prompt length relative to a baseline of 1000 chars
        let scale = (prompt_len as f64 / 1000.0).max(0.1).min(10.0);
        est * scale
    }
}

impl OptimizationLayer for DeadlineAwareLayer {
    fn name(&self) -> &'static str {
        "L12:DeadlineAware"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let priority = request.priority.clamp(0, 9) as usize;
        let base_deadline = self.base_deadlines_ms[priority];

        if priority == 0 {
            result.should_dispatch = false;
            return;
        }

        // Estimate execution time
        let estimated_ms = self.estimate_execution_ms(request.prompt.len());
        let estimated_u64 = estimated_ms as u64;

        // Compute deadline: base + buffer for priority
        let deadline = if base_deadline < estimated_u64 {
            // Even at generous estimate, this won't make it — adjust
            tracing::info!(
                "[L12] Deadline risk for {} priority={}: base={}ms < est={}ms",
                request.char_name,
                priority,
                base_deadline,
                estimated_u64
            );
            // Use estimated as minimum viable deadline
            estimated_u64 + 50
        } else {
            base_deadline
        };

        result.deadline_ms = deadline;

        // Mark should_dispatch based on whether deadline is feasible
        result.should_dispatch = deadline < 5000; // 5s absolute max

        // Update running average
        {
            let mut est = self.estimator.lock().unwrap();
            est.avg_response_ms = (est.avg_response_ms * est.sample_count as f64
                + estimated_ms)
                / (est.sample_count + 1) as f64;
            est.sample_count = est.sample_count.saturating_add(1);
        }

        tracing::info!(
            "[L12] Deadline {}ms for {} (priority={}, est={}ms)",
            deadline,
            request.char_name,
            priority,
            estimated_u64
        );
    }
}
