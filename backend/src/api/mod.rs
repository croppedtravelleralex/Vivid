mod characters;
mod checkpoint;
mod dashboard;
mod events;
mod graph;
mod search;
mod simulation;
mod tags;
mod timeline;
mod world;
mod ws;

use axum::Router;

/// Build the full API router with all M1 endpoints under /api/v1/.
pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .nest("/api/v1/world", world::routes())
        .nest("/api/v1/characters", characters::routes())
        .nest("/api/v1/simulation", simulation::routes())
        .nest("/api/v1/graph", graph::routes())
        .nest("/api/v1/timeline", timeline::routes())
        .nest("/api/v1/checkpoint", checkpoint::routes())
        .nest("/api/v1/events", events::routes())
        .nest("/api/v1/tags", tags::routes())
        .nest("/api/v1/dashboard", dashboard::routes())
        .nest("/api/v1/search", search::routes())
        .route("/api/v1/ws", axum::routing::get(ws::ws_handler))
}
