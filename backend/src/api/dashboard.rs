use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

use crate::engine::EngineState;
use crate::ApiState;

pub fn routes() -> axum::Router<ApiState> {
    axum::Router::new().route("/summary", axum::routing::get(dashboard_summary))
}

/// GET /api/v1/dashboard/summary
async fn dashboard_summary(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let engine = state.engine;
    let world = engine.world.read().await;

    let engine_guard = engine.state.lock().unwrap();
    let state_str = match &*engine_guard {
        EngineState::Paused => "paused",
        EngineState::Running { .. } => "running",
        EngineState::Stopped => "stopped",
    };
    let speed = match &*engine_guard {
        EngineState::Running { speed, .. } => speed.label(),
        _ => "paused",
    };
    drop(engine_guard);

    Ok(Json(json!({
        "status": "ok",
        "data": {
            "tick": world.timeline.tick,
            "time": world.timeline.time.to_string(),
            "speed": speed,
            "state": state_str,
            "characters": world.character_count(),
            "locations": world.location_count(),
            "temperature": world.environment.temperature,
            "weather": world.environment.weather,
            "season": world.environment.season,
            "events_triggered": engine.stats.events_triggered.load(std::sync::atomic::Ordering::Relaxed),
            "total_llm_calls": engine.stats.total_llm_calls.load(std::sync::atomic::Ordering::Relaxed),
        }
    })))
}
