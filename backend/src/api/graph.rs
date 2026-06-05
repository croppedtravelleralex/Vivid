use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use petgraph::visit::EdgeRef;
use serde_json::{json, Value};

use crate::ApiState;

pub fn routes() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/relationships", axum::routing::get(graph_relationships))
        .route("/locations", axum::routing::get(graph_locations))
}

/// GET /api/v1/graph/relationships
async fn graph_relationships(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let world = state.engine.world.read().await;
    let mut nodes = vec![];
    let mut edges = vec![];

    for node_idx in world.relationships.node_indices() {
        if let Some(node) = world.relationships.node_weight(node_idx) {
            nodes.push(json!({
                "id": node.id,
                "name": node.name,
                "node_type": "character",
                "group": 0,
            }));
        }
    }

    for edge_idx in world.relationships.edge_indices() {
        if let Some((source, target)) = world.relationships.edge_endpoints(edge_idx) {
            if let Some(edge) = world.relationships.edge_weight(edge_idx) {
                edges.push(json!({
                    "source": world.relationships.node_weight(source).map(|n| n.id),
                    "target": world.relationships.node_weight(target).map(|n| n.id),
                    "weight": edge.trust,
                    "label": &edge.label,
                }));
            }
        }
    }

    Ok(Json(json!({ "status": "ok", "data": { "nodes": nodes, "edges": edges } })))
}

/// GET /api/v1/graph/locations
async fn graph_locations(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let world = state.engine.world.read().await;
    let nodes: Vec<Value> = world
        .locations
        .iter()
        .map(|loc| {
            json!({
                "id": loc.id,
                "name": loc.name,
                "node_type": "location",
                "group": 1,
            })
        })
        .collect();

    let edges: Vec<Value> = world
        .location_graph
        .edge_references()
        .map(|edge| {
            json!({
                "source": world.location_graph.node_weight(edge.source()),
                "target": world.location_graph.node_weight(edge.target()),
                "weight": edge.weight().distance_km,
                "label": &edge.weight().description,
            })
        })
        .collect();

    Ok(Json(json!({ "status": "ok", "data": { "nodes": nodes, "edges": edges } })))
}
