use crate::llm::optimizer::{LLMOptimizationRequest, LLMOptimizationResult, OptimizationLayer};

pub struct CompressionLayer {
    max_context_lines: usize,
    max_line_length: usize,
    abbreviation_map: Vec<(&'static str, &'static str)>,
}

impl CompressionLayer {
    pub fn new() -> Self {
        Self {
            max_context_lines: 30,
            max_line_length: 200,
            abbreviation_map: vec![
                ("character", "char"),
                ("narrative", "narr"),
                ("conversation", "conv"),
                ("observation", "obs"),
                ("reflection", "refl"),
                ("background", "bg"),
                ("knowledge", "know"),
                ("relationship", "rel"),
                ("personality", "pers"),
                ("inventory", "inv"),
                ("location", "loc"),
                ("description", "desc"),
                ("information", "info"),
                ("situation", "sit"),
                ("decision", "dec"),
                ("consequence", "cons"),
                ("probability", "prob"),
                ("threshold", "thresh"),
                ("environment", "env"),
            ],
        }
    }

    fn apply_abbreviations(text: &str, map: &[(&str, &str)]) -> String {
        let mut result = text.to_string();
        for &(long, short) in map {
            result = result.replace(long, short);
        }
        result
    }

    fn truncate_lines(context: &[String], max_lines: usize) -> Vec<String> {
        if context.len() <= max_lines {
            return context.to_vec();
        }
        // Keep first N/2 and last N/2 lines
        let half = max_lines / 2;
        let mut kept: Vec<String> = context.iter().take(half).cloned().collect();
        kept.push(format!(
            "... [{} lines truncated] ...",
            context.len() - max_lines
        ));
        let skip = context.len().saturating_sub(half);
        kept.extend(context.iter().skip(skip).cloned());
        kept
    }
}

impl OptimizationLayer for CompressionLayer {
    fn name(&self) -> &'static str {
        "L14:Compression"
    }

    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult) {
        let original_len = request.prompt.len();

        // Apply abbreviations to prompt
        request.prompt = Self::apply_abbreviations(&request.prompt, &self.abbreviation_map);

        // Truncate overly long lines in prompt
        let prompt_lines: Vec<&str> = request.prompt.split('\n').collect();
        let mut compressed_lines: Vec<String> = Vec::with_capacity(prompt_lines.len());
        for line in &prompt_lines {
            if line.len() > self.max_line_length {
                let truncated: String = line.chars().take(self.max_line_length).collect();
                compressed_lines.push(format!("{}... [truncated]", truncated));
            } else {
                compressed_lines.push(line.to_string());
            }
        }
        request.prompt = compressed_lines.join("\n");

        // Truncate and compress context
        if !request.context.is_empty() {
            request.context = Self::truncate_lines(&request.context, self.max_context_lines);
            request.context = request
                .context
                .iter()
                .map(|c| Self::apply_abbreviations(c, &self.abbreviation_map))
                .collect();
        }

        let saved = original_len.saturating_sub(request.prompt.len());
        if saved > 0 {
            let tokens_saved = saved as u32 / 4;
            result.tokens_saved += tokens_saved;
            tracing::info!(
                "[L14] Compressed prompt for {} ({} chars -> {}, ~{} tokens saved)",
                request.char_name,
                original_len,
                request.prompt.len(),
                tokens_saved
            );
        }
    }
}
