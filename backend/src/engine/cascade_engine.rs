use std::collections::{HashMap, VecDeque};
use rand::Rng;
use tracing::info;

/// Propagates event effects through the social/relationship network,
/// producing second- and third-order consequences.
///
/// When an event occurs, it ripples outward through the relationship graph,
/// with magnitude decaying at each hop. The cascade engine processes these
/// ripples one tick at a time, allowing delayed reactions.
pub struct CascadeEngine {
    pub max_depth: u32,
    pub decay_factor: f64,
    pub pending: VecDeque<CascadeEvent>,
    pub history: Vec<CascadeEvent>,
    pub next_id: u64,
}

/// A single cascade event propagating through the social graph.
#[derive(Debug, Clone)]
pub struct CascadeEvent {
    pub id: u64,
    pub origin_event_id: String,
    pub depth: u32,
    pub affected_char: String,
    pub effect_type: String,
    pub magnitude: f64,
    pub probability: f64,
    pub propagated: bool,
}

impl CascadeEngine {
    pub fn new(max_depth: u32, decay_factor: f64) -> Self {
        Self {
            max_depth,
            decay_factor: decay_factor.clamp(0.0, 1.0),
            pending: VecDeque::new(),
            history: Vec::new(),
            next_id: 0,
        }
    }

    /// Inject a new event into the cascade system.
    ///
    /// Relationships is a map from character name → (neighbour → trust value).
    /// The initial cascade event will be processed on the next `tick()` call,
    /// at which point it propagates to neighbours at depth 1.
    pub fn inject(
        &mut self,
        origin: &str,
        char_name: &str,
        effect_type: &str,
        magnitude: f64,
        _relationships: &HashMap<String, HashMap<String, f64>>,
    ) {
        let id = self.next_id;
        self.next_id += 1;

        let initial = CascadeEvent {
            id,
            origin_event_id: origin.to_string(),
            depth: 0,
            affected_char: char_name.to_string(),
            effect_type: effect_type.to_string(),
            magnitude,
            probability: 1.0,
            propagated: false,
        };
        self.pending.push_back(initial);

        info!(
            "cascade: injected event '{}' affecting {} (magnitude={:.3})",
            origin, char_name, magnitude
        );
    }

    /// Process one tick of pending cascades.
    ///
    /// Returns all cascade events that fired this tick.
    pub fn tick(
        &mut self,
        relationships: &HashMap<String, HashMap<String, f64>>,
    ) -> Vec<CascadeEvent> {
        let mut fired = Vec::new();
        let to_process: Vec<CascadeEvent> = self.pending.drain(..).collect();

        for mut event in to_process {
            if event.propagated {
                // Already handled; archive it
                self.history.push(event);
                continue;
            }

            // Roll the probability check
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() < event.probability {
                event.propagated = true;
                fired.push(event.clone());

                // Propagate to next hop if under max depth
                if event.depth < self.max_depth {
                    self.propagate_to_neighbors(&event, relationships);
                }

                self.history.push(event);
            } else {
                // Probability failed — drop this branch
                info!(
                    "cascade: branch dropped (depth={}, type={}, prob={:.3})",
                    event.depth, event.effect_type, event.probability
                );
            }
        }

        if !fired.is_empty() {
            info!("cascade: {} events fired this tick", fired.len());
        }

        // Keep history bounded
        if self.history.len() > 10_000 {
            self.history.drain(0..self.history.len() - 10_000);
        }

        fired
    }

    /// Create next-hop cascade events for all neighbours of the given event's target.
    fn propagate_to_neighbors(
        &mut self,
        event: &CascadeEvent,
        relationships: &HashMap<String, HashMap<String, f64>>,
    ) {
        let next_depth = event.depth + 1;
        let next_probability = event.probability * self.decay_factor;
        let next_magnitude = event.magnitude * self.decay_factor;

        if let Some(neighbours) = relationships.get(&event.affected_char) {
            for (neighbour, trust) in neighbours {
                let adjusted_prob = next_probability * trust;
                let adjusted_mag = next_magnitude * trust;

                if adjusted_prob < 0.01 || adjusted_mag < 0.001 {
                    continue;
                }

                let id = self.next_id;
                self.next_id += 1;

                self.pending.push_back(CascadeEvent {
                    id,
                    origin_event_id: event.origin_event_id.clone(),
                    depth: next_depth,
                    affected_char: neighbour.clone(),
                    effect_type: event.effect_type.clone(),
                    magnitude: adjusted_mag,
                    probability: adjusted_prob,
                    propagated: false,
                });
            }
        }
    }
}

impl Default for CascadeEngine {
    fn default() -> Self {
        Self::new(5, 0.5)
    }
}

// ---------------------------------------------------------------------------
// SpatialPropagation
// ---------------------------------------------------------------------------

/// Propagates events through physical space using distance-based attenuation.
///
/// Events spread outward from an origin location up to `max_radius_km`,
/// with probability and severity decreasing with distance.
pub struct SpatialPropagation {
    pub max_radius_km: f64,
    pub attenuation: f64,
}

/// An event that has propagated to a nearby location.
#[derive(Debug, Clone)]
pub struct PropagatedLocationEvent {
    pub location_id: String,
    pub severity_scale: f64,
    pub delay_ticks: u64,
}

impl SpatialPropagation {
    pub fn new(max_radius_km: f64, attenuation: f64) -> Self {
        Self {
            max_radius_km,
            attenuation: attenuation.clamp(0.0, 1.0),
        }
    }

    /// Propagate an event to nearby locations given a map of
    /// location → distance from origin.
    ///
    /// Returns a list of propagated events with severity scaled by distance.
    pub fn propagate(
        &self,
        distances: &HashMap<String, f64>,
    ) -> Vec<PropagatedLocationEvent> {
        let mut results = Vec::new();
        let mut rng = rand::thread_rng();

        for (location_id, distance_km) in distances {
            if *distance_km <= 0.0 || *distance_km > self.max_radius_km {
                continue;
            }

            let prob = (1.0 - distance_km / self.max_radius_km) * self.attenuation;
            if rng.gen::<f64>() > prob {
                continue;
            }

            results.push(PropagatedLocationEvent {
                location_id: location_id.clone(),
                severity_scale: 1.0 - (distance_km / self.max_radius_km) * 0.7,
                delay_ticks: (distance_km * 2.0) as u64,
            });
        }

        results
    }
}

impl Default for SpatialPropagation {
    fn default() -> Self {
        Self::new(10.0, 0.8)
    }
}

// ---------------------------------------------------------------------------
// SocialPropagation
// ---------------------------------------------------------------------------

/// Propagates social events (gossip, news, reactions) through the
/// relationship graph using BFS with hop-based attenuation.
pub struct SocialPropagation {
    pub max_hops: usize,
    pub gossip_chance: f64,
}

/// A social ripple — a reaction to an event at distance `hops`.
#[derive(Debug, Clone)]
pub struct SocialRipple {
    pub target: String,
    pub trust_change: f64,
    pub sentiment_change: f64,
    pub delay_ticks: u64,
}

impl SocialPropagation {
    pub fn new(max_hops: usize, gossip_chance: f64) -> Self {
        Self {
            max_hops,
            gossip_chance: gossip_chance.clamp(0.0, 1.0),
        }
    }

    /// Propagate a social event through the relationship graph.
    ///
    /// `relationships` maps character name → (neighbour → trust value).
    /// Returns a list of social ripples affecting characters at various hops.
    pub fn propagate(
        &self,
        origin: &str,
        relationships: &HashMap<String, HashMap<String, f64>>,
    ) -> Vec<SocialRipple> {
        let mut ripples = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = VecDeque::new();
        let mut rng = rand::thread_rng();

        visited.insert(origin.to_string());
        queue.push_back((origin.to_string(), 0usize));

        while let Some((current, hops)) = queue.pop_front() {
            if hops >= self.max_hops {
                continue;
            }

            if let Some(neighbours) = relationships.get(&current) {
                for (neighbour, trust) in neighbours {
                    if visited.contains(neighbour) {
                        continue;
                    }
                    visited.insert(neighbour.clone());

                    let prob = self.gossip_chance / (hops + 1) as f64;
                    if rng.gen::<f64>() > prob {
                        continue;
                    }

                    let trust_delta = -0.05 * (1.0 - trust) / (hops + 1) as f64;

                    ripples.push(SocialRipple {
                        target: neighbour.clone(),
                        trust_change: trust_delta,
                        sentiment_change: trust_delta * 0.5,
                        delay_ticks: (hops + 1) as u64,
                    });

                    queue.push_back((neighbour.clone(), hops + 1));
                }
            }
        }

        ripples
    }
}

impl Default for SocialPropagation {
    fn default() -> Self {
        Self::new(5, 0.6)
    }
}

// ---------------------------------------------------------------------------
// ResourceCascade
// ---------------------------------------------------------------------------

/// Models cascading resource failures through a dependency graph.
///
/// For example: fuel shortage → electricity drop → water pump failure → hygiene crisis.
pub struct ResourceCascade {
    /// Resource → list of (dependent_resource, conversion_factor) pairs.
    pub dependencies: HashMap<String, Vec<(String, f64)>>,
}

/// A cascade event triggered by a resource change.
#[derive(Debug, Clone)]
pub struct ResourceCascadeEvent {
    pub resource: String,
    pub impact: f64,
    pub severity: String,
}

impl ResourceCascade {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    /// Register a dependency: `from` → `to` with the given conversion factor.
    pub fn add_dependency(&mut self, from: &str, to: &str, conversion: f64) {
        self.dependencies
            .entry(from.to_string())
            .or_default()
            .push((to.to_string(), conversion));
    }

    /// Propagate a resource change through the dependency graph.
    ///
    /// `resource_deltas` is a mutable map of current resource changes (fractional, -1.0 to 1.0).
    /// The method returns all cascade events generated.
    pub fn propagate(
        &self,
        origin_resource: &str,
        resource_deltas: &mut HashMap<String, f64>,
    ) -> Vec<ResourceCascadeEvent> {
        let mut queue = VecDeque::new();
        queue.push_back(origin_resource.to_string());
        let mut events = Vec::new();

        while let Some(resource) = queue.pop_front() {
            let delta = resource_deltas.get(&resource).copied().unwrap_or(0.0);
            if delta.abs() < 0.001 {
                continue;
            }

            if let Some(dependents) = self.dependencies.get(&resource) {
                for (dep, conversion) in dependents {
                    if delta.abs() > 0.15 {
                        let impact = delta * conversion;
                        let current_dep_delta = resource_deltas.get(dep).copied().unwrap_or(0.0);
                        resource_deltas.insert(dep.clone(), current_dep_delta + impact);

                        let severity = if impact.abs() > 0.3 {
                            "major"
                        } else {
                            "minor"
                        };

                        info!(
                            "cascade: resource '{}' change ({:.3}) -> '{}' impact {:.3} ({})",
                            resource, delta, dep, impact, severity
                        );

                        events.push(ResourceCascadeEvent {
                            resource: dep.clone(),
                            impact,
                            severity: severity.to_string(),
                        });

                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        events
    }
}

impl Default for ResourceCascade {
    fn default() -> Self {
        Self::new()
    }
}
