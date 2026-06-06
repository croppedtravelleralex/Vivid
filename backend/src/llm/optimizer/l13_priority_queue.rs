use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

#[derive(Debug, Clone, Eq, PartialEq)]
struct QueuedItem {
    priority: u8,
    created_at: u64,
    char_name: String,
}

impl Ord for QueuedItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier creation time
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.created_at.cmp(&self.created_at))
    }
}

impl PartialOrd for QueuedItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct PriorityQueueLayer {
    queue: Mutex<BinaryHeap<QueuedItem>>,
    pub max_queue_size: usize,
    pub aging_factor: u64,
}

impl PriorityQueueLayer {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(BinaryHeap::new()),
            max_queue_size: 200,
            aging_factor: 1, // priority boost per tick waited
        }
    }

    fn apply_aging(&self, item: &mut QueuedItem, current_tick: u64) {
        let wait_ticks = current_tick.saturating_sub(item.created_at);
        let aged_priority = item.priority.saturating_add(
            (wait_ticks / self.aging_factor) as u8,
        );
        // Cap aged priority at 9 to prevent starvation inversion
        item.priority = aged_priority.min(9);
    }
}

impl OptimizationLayer for PriorityQueueLayer {
    fn name(&self) -> &'static str {
        "L13:PriorityQueue"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let mut queue = self.queue.lock().unwrap();

        let mut new_item = QueuedItem {
            priority: request.priority,
            created_at: request.created_at,
            char_name: request.char_name.clone(),
        };

        // Apply aging — older items get priority boost to prevent starvation
        self.apply_aging(&mut new_item, request.created_at);

        // Check if this item should preempt or be queued
        let should_preempt = if let Some(top) = queue.peek() {
            new_item.priority > top.priority && new_item.priority >= 7
        } else {
            false
        };

        if should_preempt {
            // High-priority item preempts the queue
            tracing::info!(
                "[L13] Preempting queue for {} (aged_priority={})",
                request.char_name,
                new_item.priority
            );
            result.priority = new_item.priority;
            return;
        }

        // Push into queue and check if we should allow dispatch
        if queue.len() < self.max_queue_size {
            let should_dispatch = queue.peek().map(|f| f.char_name == request.char_name).unwrap_or(false);
            let front_priority = queue.peek().map(|f| f.priority).unwrap_or(0);

            if should_dispatch {
                queue.pop();
                tracing::info!(
                    "[L13] Dispatching {} from queue front (priority={})",
                    request.char_name,
                    front_priority
                );
                return;
            }

            queue.push(new_item);

            // Not at front; wait in queue
            result.should_dispatch = false;
            tracing::info!(
                "[L13] Queued (priority={}, queue_size={})",
                result.priority,
                queue.len()
            );
        }
    }
}

impl Default for PriorityQueueLayer {
    fn default() -> Self {
        Self::new()
    }
}
