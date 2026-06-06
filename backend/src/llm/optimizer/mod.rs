pub mod l0_priority_budgeting;
pub mod l1_prompt_caching;
pub mod l2_semaphore_plus;
pub mod l3_adaptive_token_limit;
pub mod l4_deduplication;
pub mod l5_frequency_control;
pub mod l6_neural_cache;
pub mod l7_similarity_router;
pub mod l8_preemptive_synthesis;
pub mod l9_batching_router;
pub mod l10_generational_cache;
pub mod l11_semantic_dedup;
pub mod l12_deadline_aware;
pub mod l13_priority_queue;
pub mod l14_compression;
pub mod l15_fallback_chain;
pub mod l16_circuit_breaker_v2;
pub mod l17_council_optimizer;

/// The full optimization pipeline. Each layer wraps the next.
pub struct OptimizationPipeline {
    pub layers: Vec<Box<dyn OptimizationLayer>>,
}

impl OptimizationPipeline {
    pub fn new() -> Self {
        Self { layers: vec![] }
    }

    /// Add a layer to the pipeline
    pub fn add_layer(&mut self, layer: Box<dyn OptimizationLayer>) {
        self.layers.push(layer);
    }

    /// Process a request through all layers
    pub fn process(&self, request: &mut LLMOptimizationRequest) -> LLMOptimizationResult {
        let mut result = LLMOptimizationResult::default();
        for layer in &self.layers {
            layer.apply(request, &mut result);
        }
        result
    }

    /// Build default L0-L17 stack
    pub fn build_default() -> Self {
        let mut p = Self::new();
        p.add_layer(Box::new(
            l0_priority_budgeting::PriorityBudgetingLayer::new(),
        ));
        p.add_layer(Box::new(l1_prompt_caching::PromptCachingLayer::new()));
        p.add_layer(Box::new(l2_semaphore_plus::SemaphorePlusLayer::new()));
        p.add_layer(Box::new(
            l3_adaptive_token_limit::AdaptiveTokenLimitLayer::new(),
        ));
        p.add_layer(Box::new(l4_deduplication::DeduplicationLayer::new()));
        p.add_layer(Box::new(l5_frequency_control::FrequencyControlLayer::new()));
        p.add_layer(Box::new(l6_neural_cache::NeuralCacheLayer::new()));
        p.add_layer(Box::new(l7_similarity_router::SimilarityRouterLayer::new()));
        p.add_layer(Box::new(
            l8_preemptive_synthesis::PreemptiveSynthesisLayer::new(),
        ));
        p.add_layer(Box::new(l9_batching_router::BatchingRouterLayer::new()));
        p.add_layer(Box::new(
            l10_generational_cache::GenerationalCacheLayer::new(),
        ));
        p.add_layer(Box::new(l11_semantic_dedup::SemanticDedupLayer::new()));
        p.add_layer(Box::new(l12_deadline_aware::DeadlineAwareLayer::new()));
        p.add_layer(Box::new(l13_priority_queue::PriorityQueueLayer::new()));
        p.add_layer(Box::new(l14_compression::CompressionLayer::new()));
        p.add_layer(Box::new(l15_fallback_chain::FallbackChainLayer::new()));
        p.add_layer(Box::new(
            l16_circuit_breaker_v2::CircuitBreakerV2Layer::new(),
        ));
        p.add_layer(Box::new(
            l17_council_optimizer::CouncilOptimizerLayer::new(),
        ));
        p
    }
}

#[derive(Debug, Clone)]
pub struct LLMOptimizationRequest {
    pub char_name: String,
    pub phase: String,
    pub prompt: String,
    pub priority: u8,
    pub budget_tokens: u32,
    pub context: Vec<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Default)]
pub struct LLMOptimizationResult {
    pub prompt_rewritten: String,
    pub tokens_saved: u32,
    pub cache_hit: bool,
    pub priority: u8,
    pub deadline_ms: u64,
    pub should_dispatch: bool,
    pub fallback_chain: Vec<String>,
}

pub trait OptimizationLayer: Send + Sync {
    fn name(&self) -> &'static str;
    fn apply(&self, request: &mut LLMOptimizationRequest, result: &mut LLMOptimizationResult);
}
