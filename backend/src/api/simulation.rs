use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

use crate::engine::{EngineState, SimSpeed};
use crate::ApiState;

pub fn routes() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/start", axum::routing::post(start_simulation))
        .route("/pause", axum::routing::post(pause_simulation))
        .route("/speed", axum::routing::post(set_speed))
        .route("/step", axum::routing::post(step_simulation))
        .route("/stop", axum::routing::post(stop_simulation))
        .route("/status", axum::routing::get(simulation_status))
        .route("/stats", axum::routing::get(simulation_stats))
}

/// POST /api/v1/simulation/start
async fn start_simulation(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let mut engine_state = state.engine.state.lock().unwrap();
    *engine_state = EngineState::Running {
        speed: SimSpeed::Detailed,
        tick_count: state.engine.current_tick(),
    };
    Ok(Json(json!({ "status": "ok", "message": "模拟已开始" })))
}

/// POST /api/v1/simulation/pause
async fn pause_simulation(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let mut engine_state = state.engine.state.lock().unwrap();
    *engine_state = EngineState::Paused;
    Ok(Json(json!({ "status": "ok", "message": "模拟已暂停" })))
}

/// POST /api/v1/simulation/speed
async fn set_speed(
    State(state): State<ApiState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let speed_str = body
        .get("speed")
        .and_then(|v| v.as_str())
        .unwrap_or("detailed");
    let speed = match speed_str {
        "fast_forward" | "fastforward" => SimSpeed::FastForward,
        "paused" => SimSpeed::Paused,
        _ => SimSpeed::Detailed,
    };

    let mut engine_state = state.engine.state.lock().unwrap();
    *engine_state = EngineState::Running {
        speed,
        tick_count: state.engine.current_tick(),
    };

    Ok(Json(json!({ "status": "ok", "speed": speed })))
}

/// POST /api/v1/simulation/step
async fn step_simulation(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let engine = state.engine;
    engine.is_stepping.store(true, std::sync::atomic::Ordering::Relaxed);
    // Set to Detailed temporarily for one tick
    {
        let mut engine_state = engine.state.lock().unwrap();
        *engine_state = EngineState::Running {
            speed: SimSpeed::Detailed,
            tick_count: engine.current_tick(),
        };
    }
    engine.detailed_tick().await;
    // Pause after step
    {
        let mut engine_state = engine.state.lock().unwrap();
        *engine_state = EngineState::Paused;
    }
    engine.is_stepping.store(false, std::sync::atomic::Ordering::Relaxed);
    Ok(Json(json!({
        "status": "ok",
        "data": {
            "tick": engine.current_tick(),
            "speed": "detailed",
        }
    })))
}

/// POST /api/v1/simulation/stop
async fn stop_simulation(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let mut engine_state = state.engine.state.lock().unwrap();
    *engine_state = EngineState::Stopped;
    Ok(Json(json!({ "status": "ok", "saved": true })))
}

/// GET /api/v1/simulation/status
async fn simulation_status(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let engine = state.engine;
    let engine_state = engine.state.lock().unwrap();
    let state_str = match &*engine_state {
        EngineState::Paused => "paused",
        EngineState::Running { .. } => "running",
        EngineState::Stopped => "stopped",
    };
    let speed = match &*engine_state {
        EngineState::Running { speed, .. } => speed.label(),
        _ => "paused",
    };
    Ok(Json(json!({
        "status": "ok",
        "data": {
            "state": state_str,
            "speed": speed,
            "tick": engine.current_tick(),
        }
    })))
}

/// GET /api/v1/simulation/stats
async fn simulation_stats(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let stats = state.engine.snapshot_stats();
    Ok(Json(json!({
        "status": "ok",
        "data": stats,
    })))
}
