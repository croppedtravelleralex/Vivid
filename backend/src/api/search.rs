use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::ApiState;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    q: Option<String>,
}

pub fn routes() -> axum::Router<ApiState> {
    axum::Router::new().route("/", axum::routing::get(search_handler))
}

/// GET /api/v1/search?q=...
async fn search_handler(
    State(state): State<ApiState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Value>, StatusCode> {
    let query = params.q.unwrap_or_default().to_lowercase();
    let world = state.engine.world.read().await;

    // Search characters by name
    let mut char_results = vec![];
    for entity_ref in world.characters.iter() {
        if let Some(name) = entity_ref.get::<&String>() {
            if name.to_lowercase().contains(&query) || query.is_empty() {
                if let Some(id) = entity_ref.get::<&Uuid>() {
                    char_results.push(json!({ "id": *id, "name": &**name }));
                }
            }
        }
    }

    // Search locations by name
    let mut loc_results = vec![];
    for loc in &world.locations {
        if loc.name.to_lowercase().contains(&query) || query.is_empty() {
            loc_results.push(json!({ "id": loc.id, "name": loc.name }));
        }
    }

    Ok(Json(json!({
        "status": "ok",
        "data": {
            "query": query,
            "characters": char_results,
            "locations": loc_results,
            "total": char_results.len() + loc_results.len(),
        }
    })))
}
