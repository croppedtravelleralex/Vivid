use std::collections::HashMap;
use rand::Rng;
use tracing::info;

/// Probability branching tree for narrative outcomes.
///
/// Each node in the tree has multiple weighted branches.
/// The system rolls weighted random selection, respecting dynamic conditions
/// evaluated against the current world state.
pub struct ProbabilityTree {
    pub nodes: Vec<ProbabilityNode>,
    pub current_node_id: Option<usize>,
}

/// A single node in the probability tree, representing a narrative branch point.
#[derive(Debug, Clone)]
pub struct ProbabilityNode {
    pub id: usize,
    pub description: String,
    pub branches: Vec<ProbabilityBranch>,
    pub chosen_branch: Option<usize>,
}

/// A weighted branch from one node to another, with optional conditions.
#[derive(Debug, Clone)]
pub struct ProbabilityBranch {
    pub target_node_id: usize,
    pub description: String,
    pub probability: f64,
    pub conditions: Vec<BranchCondition>,
}

/// A condition that must be satisfied for a branch to be eligible.
#[derive(Debug, Clone)]
pub struct BranchCondition {
    pub variable: String,
    pub op: String,
    pub value: f64,
}

impl ProbabilityTree {
    /// Create a new empty probability tree.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            current_node_id: None,
        }
    }

    /// Add a new node and return its index.
    pub fn add_node(&mut self, description: &str) -> usize {
        let id = self.nodes.len();
        self.nodes.push(ProbabilityNode {
            id,
            description: description.to_string(),
            branches: Vec::new(),
            chosen_branch: None,
        });
        id
    }

    /// Add a weighted branch from one node to another.
    ///
    /// `probability` should be in the range 0.0–1.0.
    pub fn add_branch(
        &mut self,
        from: usize,
        to: usize,
        description: &str,
        probability: f64,
    ) {
        if from >= self.nodes.len() || to >= self.nodes.len() {
            tracing::warn!(
                "probability_tree: invalid branch {} -> {} (node count = {})",
                from,
                to,
                self.nodes.len()
            );
            return;
        }
        self.nodes[from]
            .branches
            .push(ProbabilityBranch {
                target_node_id: to,
                description: description.to_string(),
                probability: probability.clamp(0.0, 1.0),
                conditions: Vec::new(),
            });
    }

    /// Add a condition to the most recently added branch of a node.
    pub fn add_condition(
        &mut self,
        node_id: usize,
        variable: &str,
        op: &str,
        value: f64,
    ) {
        if node_id >= self.nodes.len() {
            return;
        }
        if let Some(branch) = self.nodes[node_id].branches.last_mut() {
            branch.conditions.push(BranchCondition {
                variable: variable.to_string(),
                op: op.to_string(),
                value,
            });
        }
    }

    /// Choose a branch from the given node based on weighted probabilities
    /// and world state conditions.
    ///
    /// Returns the `target_node_id` of the chosen branch, or `None` if no
    /// branch is eligible.
    pub fn choose_branch(
        &mut self,
        node_id: usize,
        world_state: &HashMap<String, f64>,
    ) -> Option<usize> {
        if node_id >= self.nodes.len() {
            return None;
        }

        // Collect eligible branch data as owned values to avoid borrow conflicts
        let eligible: Vec<(usize, String, f64)> = self.nodes[node_id]
            .branches
            .iter()
            .filter(|b| self.evaluate_conditions(&b.conditions, world_state))
            .map(|b| (b.target_node_id, b.description.clone(), b.probability))
            .collect();

        if eligible.is_empty() {
            info!(
                "probability_tree: no eligible branches at node {} ({})",
                node_id, self.nodes[node_id].description
            );
            return None;
        }

        // Calculate total effective weight
        let total_weight: f64 = eligible.iter().map(|(_, _, prob)| prob).sum();
        if total_weight <= 0.0 {
            return None;
        }

        // Roll
        let mut rng = rand::thread_rng();
        let roll = rng.gen::<f64>() * total_weight;
        let mut cumulative = 0.0;

        for (target_id, desc, prob) in &eligible {
            cumulative += prob;
            if roll <= cumulative {
                self.nodes[node_id].chosen_branch = Some(*target_id);
                self.current_node_id = Some(*target_id);

                info!(
                    "probability_tree: node {} -> {} (branch: '{}', prob={:.3})",
                    node_id, target_id, desc, prob
                );

                return Some(*target_id);
            }
        }

        // Fallback: return last eligible branch
        if let Some((target_id, _desc, _prob)) = eligible.last() {
            self.nodes[node_id].chosen_branch = Some(*target_id);
            self.current_node_id = Some(*target_id);
            return Some(*target_id);
        }

        None
    }

    /// Evaluate a set of conditions against the current world state.
    ///
    /// All conditions must pass (logical AND).
    pub fn evaluate_conditions(
        &self,
        conditions: &[BranchCondition],
        state: &HashMap<String, f64>,
    ) -> bool {
        if conditions.is_empty() {
            return true;
        }

        conditions.iter().all(|cond| {
            let actual = state.get(&cond.variable).copied().unwrap_or(0.0);
            let pass = match cond.op.as_str() {
                "lt" => actual < cond.value,
                "gt" => actual > cond.value,
                "eq" => (actual - cond.value).abs() < 0.001,
                "lte" => actual <= cond.value,
                "gte" => actual >= cond.value,
                "neq" => (actual - cond.value).abs() >= 0.001,
                _ => {
                    tracing::warn!(
                        "probability_tree: unknown operator '{}' in condition",
                        cond.op
                    );
                    false
                }
            };
            if !pass {
                tracing::debug!(
                    "probability_tree: condition failed {} {} {} (actual={})",
                    cond.variable,
                    cond.op,
                    cond.value,
                    actual
                );
            }
            pass
        })
    }

    /// Reset the chosen branch on a node, allowing re-roll.
    pub fn reset_node(&mut self, node_id: usize) {
        if node_id < self.nodes.len() {
            self.nodes[node_id].chosen_branch = None;
        }
    }

    /// Get the chain of chosen branches from root to current node.
    pub fn chosen_path(&self) -> Vec<usize> {
        let mut path = Vec::new();
        let mut current = 0; // start from root

        loop {
            if current >= self.nodes.len() {
                break;
            }
            path.push(current);
            match self.nodes[current].chosen_branch {
                Some(next) => current = next,
                None => break,
            }
        }

        path
    }
}

impl Default for ProbabilityTree {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// BranchControl
// ---------------------------------------------------------------------------

/// Manages branching factor: prunes low-probability branches and caps child count.
///
/// Used to keep the probability tree from growing too wide.
pub struct BranchControl {
    pub max_children: usize,
    pub significance: f64,
}

impl BranchControl {
    pub fn new(max_children: usize, significance: f64) -> Self {
        Self {
            max_children,
            significance,
        }
    }

    /// Prune branches on a node: remove low-weight branches and cap at max_children.
    pub fn prune(&self, branches: &mut Vec<ProbabilityBranch>, state: &HashMap<String, f64>) {
        // Remove branches that don't meet the significance threshold
        branches.retain(|b| {
            let effective = self.effective_probability(b, state);
            effective >= self.significance
        });

        // Cap at max_children, keeping the highest-weight branches
        if branches.len() > self.max_children {
            branches.sort_by(|a, b| {
                let pa = self.effective_probability(a, state);
                let pb = self.effective_probability(b, state);
                pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal)
            });
            branches.truncate(self.max_children);
        }
    }

    /// Compute the effective probability of a branch, accounting for conditions.
    fn effective_probability(
        &self,
        branch: &ProbabilityBranch,
        state: &HashMap<String, f64>,
    ) -> f64 {
        // If conditions fail, probability becomes 0
        let conditions_met = branch.conditions.iter().all(|cond| {
            let actual = state.get(&cond.variable).copied().unwrap_or(0.0);
            match cond.op.as_str() {
                "lt" => actual < cond.value,
                "gt" => actual > cond.value,
                "eq" => (actual - cond.value).abs() < 0.001,
                _ => false,
            }
        });

        if !conditions_met {
            0.0
        } else {
            branch.probability
        }
    }
}

impl Default for BranchControl {
    fn default() -> Self {
        Self::new(5, 0.05)
    }
}

// ---------------------------------------------------------------------------
// TellMeMore (delayed concretisation)
// ---------------------------------------------------------------------------

/// An event seed that is placed into the world but not concretised until investigated.
///
/// This implements the "Tell Me More" pattern: the seed exists as a vague
/// hint (sound, smell, object), and its nature is only rolled when a
/// character actively investigates it.
pub struct EventSeed {
    pub id: String,
    pub location: String,
    pub seed_type: SeedType,
    pub generated: Option<ConcreteEvent>,
}

/// Types of vague event seeds.
#[derive(Debug, Clone, PartialEq)]
pub enum SeedType {
    SoundNearby,
    SuspiciousObject,
    UnusualSmell,
    StrangeFootprints,
}

/// A concrete event generated from a seed upon investigation.
#[derive(Debug, Clone)]
pub struct ConcreteEvent {
    pub event_type: String,
    pub description: String,
    pub is_threat: bool,
    pub participants: Vec<String>,
}

impl EventSeed {
    /// Create a new un-investigated seed.
    pub fn new(id: &str, location: &str, seed_type: SeedType) -> Self {
        Self {
            id: id.to_string(),
            location: location.to_string(),
            seed_type,
            generated: None,
        }
    }

    /// Investigate the seed, generating concrete details on first call.
    ///
    /// Subsequent calls return the cached generated event.
    pub fn investigate(
        &mut self,
        investigator: &str,
        neuroticism: f64,
    ) -> ConcreteEvent {
        if self.generated.is_none() {
            self.generated = Some(self.roll_concrete(investigator, neuroticism));
            info!(
                "probability_tree: seed '{}' investigated by {} -> {}",
                self.id,
                investigator,
                self.generated.as_ref().unwrap().event_type
            );
        }
        self.generated.clone().unwrap()
    }

    fn roll_concrete(&self, investigator: &str, neuroticism: f64) -> ConcreteEvent {
        let mut rng = rand::thread_rng();
        let is_threat = if neuroticism > 0.6 {
            rng.gen_bool(0.7)
        } else {
            rng.gen_bool(0.3)
        };

        let (event_type, description) = match (&self.seed_type, is_threat) {
            (SeedType::SoundNearby, true) => (
                "hostile_encounter",
                format!("{} investigates a strange sound and finds a threat.", investigator),
            ),
            (SeedType::SoundNearby, false) => (
                "false_alarm",
                format!("{} checks the noise — it was just the wind.", investigator),
            ),
            (SeedType::SuspiciousObject, true) => (
                "dangerous_discovery",
                format!("{} discovers a dangerous object.", investigator),
            ),
            (SeedType::SuspiciousObject, false) => (
                "useful_find",
                format!("{} finds something useful.", investigator),
            ),
            (SeedType::UnusualSmell, true) => (
                "hazard_detected",
                format!("{} traces the smell to a hazard.", investigator),
            ),
            (SeedType::UnusualSmell, false) => (
                "innocuous_odor",
                format!("{} finds the source — nothing dangerous.", investigator),
            ),
            (SeedType::StrangeFootprints, true) => (
                "intruder_tracked",
                format!("{} follows footprints to an intruder's trail.", investigator),
            ),
            (SeedType::StrangeFootprints, false) => (
                "animal_tracks",
                format!("{} identifies the tracks as belonging to an animal.", investigator),
            ),
        };

        ConcreteEvent {
            event_type: event_type.to_string(),
            description,
            is_threat,
            participants: vec![investigator.to_string()],
        }
    }

    /// Whether this seed has already been investigated.
    pub fn is_investigated(&self) -> bool {
        self.generated.is_some()
    }
}
