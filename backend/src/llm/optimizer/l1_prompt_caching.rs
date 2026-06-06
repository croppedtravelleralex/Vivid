use std::collections::HashMap;
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

struct CacheEntry {
    cache_key: String,
    segment_count: usize,
}

pub struct PromptCachingLayer {
    cache_keys: Mutex<HashMap<String, CacheEntry>>,
}

impl PromptCachingLayer {
    pub fn new() -> Self {
        Self {
            cache_keys: Mutex::new(HashMap::new()),
        }
    }

    fn compute_cache_key(request: &LLMOptimizationRequest) -> String {
        let ctx_hash: String = request
            .context
            .iter()
            .map(|c| format!("{}:{}", c.len(), c.chars().count() % 100))
            .collect::<Vec<_>>()
            .join(",");
        format!("{}:{}:{}", request.char_name, request.phase, ctx_hash)
    }
}

impl OptimizationLayer for PromptCachingLayer {
    fn name(&self) -> &'static str {
        "L1:PromptCaching"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let key = Self::compute_cache_key(request);
        let mut keys = self.cache_keys.lock().unwrap();

        // Insert or update cache key metadata
        let entry = keys.entry(key.clone()).or_insert(CacheEntry {
            cache_key: key.clone(),
            segment_count: 0,
        });
        entry.segment_count += 1;

        // Add cache control marker to prompt
        let marker = format!("\n<!-- CACHE_KEY: {} -->\n", entry.cache_key);
        request.prompt.push_str(&marker);

        // Mark cache hit if this segment has been seen before
        if entry.segment_count > 1 {
            result.cache_hit = true;
            result.tokens_saved += request.prompt.len() as u32 / 4;
            tracing::info!(
                "[L1] Cache hit for {} (phase={}, segments={})",
                request.char_name,
                request.phase,
                entry.segment_count
            );
        } else {
            tracing::info!(
                "[L1] New cache entry for {} (phase={})",
                request.char_name,
                request.phase
            );
        }
    }
}
