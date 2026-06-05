use std::path::Path;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Top-level config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub schema_version: String,
    pub engine: EngineConfig,
    pub llm: LlmConfig,
    pub checkpoint: CheckpointConfig,
    pub event_log: EventLogConfig,
    pub memory: MemoryConfig,
    pub social: SocialConfig,
    pub chaos: ChaosConfig,
    pub economics: EconomicsConfig,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub detailed_tick_minutes: u64,
    pub fastforward_tick_hours: u64,
    pub idle_threshold: u8,
    pub random_seed: u64,
    #[serde(default = "default_max_concurrent_llm")]
    pub max_concurrent_llm: usize,
    #[serde(default = "default_llm_timeout_seconds")]
    pub llm_timeout_seconds: u64,
    #[serde(default = "default_checkpoint_interval")]
    pub checkpoint_interval: u64,
}

fn default_max_concurrent_llm() -> usize { 10 }
fn default_llm_timeout_seconds() -> u64 { 30 }
fn default_checkpoint_interval() -> u64 { 100 }

// ---------------------------------------------------------------------------
// LLM
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_concurrent: usize,
    pub timeout_seconds: u64,
}

// ---------------------------------------------------------------------------
// Checkpoint
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct CheckpointConfig {
    pub interval: u64,
    pub max_auto: usize,
    pub max_delta: u64,
}

// ---------------------------------------------------------------------------
// Event log
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct EventLogConfig {
    pub max_entries: usize,
    pub auto_archive_after: u64,
}

// ---------------------------------------------------------------------------
// Memory
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryConfig {
    pub max_per_character: usize,
    pub auto_summarize_after: usize,
}

// ---------------------------------------------------------------------------
// Social models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct SocialConfig {
    pub degroot_weight_base: f64,
    pub friedkin_johnsen_lambda: f64,
    pub hk_epsilon: f64,
    pub bicchieri_empirical_threshold: f64,
    pub bicchieri_normative_threshold: f64,
    pub gossip_chance: f64,
    pub panic_threshold_mean: f64,
    pub schelling_threshold: f64,
}

// ---------------------------------------------------------------------------
// Chaos models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ChaosConfig {
    pub logistic_r_default: f64,
    pub sandpile_threshold: f64,
    pub power_law_alpha: f64,
    pub langton_lambda: f64,
}

// ---------------------------------------------------------------------------
// Economics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct EconomicsConfig {
    pub loss_aversion_lambda: f64,
    pub prospect_alpha: f64,
    pub hyperbolic_discount_k: f64,
    pub production_alpha_forage: f64,
    pub production_alpha_construction: f64,
    pub production_alpha_scout: f64,
    pub ostrom_sustainability_threshold: f64,
    pub cooperation_critical_scarcity: f64,
}

// ---------------------------------------------------------------------------
// Config loading + validation
// ---------------------------------------------------------------------------

impl Config {
    /// Load and validate config from a YAML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Vec<String>> {
        let content =
            std::fs::read_to_string(path.as_ref()).map_err(|e| vec![e.to_string()])?;
        let config: Config =
            serde_yaml::from_str(&content).map_err(|e| vec![e.to_string()])?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = vec![];
        if self.social.hk_epsilon <= 0.0 || self.social.hk_epsilon > 1.0 {
            errors.push("social.hk_epsilon must be in (0, 1]".into());
        }
        if self.chaos.logistic_r_default <= 0.0 || self.chaos.logistic_r_default > 4.0 {
            errors.push("chaos.logistic_r must be in (0, 4]".into());
        }
        if self.economics.loss_aversion_lambda < 1.0 {
            errors.push("economics.loss_aversion_lambda must be >= 1.0".into());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
