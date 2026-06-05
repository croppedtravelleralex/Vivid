use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use governor::{DefaultDirectRateLimiter, RateLimiter};
use reqwest::Client;
use tokio::sync::Semaphore;
use tracing::{info, warn};
use uuid::Uuid;

use crate::engine::{LLMError, LLMMessage, LLMResponse, TokenUsage};

// ---------------------------------------------------------------------------
// Circuit breaker
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum CircuitBreakerState {
    Closed,
    Open { until: chrono::DateTime<chrono::Utc> },
    HalfOpen,
}

#[derive(Debug, Clone)]
struct CircuitState {
    state: CircuitBreakerState,
    failure_count: u64,
    last_failure: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for CircuitState {
    fn default() -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            last_failure: None,
        }
    }
}

// ---------------------------------------------------------------------------
// LLMGateway
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct LLMGateway {
    pub(crate) api_key: String,
    pub(crate) base_url: String,
    pub(crate) model: String,
    pub(crate) semaphore: Arc<Semaphore>,
    pub(crate) client: Client,
    pub(crate) max_concurrent: usize,
    pub(crate) timeout_seconds: u64,
    rate_limiter: DefaultDirectRateLimiter,
    circuit: tokio::sync::Mutex<CircuitState>,
    consecutive_failures_before_open: u64,
    circuit_open_seconds: u64,
}

impl LLMGateway {
    pub fn new(
        api_key: String,
        base_url: String,
        model: String,
        max_concurrent: usize,
        timeout_seconds: u64,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        // Allow ~60 RPM
        let rate_limiter = RateLimiter::direct(governor::Quota::per_second(
            std::num::NonZeroU32::new(60).unwrap(),
        ));

        Self {
            api_key,
            base_url,
            model,
            semaphore,
            client,
            max_concurrent,
            timeout_seconds,
            rate_limiter,
            circuit: tokio::sync::Mutex::new(CircuitState::default()),
            consecutive_failures_before_open: 5,
            circuit_open_seconds: 30,
        }
    }

    /// Call the LLM API with circuit breaker, rate limiter, and semaphore.
    pub async fn call_llm(
        &self,
        messages: &[LLMMessage],
    ) -> Result<LLMResponse, LLMError> {
        // Check circuit breaker
        {
            let mut circuit = self.circuit.lock().await;
            match &circuit.state {
                CircuitBreakerState::Open { until } if chrono::Utc::now() < *until => {
                    return Err(LLMError::CircuitBreakerOpen);
                }
                // Window expired: transition to HalfOpen for probe
                CircuitBreakerState::Open { .. } => {
                    circuit.state = CircuitBreakerState::HalfOpen;
                    // Fall through to allow ONE probe request
                }
                CircuitBreakerState::HalfOpen => {
                    // Second concurrent request while probe is in flight
                    return Err(LLMError::CircuitBreakerOpen);
                }
                CircuitBreakerState::Closed => {}
            }
        }

        // Rate limit — ignore errors (best effort)
        let _ = self.rate_limiter.until_ready().await;

        // Acquire semaphore permit
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| LLMError::ApiError("Semaphore closed".into()))?;

        let start = std::time::Instant::now();

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 1024,
        });

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match response {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let status = resp.status();
                    self.record_failure().await;

                    return if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        Err(LLMError::RateLimited)
                    } else {
                        Err(LLMError::ApiError(format!("HTTP {}", status)))
                    };
                }

                // Reset circuit on success
                {
                    let mut circuit = self.circuit.lock().await;
                    circuit.failure_count = 0;
                    circuit.state = CircuitBreakerState::Closed;
                }

                // Parse response
                let json: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| LLMError::ParseError(e.to_string()))?;

                let content = json["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                let usage = json["usage"].clone();
                let token_usage = TokenUsage {
                    prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                    completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
                    total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as u32,
                };

                info!(latency_ms, tokens = token_usage.total_tokens, "LLM 调用成功");

                Ok(LLMResponse {
                    content,
                    usage: token_usage,
                    latency_ms,
                })
            }
            Err(e) => {
                self.record_failure().await;
                Err(LLMError::ApiError(e.to_string()))
            }
        }
    }

    async fn record_failure(&self) {
        let mut circuit = self.circuit.lock().await;
        circuit.failure_count += 1;
        circuit.last_failure = Some(Utc::now());
        match &circuit.state {
            CircuitBreakerState::HalfOpen => {
                // Probe failed — back to Open
                circuit.state = CircuitBreakerState::Open {
                    until: Utc::now()
                        + chrono::Duration::seconds(self.circuit_open_seconds as i64),
                };
            }
            _ => {
                if circuit.failure_count >= self.consecutive_failures_before_open {
                    circuit.state = CircuitBreakerState::Open {
                        until: Utc::now()
                            + chrono::Duration::seconds(self.circuit_open_seconds as i64),
                    };
                    warn!(
                        "Circuit breaker OPEN after {} failures",
                        circuit.failure_count
                    );
                }
            }
        }
    }

    /// Batch decisions for multiple characters.
    pub async fn batch_decide(
        &self,
        snapshots: Vec<(Uuid, CharContextSnapshot)>,
    ) -> Vec<Decision> {
        let mut results = Vec::new();

        for (char_id, snapshot) in snapshots {
            let messages = vec![
                LLMMessage {
                    role: "system".into(),
                    content: snapshot.system_prompt.clone(),
                },
                LLMMessage {
                    role: "user".into(),
                    content: snapshot.perceive_prompt.clone(),
                },
            ];

            match self.call_llm(&messages).await {
                Ok(response) => {
                    results.push(Decision {
                        char_id,
                        action: serde_json::from_str(&response.content).unwrap_or_default(),
                        thought: response.content.clone(),
                        confidence: 0.7,
                    });
                }
                Err(e) => {
                    warn!(%char_id, error = %e, "LLM decision failed");
                }
            }
        }

        results
    }
}

// ---------------------------------------------------------------------------
// Character context snapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CharContextSnapshot {
    pub system_prompt: String,
    pub perceive_prompt: String,
    pub think_prompt: Option<String>,
    pub act_prompt: Option<String>,
}

impl CharContextSnapshot {
    pub fn build(
        char_name: &str,
        traits: &str,
        state: &str,
        location: &str,
        visible_chars: &[String],
        recent_memories: &[String],
    ) -> Self {
        let system_prompt = format!(
            "你是 {}。\n性格：{}\n当前状态：{}\n位置：{}\n\n\
            用中文第一人称回应。输出严格 JSON 格式。",
            char_name, traits, state, location
        );

        let perceive_prompt = format!(
            "你现在位于 {}。周围有：{}。\n最近记忆：{}\n\
            输出 JSON: {{ \"noticed\": [...], \"priority_level\": \"high|normal|low\" }}",
            location,
            visible_chars.join(", "),
            recent_memories.join("; "),
        );

        Self {
            system_prompt,
            perceive_prompt,
            think_prompt: None,
            act_prompt: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Decision result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Decision {
    pub char_id: Uuid,
    pub action: serde_json::Value,
    pub thought: String,
    pub confidence: f64,
}
