use std::collections::HashMap;
use rand::Rng;
use tracing::info;

/// Detects "collisions" between character states that could generate emergent events.
///
/// Emergent events arise from subsystem intersections rather than direct triggers:
/// resource scarcity × low trust = resource conflict, opinion divergence over time = schism, etc.
pub struct EmergentDetector {
    pub sensitivity: f64,
    pub last_check_tick: u64,
    pub collision_log: Vec<Collision>,
}

/// A detected collision between two or more subsystem states.
#[derive(Debug, Clone)]
pub struct Collision {
    pub tick: u64,
    pub collision_type: CollisionType,
    pub participants: Vec<String>,
    pub intensity: f64,
    pub description: String,
}

/// Categories of subsystem collisions that can generate emergent events.
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionType {
    /// Two or more characters want the same limited resource.
    ResourceContention,
    /// Character beliefs or opinions are drifting apart.
    OpinionDivergence,
    /// Trust between characters dropped below a critical threshold.
    TrustViolation,
    /// Asymmetric knowledge — one character knows something another doesn't.
    SecretExposure,
    /// Stress, fear, or panic spreading between characters.
    EmotionalContagion,
    /// A subgroup splitting off from the main group.
    GroupFormation,
}

impl EmergentDetector {
    /// Create a new detector with the given sensitivity (0.0–1.0).
    pub fn new(sensitivity: f64) -> Self {
        Self {
            sensitivity: sensitivity.clamp(0.0, 1.0),
            last_check_tick: 0,
            collision_log: Vec::new(),
        }
    }

    /// Run all collision checks against the current world snapshot.
    ///
    /// Returns a list of newly detected collisions.
    pub fn detect(
        &mut self,
        tick: u64,
        world_snapshot: &WorldSnapshot,
    ) -> Vec<Collision> {
        self.last_check_tick = tick;
        let mut collisions = Vec::new();

        collisions.extend(self.check_resource_contention(world_snapshot));
        collisions.extend(self.check_opinion_divergence(world_snapshot));
        collisions.extend(self.check_trust_violations(world_snapshot));
        collisions.extend(self.check_emotional_contagion(world_snapshot));

        // Log significant collisions
        for c in &collisions {
            if c.intensity > self.sensitivity {
                info!(
                    "emergent: {:?} collision detected (intensity={:.3}, participants={:?})",
                    c.collision_type, c.intensity, c.participants
                );
            }
        }

        self.collision_log.extend(collisions.clone());
        // Keep log bounded
        if self.collision_log.len() > 1000 {
            self.collision_log.drain(0..self.collision_log.len() - 1000);
        }

        collisions
            .into_iter()
            .filter(|c| c.intensity >= self.sensitivity)
            .collect()
    }

    /// Check for resource contention at locations where supply is critically low.
    fn check_resource_contention(&self, world: &WorldSnapshot) -> Vec<Collision> {
        let mut results = Vec::new();

        for (location_id, location) in &world.locations {
            let critical_resources: Vec<&String> = location
                .resources
                .iter()
                .filter(|(_, stock)| **stock < 0.2)
                .map(|(name, _)| name)
                .collect();

            if critical_resources.is_empty() {
                continue;
            }

            let occupants = world.get_characters_at(location_id);
            if occupants.len() < 2 {
                continue;
            }

            for i in 0..occupants.len() {
                for j in (i + 1)..occupants.len() {
                    let trust = world
                        .relationships
                        .get(&occupants[i])
                        .and_then(|rels| rels.get(&occupants[j]))
                        .copied()
                        .unwrap_or(0.5);

                    if trust < 0.3 {
                        let intensity = (1.0 - trust) * 0.8;
                        if intensity >= self.sensitivity {
                            results.push(Collision {
                                tick: self.last_check_tick,
                                collision_type: CollisionType::ResourceContention,
                                participants: vec![
                                    occupants[i].clone(),
                                    occupants[j].clone(),
                                ],
                                intensity,
                                description: format!(
                                    "Resource contention over '{}' at {} between {} and {}",
                                    critical_resources[0], location_id,
                                    occupants[i], occupants[j]
                                ),
                            });
                        }
                    }
                }
            }
        }

        results
    }

    /// Check for opinion divergence — characters whose trust has been degrading.
    fn check_opinion_divergence(&self, world: &WorldSnapshot) -> Vec<Collision> {
        let mut results = Vec::new();

        for (char_a, rels) in &world.relationships {
            for (char_b, trust) in rels {
                if char_a >= char_b {
                    continue; // avoid duplicate pairs
                }
                // If trust has dropped significantly from baseline
                if *trust < 0.25 {
                    let intensity = (0.25 - trust) * 1.2;
                    if intensity >= self.sensitivity {
                        results.push(Collision {
                            tick: self.last_check_tick,
                            collision_type: CollisionType::OpinionDivergence,
                            participants: vec![char_a.clone(), char_b.clone()],
                            intensity,
                            description: format!(
                                "Opinion divergence between {} and {} (trust={:.2})",
                                char_a, char_b, trust
                            ),
                        });
                    }
                }
            }
        }

        results
    }

    /// Check for trust violations — trust dropped below a critical floor.
    fn check_trust_violations(&self, world: &WorldSnapshot) -> Vec<Collision> {
        let mut results = Vec::new();

        // Low-trust pairs from the relationship map
        for (char_a, rels) in &world.relationships {
            for (char_b, trust) in rels {
                if char_a >= char_b {
                    continue;
                }
                if *trust < 0.15 {
                    let intensity = (0.15 - trust) * 1.5;
                    if intensity >= self.sensitivity {
                        results.push(Collision {
                            tick: self.last_check_tick,
                            collision_type: CollisionType::TrustViolation,
                            participants: vec![char_a.clone(), char_b.clone()],
                            intensity,
                            description: format!(
                                "Trust violation between {} and {} (trust={:.2})",
                                char_a, char_b, trust
                            ),
                        });
                    }
                }
            }
        }

        results
    }

    /// Check for emotional contagion — widespread stress in the group.
    fn check_emotional_contagion(&self, world: &WorldSnapshot) -> Vec<Collision> {
        let mut results = Vec::new();

        let char_count = world.characters.len() as f64;
        if char_count < 2.0 {
            return results;
        }

        let avg_stress: f64 = world.characters.values().map(|c| c.stress).sum::<f64>() / char_count;

        if avg_stress > 0.7 {
            let high_stress_chars: Vec<&String> = world
                .characters
                .iter()
                .filter(|(_, c)| c.stress > 0.7)
                .map(|(name, _)| name)
                .collect();

            if high_stress_chars.len() >= 2 {
                let intensity = (avg_stress - 0.7) * 1.5;
                if intensity >= self.sensitivity {
                    results.push(Collision {
                        tick: self.last_check_tick,
                        collision_type: CollisionType::EmotionalContagion,
                        participants: high_stress_chars.into_iter().cloned().collect(),
                        intensity,
                        description: format!(
                            "Emotional contagion: group-wide stress at {:.2}",
                            avg_stress
                        ),
                    });
                }
            }
        }

        results
    }
}

impl Default for EmergentDetector {
    fn default() -> Self {
        Self::new(0.3)
    }
}

// ---------------------------------------------------------------------------
// WorldSnapshot
// ---------------------------------------------------------------------------

/// A lightweight snapshot of world state used for emergent detection.
///
/// This is a stand-in for the full `WorldState` — integrate by populating
/// from the real world state during the simulation tick.
#[derive(Debug, Clone)]
pub struct WorldSnapshot {
    /// Character name → CharacterData
    pub characters: HashMap<String, CharacterData>,
    /// Character name → (other character name → trust value 0.0–1.0)
    pub relationships: HashMap<String, HashMap<String, f64>>,
    /// Location name → LocationData
    pub locations: HashMap<String, LocationData>,
}

/// Per-character data relevant to emergent detection.
#[derive(Debug, Clone)]
pub struct CharacterData {
    pub stress: f64,
    pub location: String,
}

/// Per-location data relevant to emergent detection.
#[derive(Debug, Clone)]
pub struct LocationData {
    /// Resource name → fraction remaining (0.0–1.0)
    pub resources: HashMap<String, f64>,
    /// Character names currently at this location
    pub occupants: Vec<String>,
}

impl WorldSnapshot {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            relationships: HashMap::new(),
            locations: HashMap::new(),
        }
    }

    /// Get the names of characters at a given location.
    pub fn get_characters_at(&self, location_id: &str) -> Vec<String> {
        self.locations
            .get(location_id)
            .map(|loc| loc.occupants.clone())
            .unwrap_or_default()
    }
}

impl Default for WorldSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// TraitEventMatrix
// ---------------------------------------------------------------------------

/// A CK3-style trait–event matrix that maps (context, trait-combination) pairs
/// to weighted event templates.
///
/// The same world state produces different events depending on character
/// personality traits and the current context tag.
pub struct TraitEventMatrix {
    pub entries: HashMap<(String, String), Vec<WeightedEvent>>,
}

/// An event template paired with its base probability weight.
#[derive(Debug, Clone)]
pub struct WeightedEvent {
    pub template: EventTemplate,
    pub base_weight: f64,
}

/// Template for an event before instantiation with specific participants.
#[derive(Debug, Clone)]
pub struct EventTemplate {
    pub event_type: String,
    pub description_template: String,
}

impl TraitEventMatrix {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register a (context, trait_combo) → event mapping.
    pub fn register(
        &mut self,
        context_tag: &str,
        trait_combo: &str,
        template: EventTemplate,
        base_weight: f64,
    ) {
        let key = (context_tag.to_string(), trait_combo.to_string());
        self.entries
            .entry(key)
            .or_default()
            .push(WeightedEvent { template, base_weight });
    }

    /// Roll for events matching the given character traits and context.
    ///
    /// Returns a list of instantiated events whose weight roll succeeded.
    pub fn roll(
        &self,
        context_tag: &str,
        trait_combos: &[String],
        character_name: &str,
    ) -> Vec<InstantiatedEvent> {
        let mut results = Vec::new();
        let mut rng = rand::thread_rng();

        for combo in trait_combos {
            let key = (context_tag.to_string(), combo.clone());
            if let Some(entries) = self.entries.get(&key) {
                for weighted in entries {
                    if rng.gen::<f64>() < weighted.base_weight {
                        results.push(InstantiatedEvent {
                            event_type: weighted.template.event_type.clone(),
                            description: weighted
                                .template
                                .description_template
                                .replace("{character}", character_name),
                            participants: vec![character_name.to_string()],
                        });
                    }
                }
            }
        }

        results
    }
}

impl Default for TraitEventMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// An event that has been rolled and assigned specific participants.
#[derive(Debug, Clone)]
pub struct InstantiatedEvent {
    pub event_type: String,
    pub description: String,
    pub participants: Vec<String>,
}

// ---------------------------------------------------------------------------
// CrisisSandpile (SOC stress overflow)
// ---------------------------------------------------------------------------

/// Per-character stress accumulator modelled as a sandpile.
///
/// Stress accumulates each tick. When it crosses the threshold it "avalanches",
/// producing an emergent event and propagating stress to social neighbours.
pub struct CrisisSandpile {
    pub stress: HashMap<String, f64>,
    pub threshold: f64,
}

impl CrisisSandpile {
    pub fn new(threshold: f64) -> Self {
        Self {
            stress: HashMap::new(),
            threshold,
        }
    }

    /// Add stress to a character. Returns events if the threshold is exceeded.
    pub fn add_stress(
        &mut self,
        char_name: &str,
        amount: f64,
        relationships: &HashMap<String, HashMap<String, f64>>,
    ) -> Vec<String> {
        let stress = self.stress.entry(char_name.to_string()).or_insert(0.0);
        *stress += amount;

        let mut events = Vec::new();

        if *stress >= self.threshold {
            *stress = 0.0;
            info!(
                "emergent: stress overflow for {} (threshold={})",
                char_name, self.threshold
            );
            events.push(char_name.to_string());

            // Propagate stress to neighbours
            if let Some(neighbours) = relationships.get(char_name) {
                for (neighbour, _trust) in neighbours {
                    let ns = self.stress.entry(neighbour.clone()).or_insert(0.0);
                    *ns += 0.5;
                    if *ns >= self.threshold {
                        *ns = 0.0;
                        events.push(neighbour.clone());
                        info!(
                            "emergent: stress cascade from {} to {}",
                            char_name, neighbour
                        );
                    }
                }
            }
        }

        events
    }

    /// Get the current stress of a character.
    pub fn get_stress(&self, char_name: &str) -> f64 {
        self.stress.get(char_name).copied().unwrap_or(0.0)
    }

    /// Apply natural decay to all stress levels.
    pub fn tick_decay(&mut self, rate: f64) {
        for stress in self.stress.values_mut() {
            *stress = (*stress - rate).max(0.0);
        }
    }
}

impl Default for CrisisSandpile {
    fn default() -> Self {
        Self::new(4.0)
    }
}
