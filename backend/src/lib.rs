pub mod api;
pub mod config;
pub mod engine;
pub mod llm;
pub mod models;
pub mod storage;
pub mod telemetry;

use std::sync::Arc;

pub use crate::engine::SimulationEngine;
pub use crate::models::world::WorldState;

/// Shared application state for Axum routers.
#[derive(Clone)]
pub struct ApiState {
    pub engine: Arc<SimulationEngine>,
}
