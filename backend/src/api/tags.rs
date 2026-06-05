use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::ApiState;

pub fn routes() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/heatmap", axum::routing::get(tags_heatmap))
        .route("/threads", axum::routing::get(tags_threads))
}

/// GET /api/v1/tags/heatmap
async fn tags_heatmap(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let index = state.engine.tag_index.read().await;
    Ok(Json(json!({
        "status": "ok",
        "data": &index.heatmap,
    })))
}

/// GET /api/v1/tags/threads
async fn tags_threads(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let index = state.engine.tag_index.read().await;
    Ok(Json(json!({
        "status": "ok",
        "data": &index.threads,
    })))
}
