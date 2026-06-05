use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::storage::checkpoint::CheckpointManager;
use crate::ApiState;

fn internal_error(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status":"error","message":msg})))
}
fn not_found_err(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::NOT_FOUND, Json(json!({"status":"error","message":msg})))
}

pub fn routes() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/save", axum::routing::post(save_checkpoint))
        .route("/load", axum::routing::post(load_checkpoint))
        .route("/list", axum::routing::get(list_checkpoints))
}

/// POST /api/v1/checkpoint/save
async fn save_checkpoint(
    State(state): State<ApiState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let tag = body.get("tag").and_then(|v| v.as_str()).unwrap_or("manual_save");
    let world = state.engine.world.read().await;
    let manager = CheckpointManager::new("data/checkpoints", 50)
        .map_err(|e| internal_error(&e))?;
    let meta = manager.save_snapshot(&world, tag)
        .map_err(|e| internal_error(&e))?;
    Ok(Json(json!({ "status": "ok", "data": meta })))
}

/// POST /api/v1/checkpoint/load
async fn load_checkpoint(
    State(state): State<ApiState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let tag = body.get("tag").and_then(|v| v.as_str()).unwrap_or("manual_save");
    let manager = CheckpointManager::new("data/checkpoints", 50)
        .map_err(|e| internal_error(&e))?;
    let seed = { state.engine.config.read().await.random_seed };
    let loaded_world = manager.load_snapshot(tag, seed)
        .map_err(|_| not_found_err("Checkpoint not found"))?;
    *state.engine.world.write().await = loaded_world;
    Ok(Json(json!({ "status": "ok", "message": format!("已加载检查点: {}", tag) })))
}

/// GET /api/v1/checkpoint/list
async fn list_checkpoints(
    State(_state): State<ApiState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let manager = CheckpointManager::new("data/checkpoints", 50)
        .map_err(|e| internal_error(&e))?;
    let list = manager.list_checkpoints()
        .map_err(|e| internal_error(&e))?;
    Ok(Json(json!({ "status": "ok", "data": list })))
}
