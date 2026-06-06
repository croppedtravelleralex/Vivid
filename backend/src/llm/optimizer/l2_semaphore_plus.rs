use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

pub struct SemaphorePlusLayer {
    max_concurrent: u32,
    active_critical: AtomicU32,
    active_normal: AtomicU32,
    total_queued: Mutex<Vec<u8>>,
}

impl SemaphorePlusLayer {
    pub fn new() -> Self {
        Self {
            max_concurrent: 8,
            active_critical: AtomicU32::new(0),
            active_normal: AtomicU32::new(0),
            total_queued: Mutex::new(Vec::new()),
        }
    }

    /// Priority threshold: requests above this get semaphore priority
    const CRITICAL_PRIORITY: u8 = 6;
}

impl OptimizationLayer for SemaphorePlusLayer {
    fn name(&self) -> &'static str {
        "L2:SemaphorePlus"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let is_critical = request.priority >= Self::CRITICAL_PRIORITY;

        // Track queued priorities
        {
            let mut queued = self.total_queued.lock().unwrap();
            queued.push(request.priority);
            // Keep only last 100 entries
            if queued.len() > 100 {
                queued.remove(0);
            }
        }

        if is_critical {
            let active = self.active_critical.fetch_add(1, Ordering::SeqCst);
            // Critical requests always get a slot, even if it means preempting
            if active >= self.max_concurrent / 2 {
                // Allow queueing but don't block
                tracing::info!(
                    "[L2] Critical request queued for {} (active_critical={})",
                    request.char_name,
                    active
                );
            }
        } else {
            let active = self.active_normal.fetch_add(1, Ordering::SeqCst);
            let critical = self.active_critical.load(Ordering::SeqCst);
            let total_active = active + critical;

            if total_active >= self.max_concurrent {
                // Backpressure: throttle non-critical when congested
                result.should_dispatch = false;
                tracing::info!(
                    "[L2] Throttled non-critical request for {} (active={}, critical={})",
                    request.char_name,
                    total_active,
                    critical
                );
                return;
            }
        }

        tracing::info!(
            "[L2] Dispatch allowed for {} (priority={})",
            request.char_name,
            request.priority
        );
    }
}
