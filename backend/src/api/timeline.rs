use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::ApiState;

pub fn routes() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/", axum::routing::get(timeline_index))
        .route("/events", axum::routing::get(timeline_events))
}

/// GET /api/v1/timeline
async fn timeline_index(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let world = state.engine.world.read().await;
    Ok(Json(json!({
        "status": "ok",
        "data": {
            "tick": world.timeline.tick,
            "time": world.timeline.time.to_string(),
            "event_count": world.timeline.events.len(),
        }
    })))
}

/// GET /api/v1/timeline/events
async fn timeline_events(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let world = state.engine.world.read().await;
    Ok(Json(json!({
        "status": "ok",
        "data": {
            "events": world.timeline.events,
            "total": world.timeline.events.len() as u64,
        }
    })))
}
