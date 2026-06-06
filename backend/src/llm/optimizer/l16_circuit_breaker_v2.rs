use std::collections::VecDeque;
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

struct FailureRecord {
    timestamp: u64,
    error_kind: String,
}

pub struct CircuitBreakerV2Layer {
    failure_history: Mutex<VecDeque<FailureRecord>>,
    pub window_ms: u64,
    pub max_failures: u32,
    pub recovery_threshold_ms: u64,
    pub success_streak_reset: u32,
}

impl CircuitBreakerV2Layer {
    pub fn new() -> Self {
        Self {
            failure_history: Mutex::new(VecDeque::with_capacity(200)),
            window_ms: 60_000,
            max_failures: 5,
            recovery_threshold_ms: 30_000,
            success_streak_reset: 3,
        }
    }

    fn failure_rate(&self, now: u64) -> (u32, u32) {
        let hist = self.failure_history.lock().unwrap();
        let cutoff = now.saturating_sub(self.window_ms);
        let recent: u32 = hist.iter().filter(|r| r.timestamp > cutoff).count() as u32;
        let total = hist.len() as u32;
        (recent, total)
    }

    /// Predict failure probability based on recent failure rate and error types
    fn predict_failure_probability(&self, now: u64) -> f64 {
        let (recent, total) = self.failure_rate(now);
        if total == 0 {
            return 0.0;
        }
        let rate = recent as f64 / total as f64;

        // If recent failures are escalating, predict higher probability
        let hist = self.failure_history.lock().unwrap();
        let mut escalating = 0.0;
        if hist.len() >= 3 {
            let recent_three: Vec<&FailureRecord> = hist.iter().rev().take(3).collect();
            if recent_three.len() == 3 {
                let t1 = recent_three[0].timestamp;
                let t2 = recent_three[1].timestamp;
                let t3 = recent_three[2].timestamp;
                // Failures getting closer together = escalating
                if t1 > t2 && t2 > t3 && (t1 - t2) < (t2 - t3) {
                    escalating = 0.2;
                }
            }
        }

        (rate + escalating).min(1.0)
    }
}

impl OptimizationLayer for CircuitBreakerV2Layer {
    fn name(&self) -> &'static str {
        "L16:CircuitBreakerV2"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let now = request.created_at;
        let predicted = self.predict_failure_probability(now);
        let (recent_failures, total) = self.failure_rate(now);

        // ML-predictive circuit breaker logic
        if predicted > 0.7 || recent_failures >= self.max_failures {
            // Circuit OPEN — block dispatch, route to fallback
            result.should_dispatch = false;
            result.fallback_chain = vec![
                "cached_fallback".to_string(),
                "rule_engine".to_string(),
                "default_response".to_string(),
            ];

            tracing::info!(
                "[L16] Circuit OPEN for {} (predicted={:.2}, recent_failures={}, total={})",
                request.char_name,
                predicted,
                recent_failures,
                total
            );
            return;
        }

        if predicted > 0.4 {
            // Circuit HALF-OPEN — allow but with compressed prompt
            let original_len = request.prompt.len();
            request.prompt.truncate(original_len / 2);
            tracing::info!(
                "[L16] Circuit HALF-OPEN for {} (predicted={:.2})",
                request.char_name,
                predicted
            );
            return;
        }

        // Circuit CLOSED — normal operation
        tracing::info!(
            "[L16] Circuit CLOSED for {} (predicted={:.2}, recent_failures={})",
            request.char_name,
            predicted,
            recent_failures
        );
    }
}
