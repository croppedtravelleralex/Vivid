use std::collections::HashMap;
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

struct CharRateLimit {
    call_timestamps: Vec<u64>,
    tokens_used: u32,
}

pub struct FrequencyControlLayer {
    limits: Mutex<HashMap<String, CharRateLimit>>,
    pub max_calls_per_min: u32,
    pub max_tokens_per_min: u32,
    pub burst_size: u32,
}

impl FrequencyControlLayer {
    pub fn new() -> Self {
        Self {
            limits: Mutex::new(HashMap::new()),
            max_calls_per_min: 10,
            max_tokens_per_min: 32000,
            burst_size: 3,
        }
    }

    fn prune_old(ts: &mut Vec<u64>, now: u64, window_ms: u64) {
        let cutoff = now.saturating_sub(window_ms);
        ts.retain(|&t| t > cutoff);
    }
}

impl OptimizationLayer for FrequencyControlLayer {
    fn name(&self) -> &'static str {
        "L5:FrequencyControl"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let now = request.created_at;
        let window_ms: u64 = 60_000; // 1 minute

        let mut limits = self.limits.lock().unwrap();
        let entry = limits
            .entry(request.char_name.clone())
            .or_insert(CharRateLimit {
                call_timestamps: Vec::new(),
                tokens_used: 0,
            });

        // Prune old entries outside the window
        Self::prune_old(&mut entry.call_timestamps, now, window_ms);

        // Count calls within window
        let calls_in_window = entry.call_timestamps.len() as u32;

        // Allow burst if within burst limit
        if calls_in_window < self.burst_size {
            entry.call_timestamps.push(now);
            tracing::info!(
                "[L5] Burst slot for {} (calls_in_window={})",
                request.char_name,
                calls_in_window
            );
            return;
        }

        // Enforce per-char rate limit
        if calls_in_window >= self.max_calls_per_min {
            result.should_dispatch = false;
            tracing::info!(
                "[L5] Rate limit exceeded for {} (calls={}/{})",
                request.char_name,
                calls_in_window,
                self.max_calls_per_min
            );
            return;
        }

        // Enforce token budget per window
        let estimated_tokens = request.prompt.len() as u32 / 4;
        if entry.tokens_used + estimated_tokens > self.max_tokens_per_min {
            result.should_dispatch = false;
            tracing::info!(
                "[L5] Token budget exceeded for {} (used={}, estimated={})",
                request.char_name,
                entry.tokens_used,
                estimated_tokens
            );
            return;
        }

        // Record the call
        entry.call_timestamps.push(now);
        entry.tokens_used += estimated_tokens;
    }
}
