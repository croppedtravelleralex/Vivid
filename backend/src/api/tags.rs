use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::ApiState;

pub fn routes() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/heatmap", axum::routing::get(tags_heatmap))
        .route("/threads", axum::routing::get(tags_threads))
}

async fn tags_heatmap(State(state): State<ApiState>) -> Json<Value> {
    let index = state.engine.tag_index.read().await;
    let active_count: usize = index.active_threads.values().map(|v| v.len()).sum();
    let archived_count = index.archived_threads.len();
    let event_count = index.event_tags.len();
    let char_count = index.character_tags.len();

    Json(json!({
        "status": "ok",
        "data": {
            "tagged_events": event_count,
            "tagged_characters": char_count,
            "active_threads": active_count,
            "archived_threads": archived_count,
        }
    }))
}

async fn tags_threads(State(state): State<ApiState>) -> Json<Value> {
    let index = state.engine.tag_index.read().await;
    let active: Vec<_> = index.active_threads.values().flat_map(|v| v.iter()).collect();
    Json(json!({
        "status": "ok",
        "data": {
            "active": active,
            "archived": &index.archived_threads,
        }
    }))
}
