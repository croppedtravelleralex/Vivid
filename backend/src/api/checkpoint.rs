use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::storage::checkpoint::CheckpointManager;
use crate::ApiState;

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
) -> Result<Json<Value>, StatusCode> {
    let tag = body
        .get("tag")
        .and_then(|v| v.as_str())
        .unwrap_or("manual_save");

    let world = state.engine.world.read().await;
    let manager = CheckpointManager::new("data/checkpoints", 50).map_err(|_| {
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let meta = manager.save_snapshot(&world, tag).map_err(|_| {
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(json!({
        "status": "ok",
        "data": meta,
    })))
}

/// POST /api/v1/checkpoint/load
async fn load_checkpoint(
    State(state): State<ApiState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let tag = body
        .get("tag")
        .and_then(|v| v.as_str())
        .unwrap_or("manual_save");

    let manager = CheckpointManager::new("data/checkpoints", 50).map_err(|_| {
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let seed = {
        let config = state.engine.config.read().await;
        config.random_seed
    };

    let world = manager.load_snapshot(tag, seed).map_err(|_| {
        StatusCode::NOT_FOUND
    })?;

    // Replace the engine's world with the loaded one
    let mut engine_world = state.engine.world.write().await;
    *engine_world = world;

    Ok(Json(json!({
        "status": "ok",
        "message": format!("已加载检查点: {}", tag),
    })))
}

/// GET /api/v1/checkpoint/list
async fn list_checkpoints(
    State(_state): State<ApiState>,
) -> Result<Json<Value>, StatusCode> {
    let manager = CheckpointManager::new("data/checkpoints", 50).map_err(|_| {
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let list = manager.list_checkpoints().map_err(|_| {
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(json!({
        "status": "ok",
        "data": list,
    })))
}
