use std::collections::HashMap;
use std::path::Path;

use tracing::info;
use uuid::Uuid;

use crate::models::character::{
    upgrade_v1_to_v3, CharacterCardV1, CharacterCardV3,
};
use crate::models::world::{CharacterState, EnvironmentState, LocationNode, WorldState};

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
        let date_str = json.get("startDate")?.as_str()?;
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

        if let Some(locations) = json.get("locations").and_then(|v| v.as_array()) {
            for loc_val in locations {
                let name = loc_val
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let node = LocationNode {
                    id: Uuid::now_v7(),
                    name,
                    description: loc_val
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    category: crate::models::world::LocationCategory::Other,
                    condition: loc_val
                        .get("condition")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0),
                    max_occupancy: loc_val
                        .get("maxOccupancy")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(10) as u32,
                    tags: loc_val
                        .get("tags")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default(),
                    resources: HashMap::new(),
                    position: None,
                };
                self.locations.push(node);
            }
        }
        Ok(())
    }

    fn load_environment(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        if let Some(temp) = json.get("initialTemperature").and_then(|v| v.as_f64()) {
            self.environment.temperature = temp;
        }
        if let Some(rate) = json
            .get("globalCoolingRate")
            .and_then(|r| r.get("perDay"))
            .and_then(|v| v.as_f64())
        {
            self.environment.global_cooling_rate = rate;
        }

        info!(
            "环境初始化: temp={}, cooling_rate={}",
            self.environment.temperature, self.environment.global_cooling_rate
        );
        Ok(())
    }

    fn load_events(&mut self, _path: impl AsRef<Path>) -> Result<(), String> {
        info!("事件数据加载 (占位)");
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
