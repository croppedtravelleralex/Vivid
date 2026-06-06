use std::collections::HashSet;

use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

pub struct DeduplicationLayer {
    min_block_length: usize,
}

impl DeduplicationLayer {
    pub fn new() -> Self {
        Self {
            min_block_length: 20,
        }
    }

    /// Split context into blocks by newlines
    fn split_blocks(context: &[String]) -> Vec<&str> {
        context.iter().flat_map(|s| s.split('\n')).collect()
    }
}

impl OptimizationLayer for DeduplicationLayer {
    fn name(&self) -> &'static str {
        "L4:Deduplication"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        if request.context.len() < 2 {
            return;
        }

        // Deduplicate context blocks
        let mut seen: HashSet<&str> = HashSet::new();
        let mut deduped: Vec<String> = Vec::with_capacity(request.context.len());
        let mut removed_chars: usize = 0;

        for block in &request.context {
            if block.len() < self.min_block_length {
                // Keep small blocks (likely unique identifiers)
                deduped.push(block.clone());
                continue;
            }

            if seen.contains(block.as_str()) {
                removed_chars += block.len();
                continue;
            }

            seen.insert(block.as_str());
            deduped.push(block.clone());
        }

        if removed_chars > 0 {
            let tokens_saved = removed_chars as u32 / 4;
            result.tokens_saved += tokens_saved;
            request.context = deduped;
            tracing::info!(
                "[L4] Removed {} duplicate chars (~{} tokens) for {}",
                removed_chars,
                tokens_saved,
                request.char_name
            );
        }

        // Also deduplicate prompt lines
        let prompt_blocks: Vec<&str> = request.prompt.split('\n').collect();
        let mut p_seen: HashSet<&str> = HashSet::new();
        let mut p_deduped: Vec<&str> = Vec::with_capacity(prompt_blocks.len());

        for line in &prompt_blocks {
            if line.len() < self.min_block_length || p_seen.insert(line) {
                p_deduped.push(line);
            }
        }

        if p_deduped.len() < prompt_blocks.len() {
            request.prompt = p_deduped.join("\n");
        }
    }
}
