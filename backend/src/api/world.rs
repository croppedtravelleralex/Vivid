use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

use crate::ApiState;

pub fn routes() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/", axum::routing::get(world_summary))
        .route("/environment", axum::routing::get(world_environment))
        .route("/locations", axum::routing::get(world_locations))
}

/// GET /api/v1/world
async fn world_summary(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let world = state.engine.world.read().await;
    Ok(Json(json!({
        "status": "ok",
        "data": {
            "tick": world.timeline.tick,
            "time": world.timeline.time.to_string(),
            "character_count": world.character_count(),
            "location_count": world.location_count(),
            "temperature": world.environment.temperature,
            "weather": world.environment.weather,
            "season": world.environment.season,
        }
    })))
}

/// GET /api/v1/world/environment
async fn world_environment(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let world = state.engine.world.read().await;
    Ok(Json(json!({
        "status": "ok",
        "data": {
            "temperature": world.environment.temperature,
            "weather": world.environment.weather,
            "season": world.environment.season,
            "daylight": world.environment.daylight,
        }
    })))
}

/// GET /api/v1/world/locations
async fn world_locations(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let world = state.engine.world.read().await;
    let locations: Vec<Value> = world
        .locations
        .iter()
        .map(|loc| {
            json!({
                "id": loc.id,
                "name": loc.name,
                "description": loc.description,
                "condition": loc.condition,
                "tags": loc.tags,
            })
        })
        .collect();
    Ok(Json(json!({ "status": "ok", "data": locations })))
}
