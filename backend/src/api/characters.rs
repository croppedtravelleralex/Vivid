use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::ApiState;

pub fn routes() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/", axum::routing::get(character_list))
        .route("/{id}", axum::routing::get(character_detail))
        .route("/{id}/memory", axum::routing::get(character_memory))
        .route("/{id}/relationships", axum::routing::get(character_relationships))
}

/// GET /api/v1/characters
async fn character_list(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let world = state.engine.world.read().await;
    let mut chars = vec![];

    for entity_ref in world.characters.iter() {
        if let Some(id) = entity_ref.get::<&Uuid>() {
            if let Some(name) = entity_ref.get::<&String>() {
                if let Some(cs) = entity_ref.get::<&crate::models::world::CharacterState>() {
                    chars.push(json!({
                        "id": *id,
                        "name": &**name,
                        "hp": cs.hp,
                        "max_hp": cs.max_hp,
                        "hunger": cs.hunger,
                        "mental": cs.mental,
                        "location": cs.location,
                        "is_idle": cs.is_idle,
                    }));
                }
            }
        }
    }
    Ok(Json(json!({ "status": "ok", "data": chars })))
}

/// GET /api/v1/characters/:id
async fn character_detail(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let world = state.engine.world.read().await;
    for entity_ref in world.characters.iter() {
        if let Some(char_id) = entity_ref.get::<&Uuid>() {
            if *char_id == id {
                let name = &**entity_ref.get::<&String>().unwrap();
                let cs = &*entity_ref.get::<&crate::models::world::CharacterState>().unwrap();
                return Ok(Json(json!({
                    "status": "ok",
                    "data": {
                        "id": *char_id,
                        "name": name,
                        "state": {
                            "hp": cs.hp,
                            "max_hp": cs.max_hp,
                            "hunger": cs.hunger,
                            "warmth": cs.warmth,
                            "fatigue": cs.fatigue,
                            "mental": cs.mental,
                            "stress": cs.stress,
                            "location": cs.location,
                            "is_idle": cs.is_idle,
                        },
                    }
                })));
            }
        }
    }
    Err(StatusCode::NOT_FOUND)
}

/// GET /api/v1/characters/:id/memory
async fn character_memory(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let world = state.engine.world.read().await;
    let found = world.characters.iter().any(|er| {
        er.get::<&Uuid>().map(|r| *r) == Some(id)
    });
    if found {
        Ok(Json(json!({ "status": "ok", "data": [] })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// GET /api/v1/characters/:id/relationships
async fn character_relationships(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let world = state.engine.world.read().await;
    let found = world.characters.iter().any(|er| {
        er.get::<&Uuid>().map(|r| *r) == Some(id)
    });
    if found {
        Ok(Json(json!({ "status": "ok", "data": [] })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
