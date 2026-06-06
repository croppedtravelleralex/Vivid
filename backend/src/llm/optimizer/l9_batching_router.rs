use std::collections::HashMap;
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

#[derive(Clone)]
struct BatchGroup {
    batch_id: String,
    member_phases: Vec<String>,
    created_at: u64,
}

pub struct BatchingRouterLayer {
    groups: Mutex<HashMap<String, BatchGroup>>,
    pub batch_window_ms: u64,
    pub max_batch_size: usize,
}

impl BatchingRouterLayer {
    pub fn new() -> Self {
        Self {
            groups: Mutex::new(HashMap::new()),
            batch_window_ms: 200,
            max_batch_size: 20,
        }
    }

    fn determine_batch_key(request: &LLMOptimizationRequest) -> String {
        // Same character + same broad phase category = batchable
        let broad_phase = match request.phase.as_str() {
            "greeting" | "response" | "answer" => "dialogue",
            "action" | "reaction" | "combat_action" => "action",
            "observation" | "reflection" | "discovery" => "cognition",
            _ => &request.phase,
        };
        format!("{}:{}", request.char_name, broad_phase)
    }
}

impl OptimizationLayer for BatchingRouterLayer {
    fn name(&self) -> &'static str {
        "L9:BatchingRouter"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let batch_key = Self::determine_batch_key(request);
        let mut groups = self.groups.lock().unwrap();

        if let Some(group) = groups.get_mut(&batch_key) {
            group.member_phases.push(request.phase.clone());

            if group.member_phases.len() >= self.max_batch_size {
                // Group is full, should dispatch as batch
                result.prompt_rewritten = format!(
                    "[BATCH:{}] {} requests merged for {}",
                    group.batch_id,
                    group.member_phases.len(),
                    request.char_name
                );
                result.tokens_saved = (group.member_phases.len() as u32 - 1) * 200;
                tracing::info!(
                    "[L9] Batch full for {} (size={}, phases={:?})",
                    request.char_name,
                    group.member_phases.len(),
                    group.member_phases
                );

                // Reset group after dispatch
                groups.remove(&batch_key);
                return;
            }

            // Not yet full, defer dispatch
            result.should_dispatch = false;
            tracing::info!(
                "[L9] Buffered in batch {} (size={}/{})",
                batch_key,
                group.member_phases.len(),
                self.max_batch_size
            );
        } else {
            // Start new batch group
            let group = BatchGroup {
                batch_id: format!("{}_{}", request.char_name, request.created_at),
                member_phases: vec![request.phase.clone()],
                created_at: request.created_at,
            };
            groups.insert(batch_key.clone(), group);
            result.should_dispatch = false; // Wait for more items
            tracing::info!(
                "[L9] New batch started for {}",
                batch_key
            );
        }
    }
}
