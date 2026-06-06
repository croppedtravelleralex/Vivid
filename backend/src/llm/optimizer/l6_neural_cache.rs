use std::collections::HashMap;
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

struct CachedEntry {
    prompt_hash: u64,
    response_summary: String,
    hit_count: u64,
}

pub struct NeuralCacheLayer {
    cache: Mutex<HashMap<u64, CachedEntry>>,
    pub max_entries: usize,
    pub min_hits_for_promotion: u64,
}

impl NeuralCacheLayer {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            max_entries: 500,
            min_hits_for_promotion: 3,
        }
    }

    /// Simple djb2 hash for prompt fingerprinting
    fn hash_prompt(prompt: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in prompt.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }

    fn evict_if_needed(&self, cache: &mut HashMap<u64, CachedEntry>) {
        if cache.len() >= self.max_entries {
            // Evict entry with lowest hit count
            if let Some(min_key) = cache
                .iter()
                .min_by_key(|(_, v)| v.hit_count)
                .map(|(k, _)| *k)
            {
                cache.remove(&min_key);
            }
        }
    }
}

impl OptimizationLayer for NeuralCacheLayer {
    fn name(&self) -> &'static str {
        "L6:NeuralCache"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let prompt_hash = Self::hash_prompt(&request.prompt);
        let mut cache = self.cache.lock().unwrap();

        if let Some(entry) = cache.get_mut(&prompt_hash) {
            entry.hit_count += 1;

            // Only serve from cache if hit count exceeds threshold
            // (avoids caching one-off prompts)
            if entry.hit_count >= self.min_hits_for_promotion {
                result.cache_hit = true;
                result.prompt_rewritten = entry.response_summary.clone();
                result.tokens_saved = request.prompt.len() as u32 / 2;
                tracing::info!(
                    "[L6] Neural cache HIT for {} (hash={}, hits={})",
                    request.char_name,
                    prompt_hash,
                    entry.hit_count
                );
                return;
            }
        } else {
            // Insert new entry
            Self::evict_if_needed(self, &mut cache);
            cache.insert(
                prompt_hash,
                CachedEntry {
                    prompt_hash,
                    response_summary: String::new(),
                    hit_count: 1,
                },
            );
        }

        tracing::info!(
            "[L6] Neural cache MISS for {} (hash={})",
            request.char_name,
            prompt_hash
        );
    }
}
