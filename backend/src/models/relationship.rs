use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Relationship graph primitives
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipNode {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEdge {
    pub trust: f64,
    pub familiarity: f64,
    pub sentiment: f64,
    pub label: String,
}

/// Definition of a relationship from one character to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipDef {
    pub target_id: Uuid,
    pub label: String,
    pub trust: i8,
    pub familiarity: i8,
    pub sentiment: i8,
    pub description: String,
}

/// Summary returned by the API for a character's relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipSummary {
    pub target_id: Uuid,
    pub target_name: String,
    pub trust: f64,
    pub familiarity: f64,
    pub sentiment: f64,
    pub label: String,
}

// ---------------------------------------------------------------------------
// WebSocket diff types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipDelta {
    pub trust: f64,
    pub familiarity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipValues {
    pub trust: f64,
    pub familiarity: f64,
    pub sentiment: f64,
}
