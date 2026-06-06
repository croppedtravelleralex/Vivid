use std::collections::HashMap;
use std::sync::Mutex;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

struct SemanticSignature {
    ngram_set: Vec<u64>,
    original: String,
}

pub struct SemanticDedupLayer {
    seen: Mutex<HashMap<String, SemanticSignature>>,
    pub ngram_size: usize,
    pub similarity_threshold: f64,
}

impl SemanticDedupLayer {
    pub fn new() -> Self {
        Self {
            seen: Mutex::new(HashMap::new()),
            ngram_size: 3,
            similarity_threshold: 0.80,
        }
    }

    /// Extract character-level n-gram hashes from text
    fn extract_ngrams(text: &str, n: usize) -> Vec<u64> {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() < n {
            return vec![];
        }
        chars
            .windows(n)
            .map(|w| {
                let mut h: u64 = 0;
                for &c in w {
                    h = h.wrapping_mul(131).wrapping_add(c as u64);
                }
                h
            })
            .collect()
    }

    /// Compute n-gram overlap coefficient
    fn ngram_similarity(a: &[u64], b: &[u64]) -> f64 {
        if a.is_empty() && b.is_empty() {
            return 1.0;
        }
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let set_a: std::collections::HashSet<u64> = a.iter().copied().collect();
        let set_b: std::collections::HashSet<u64> = b.iter().copied().collect();

        let intersection = set_a.intersection(&set_b).count();
        let min_size = set_a.len().min(set_b.len());

        if min_size == 0 {
            return 0.0;
        }
        intersection as f64 / min_size as f64
    }
}

impl OptimizationLayer for SemanticDedupLayer {
    fn name(&self) -> &'static str {
        "L11:SemanticDedup"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        if request.context.is_empty() {
            return;
        }

        let mut seen = self.seen.lock().unwrap();

        // For each context block, check if semantically similar to a seen block
        let mut deduped: Vec<String> = Vec::with_capacity(request.context.len());
        let mut removed: usize = 0;

        for block in &request.context {
            let block_ngrams = Self::extract_ngrams(block, self.ngram_size);
            if block_ngrams.is_empty() {
                deduped.push(block.clone());
                continue;
            }

            let mut is_duplicate = false;
            for entry in seen.values() {
                let sim = Self::ngram_similarity(&block_ngrams, &entry.ngram_set);
                if sim >= self.similarity_threshold {
                    is_duplicate = true;
                    break;
                }
            }

            if is_duplicate {
                removed += block.len();
            } else {
                seen.insert(
                    format!("{}_{}", request.char_name, block.len()),
                    SemanticSignature {
                        ngram_set: block_ngrams,
                        original: block.clone(),
                    },
                );
                deduped.push(block.clone());
            }
        }

        if removed > 0 {
            let tokens_saved = removed as u32 / 4;
            result.tokens_saved += tokens_saved;
            request.context = deduped;
            tracing::info!(
                "[L11] Semantic dedup removed {} chars (~{} tokens) for {}",
                removed,
                tokens_saved,
                request.char_name
            );
        }
    }
}
