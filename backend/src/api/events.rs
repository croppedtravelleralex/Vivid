use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::event::{EventPriority, EventSource, EventType, ScheduledEvent, Severity};
use crate::ApiState;

pub fn routes() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/", axum::routing::post(create_event))
        .route("/{id}", axum::routing::get(get_event))
}

/// POST /api/v1/events — manually inject an event
async fn create_event(
    State(state): State<ApiState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let current_time = {
        let world = state.engine.world.read().await;
        world.timeline.time
    };

    let event_id = Uuid::now_v7();
    let title = body
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Manual Event")
        .to_string();
    let description = body
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let event = ScheduledEvent {
        id: event_id,
        trigger_time: current_time,
        event_type: EventType::Custom(title.clone()),
        title,
        description,
        participants: vec![],
        priority: EventPriority::Normal,
        severity: Severity::Normal,
        source: EventSource::Manual,
        one_shot: true,
        fired: false,
    };

    {
        let mut world = state.engine.world.write().await;
        world.event_system.queue.push(event);
    }

    Ok(Json(json!({
        "status": "ok",
        "data": { "event_id": event_id }
    })))
}

/// GET /api/v1/events/:id — lookup event detail
async fn get_event(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let world = state.engine.world.read().await;
    for hist in &world.timeline.events {
        if hist.id == id {
            return Ok(Json(json!({
                "status": "ok",
                "data": hist,
            })));
        }
    }
    Err((StatusCode::NOT_FOUND, Json(json!({"status":"error","message":"not found"}))))
}
