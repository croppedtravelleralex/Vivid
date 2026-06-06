use std::collections::HashMap;
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

struct SimilarityGroup {
    centroid_hash: u64,
    response: String,
    member_keys: Vec<u64>,
}

pub struct SimilarityRouterLayer {
    groups: Mutex<Vec<SimilarityGroup>>,
    pub similarity_threshold: f64,
    pub max_groups: usize,
}

impl SimilarityRouterLayer {
    pub fn new() -> Self {
        Self {
            groups: Mutex::new(Vec::new()),
            similarity_threshold: 0.85,
            max_groups: 100,
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

    /// Compute a simple similarity score between two prompts
    fn compute_similarity(a: &str, b: &str) -> f64 {
        if a.is_empty() && b.is_empty() {
            return 1.0;
        }
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        // Jaccard-like similarity over word tokens
        let words_a: Vec<&str> = a.split_whitespace().collect();
        let words_b: Vec<&str> = b.split_whitespace().collect();

        let set_a: std::collections::HashSet<&str> = words_a.iter().copied().collect();
        let set_b: std::collections::HashSet<&str> = words_b.iter().copied().collect();

        if set_a.is_empty() && set_b.is_empty() {
            return 1.0;
        }

        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        intersection as f64 / union as f64
    }
}

impl OptimizationLayer for SimilarityRouterLayer {
    fn name(&self) -> &'static str {
        "L7:SimilarityRouter"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let prompt_hash = Self::hash_prompt(&request.prompt);
        let mut groups = self.groups.lock().unwrap();

        // Search existing groups for similarity
        for group in groups.iter() {
            let sim = Self::compute_similarity(
                &request.prompt,
                &group.response,
            );
            if sim >= self.similarity_threshold {
                result.cache_hit = true;
                result.prompt_rewritten = group.response.clone();
                result.tokens_saved = (request.prompt.len() as u32 * 3) / 4;
                tracing::info!(
                    "[L7] Similarity route for {} (sim={:.2}, group_members={})",
                    request.char_name,
                    sim,
                    group.member_keys.len()
                );
                return;
            }
        }

        // Find or create a new group
        let new_group = SimilarityGroup {
            centroid_hash: prompt_hash,
            response: request.prompt.clone(),
            member_keys: vec![prompt_hash],
        };

        if groups.len() < self.max_groups {
            groups.push(new_group);
        } else {
            // Replace least-used group
            groups[0] = new_group;
        }
    }
}
