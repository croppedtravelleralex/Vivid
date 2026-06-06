use std::collections::HashMap;
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

#[derive(Clone)]
struct SessionEntry {
    session_id: String,
    prompt_hash: u64,
    response: String,
    access_count: u64,
    last_access: u64,
}

pub struct GenerationalCacheLayer {
    sessions: Mutex<Vec<SessionEntry>>,
    pub max_sessions: usize,
    pub session_ttl_ticks: u64,
}

impl GenerationalCacheLayer {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(Vec::new()),
            max_sessions: 200,
            session_ttl_ticks: 100,
        }
    }

    fn hash_prompt(prompt: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in prompt.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }

    fn evict_lru(cache: &mut Vec<SessionEntry>, max: usize) {
        while cache.len() > max {
            // Find least recently accessed
            if let Some(idx) = cache
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.last_access)
                .map(|(i, _)| i)
            {
                cache.remove(idx);
            } else {
                break;
            }
        }
    }

    fn evict_stale(cache: &mut Vec<SessionEntry>, now: u64, ttl: u64) {
        cache.retain(|e| now.saturating_sub(e.last_access) < ttl);
    }
}

impl OptimizationLayer for GenerationalCacheLayer {
    fn name(&self) -> &'static str {
        "L10:GenerationalCache"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let prompt_hash = Self::hash_prompt(&request.prompt);
        let now = request.created_at;

        let mut sessions = self.sessions.lock().unwrap();

        // Evict stale entries
        Self::evict_stale(&mut sessions, now, self.session_ttl_ticks);

        // Look for cross-session cache hit
        for entry in sessions.iter_mut() {
            if entry.prompt_hash == prompt_hash {
                entry.access_count += 1;
                entry.last_access = now;
                result.cache_hit = true;
                result.prompt_rewritten = entry.response.clone();
                result.tokens_saved = request.prompt.len() as u32 / 2;
                tracing::info!(
                    "[L10] Generational cache HIT for {} (session={}, accesses={})",
                    request.char_name,
                    entry.session_id,
                    entry.access_count
                );
                return;
            }
        }

        // Cache miss: insert new entry
        Self::evict_lru(&mut sessions, self.max_sessions);
        sessions.push(SessionEntry {
            session_id: format!("{}_{}", request.char_name, now),
            prompt_hash,
            response: String::new(),
            access_count: 1,
            last_access: now,
        });

        tracing::info!(
            "[L10] Generational cache MISS for {} (hash={})",
            request.char_name,
            prompt_hash
        );
    }
}
