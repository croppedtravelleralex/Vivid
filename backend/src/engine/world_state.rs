use std::collections::HashMap;
use std::path::Path;

use tracing::info;
use uuid::Uuid;

use crate::models::character::{
    upgrade_v1_to_v3, CharacterCardV1, CharacterCardV1Flat, CharacterCardV3,
};
use crate::models::world::{
    CharacterState, EnvironmentState, LocationCategory, LocationNode, ResourceStock, WorldState,
};

impl WorldState {
    /// Load initial world state from JSON data files.
    pub fn load_from_files(
        characters_dir: impl AsRef<Path>,
        locations_file: impl AsRef<Path>,
        environment_file: impl AsRef<Path>,
        events_file: impl AsRef<Path>,
        seed: u64,
    ) -> Result<Self, Vec<String>> {
        let mut errors = vec![];

        let start_date = Self::load_start_date(&environment_file).unwrap_or_else(|| {
            chrono::NaiveDateTime::parse_from_str("2025-12-03T18:00:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap()
        });

        let mut world = WorldState::new(start_date, seed);

        if let Err(e) = world.load_characters(characters_dir) {
            errors.push(format!("Failed to load characters: {}", e));
        }
        if let Err(e) = world.load_locations(locations_file) {
            errors.push(format!("Failed to load locations: {}", e));
        }
        if let Err(e) = world.load_environment(environment_file) {
            errors.push(format!("Failed to load environment: {}", e));
        }
        if let Err(e) = world.load_events(events_file) {
            errors.push(format!("Failed to load events: {}", e));
        }

        if errors.is_empty() {
            info!(
                "World loaded: {} chars, {} locations",
                world.character_count(),
                world.location_count()
            );
            Ok(world)
        } else {
            Err(errors)
        }
    }

    fn load_start_date(path: impl AsRef<Path>) -> Option<chrono::NaiveDateTime> {
        let content = std::fs::read_to_string(path.as_ref()).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        // Try initial_date (snake_case) or startDate (camelCase)
        let date_str = json
            .get("initial_date")
            .or_else(|| json.get("startDate"))
            .and_then(|v| v.as_str())?;
        chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S").ok()
    }

    fn load_characters(&mut self, dir: impl AsRef<Path>) -> Result<(), String> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Err(format!("Characters directory not found: {:?}", dir));
        }

        for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

            if let Ok(v3) = serde_json::from_str::<CharacterCardV3>(&content) {
                let id = Uuid::now_v7();
                let state = CharacterState::from(v3.runtime.state.clone());
                self.characters.spawn((id, v3.identity.name.clone(), state));
                info!("加载角色 (v3): {}", v3.identity.name);
            } else if let Ok(flat) = serde_json::from_str::<CharacterCardV1Flat>(&content) {
                let name = flat.name.clone();
                let v3 = flat.upgrade();
                let id = Uuid::now_v7();
                let state = CharacterState::from(v3.runtime.state.clone());
                self.characters.spawn((id, v3.identity.name.clone(), state));
                info!("加载角色 (v1-flat->v3): {}", name);
            } else if let Ok(v1) = serde_json::from_str::<CharacterCardV1>(&content) {
                let name = v1.base.name.clone();
                let v3 = upgrade_v1_to_v3(v1);
                let id = Uuid::now_v7();
                let state = CharacterState::from(v3.runtime.state.clone());
                self.characters.spawn((id, v3.identity.name.clone(), state));
                info!("加载角色 (v1->v3 迁移): {}", name);
            } else {
                return Err(format!("Cannot parse character file: {:?}", path));
            }
        }
        Ok(())
    }

    fn load_locations(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        // The JSON may be a flat array or wrapped in {"locations": [...]}
        let locations = json
            .as_array()
            .or_else(|| json.get("locations").and_then(|v| v.as_array()))
            .ok_or_else(|| "locations.json must be an array or have a 'locations' key".to_string())?;

        for loc_val in locations {
            let _id_str = loc_val.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
            let name = loc_val.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

            // Parse resources: {"food": 200, "water": 100} → HashMap<String, ResourceStock>
            let resources: HashMap<String, ResourceStock> = loc_val
                .get("resources")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| {
                            let amount = v.as_f64().unwrap_or(0.0);
                            (
                                k.clone(),
                                ResourceStock {
                                    current: amount,
                                    max: amount,
                                    unit: "units".into(),
                                    daily_consumption: 0.0,
                                },
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Parse category string
            let cat_str = loc_val.get("category").and_then(|v| v.as_str()).unwrap_or("other");
            let category = match cat_str {
                "shelter" => LocationCategory::Shelter,
                "resource" => LocationCategory::Resource,
                "danger" => LocationCategory::Danger,
                "transit" => LocationCategory::Transit,
                "residential" => LocationCategory::Residential,
                "commercial" => LocationCategory::Commercial,
                "medical" => LocationCategory::Medical,
                "industrial" => LocationCategory::Industrial,
                _ => LocationCategory::Other,
            };

            // Generate a stable UUID from the id string
            let id = Uuid::now_v7();

            let tags: Vec<String> = loc_val
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let node = LocationNode {
                id,
                name,
                description: loc_val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                category,
                condition: 1.0,
                max_occupancy: 20,
                tags,
                resources,
                position: None,
            };
            self.locations.push(node);
        }
        Ok(())
    }

    fn load_environment(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        self.environment.temperature = json
            .get("initial_temperature")
            .or_else(|| json.get("initialTemperature"))
            .and_then(|v| v.as_f64())
            .unwrap_or(-5.0);

        // cooling rate: try temperature_drop_per_year or globalCoolingRate.perDay
        self.environment.global_cooling_rate = json
            .get("temperature_drop_per_year")
            .and_then(|v| v.as_f64())
            .or_else(|| {
                json.get("globalCoolingRate")
                    .and_then(|r| r.get("perDay"))
                    .and_then(|v| v.as_f64())
            })
            .unwrap_or(-0.08);

        self.environment.global_cooling_rate /= 365.0; // per_year → per_day

        info!(
            "环境初始化: temp={}, cooling_per_day={}",
            self.environment.temperature, self.environment.global_cooling_rate
        );
        Ok(())
    }

    fn load_events(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        use crate::models::event::*;
        use crate::models::timeline::SimTime;

        let start_datetime = self.timeline.time.datetime;

        // Load seed events
        if let Some(seed_events) = json.get("seed_events").and_then(|v| v.as_array()) {
            for ev in seed_events {
                let title = ev.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let description = ev.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let severity_str = ev.get("severity").and_then(|v| v.as_str()).unwrap_or("normal");
                let trigger_tick = ev.get("trigger_tick").and_then(|v| v.as_u64()).unwrap_or(0);

                // Convert tick offset to SimTime
                let trigger_datetime = start_datetime + chrono::Duration::minutes(5 * trigger_tick as i64);
                let trigger_time = SimTime::new(trigger_datetime);

                let severity = match severity_str {
                    "critical" => Severity::Critical,
                    "major" => Severity::Major,
                    "minor" => Severity::Minor,
                    _ => Severity::Normal,
                };

                let scheduled = ScheduledEvent {
                    id: Uuid::now_v7(),
                    trigger_time,
                    event_type: EventType::ExternalStimulus(title.clone()),
                    title,
                    description,
                    participants: vec![],
                    priority: EventPriority::Normal,
                    severity,
                    source: EventSource::Scheduled,
                    one_shot: true,
                    fired: false,
                };
                self.event_system.queue.push(scheduled);
            }
        }

        // Load conditional triggers
        if let Some(triggers) = json.get("conditional_triggers").and_then(|v| v.as_array()) {
            for t in triggers {
                let trigger_id = t.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let event_type_str = t.get("event_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let event_title = t.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();

                // Build TriggerCondition from simple condition object
                let condition = t.get("condition").and_then(|c| {
                    let _cond_type = c.get("type")?.as_str()?;
                    let resource = c.get("resource")?.as_str()?;
                    let threshold = c.get("threshold")?.as_f64()?;
                    Some(TriggerCondition {
                        and: Some(vec![TriggerClause {
                            variable: format!("resources.{}", resource),
                            op: "lt".into(),
                            value: serde_json::json!(threshold),
                        }]),
                        or: None,
                        variable: None,
                        op: None,
                        value: None,
                    })
                }).unwrap_or(TriggerCondition {
                    and: None, or: None,
                    variable: Some("always".into()),
                    op: Some("eq".into()),
                    value: Some(serde_json::json!(true)),
                });

                let trigger = ConditionTrigger {
                    id: trigger_id,
                    condition,
                    effect: TriggerEffect {
                        effect_type: event_type_str,
                        description: event_title,
                        affected_locations: None,
                        message: "Conditional trigger activated".into(),
                    },
                    one_shot: true,
                };
                self.event_system.queue.condition_triggers.push(trigger);
            }
        }

        info!(
            "事件已加载: {} 种子, {} 条件",
            json.get("seed_events").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
            json.get("conditional_triggers").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Character state access (via hecs query)
    // -----------------------------------------------------------------------

    pub fn get_character_count(&self) -> usize {
        self.characters.iter().count()
    }
}

// ---------------------------------------------------------------------------
// Checkpoint serialization helpers
// ---------------------------------------------------------------------------

impl WorldState {
    /// Serialize to bincode bytes.
    pub fn to_bincode(&self) -> Result<Vec<u8>, String> {
        let snapshot = WorldStateSnapshot {
            tick: self.timeline.tick,
            datetime: self.timeline.time.datetime,
            environment: self.environment.clone(),
            locations: self.locations.clone(),
            resources: self.resources.clone(),
            character_count: self.character_count(),
        };
        bincode::serialize(&snapshot).map_err(|e| e.to_string())
    }

    /// Deserialize from bincode bytes (partial restore).
    pub fn from_bincode(data: &[u8], seed: u64) -> Result<Self, String> {
        let snapshot: WorldStateSnapshot = bincode::deserialize(data).map_err(|e| e.to_string())?;
        let mut world = WorldState::new(snapshot.datetime, seed);
        world.timeline.tick = snapshot.tick;
        world.environment = snapshot.environment;
        world.locations = snapshot.locations;
        world.resources = snapshot.resources;
        Ok(world)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WorldStateSnapshot {
    tick: u64,
    datetime: chrono::NaiveDateTime,
    environment: EnvironmentState,
    locations: Vec<LocationNode>,
    resources: HashMap<String, f64>,
    character_count: usize,
}
