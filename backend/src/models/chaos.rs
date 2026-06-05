//! Chaos & complex systems models (doc 08)
//! Nonlinear dynamics, self-organized criticality, cascades, antifragility.
use std::collections::{VecDeque, HashMap};
use rand::Rng;

// ============================================================
// 1. Logistic Map
// ============================================================

/// Logistic map: x_{n+1} = r * x_n * (1 - x_n)
pub struct LogisticMap {
    pub r: f64,
    pub x: f64,
    pub history: VecDeque<f64>,
}

impl LogisticMap {
    pub fn new(r: f64, x: f64) -> Self {
        let mut h = VecDeque::new();
        h.push_back(x);
        Self { r, x, history: h }
    }

    pub fn step(&mut self) -> f64 {
        self.x = self.r * self.x * (1.0 - self.x);
        self.history.push_back(self.x);
        if self.history.len() > 1000 { self.history.pop_front(); }
        self.x
    }

    pub fn regime(&self) -> Regime {
        match self.r {
            r if r < 1.0 => Regime::Extinction,
            r if r < 3.0 => Regime::Stable,
            r if r < 3.5699 => Regime::Periodic,
            _ => Regime::Chaotic,
        }
    }
}

pub enum Regime { Extinction, Stable, Periodic, Chaotic }

// ============================================================
// 2. Lyapunov Exponent
// ============================================================

/// λ = (1/N) * Σ ln|r·(1-2x_n)|
pub fn lyapunov_exponent(r: f64, orbit: &[f64]) -> f64 {
    let n = orbit.len();
    if n == 0 { return 0.0; }
    let sum: f64 = orbit.iter().map(|&x| (r * (1.0 - 2.0 * x)).abs().ln()).sum();
    sum / n as f64
}

// ============================================================
// 3. Strange Attractor (social Lorenz)
// ============================================================

/// Lorenz-like system: dx/dt = σ(y-x), dy/dt = x(ρ-z)-y, dz/dt = xy-βz
pub struct SocialAttractor {
    pub x: f64, pub y: f64, pub z: f64,
    pub sigma: f64, pub rho: f64, pub beta: f64,
}

impl Default for SocialAttractor {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0, z: 1.0, sigma: 10.0, rho: 28.0, beta: 8.0 / 3.0 }
    }
}

impl SocialAttractor {
    pub fn step(&mut self, dt: f64) -> (f64, f64, f64) {
        let dx = self.sigma * (self.y - self.x);
        let dy = self.x * (self.rho - self.z) - self.y;
        let dz = self.x * self.y - self.beta * self.z;
        self.x += dx * dt;
        self.y += dy * dt;
        self.z += dz * dt;
        (self.x, self.y, self.z)
    }
}

// ============================================================
// 4. Bifurcation & Critical Slowing Down
// ============================================================

/// Detect if system is approaching a tipping point
pub fn critical_slowing_down(recovery_rates: &[f64], threshold: f64) -> bool {
    if recovery_rates.len() < 3 { return false; }
    let recent: f64 = recovery_rates.iter().rev().take(3).sum::<f64>() / 3.0;
    recent < threshold
}

/// Saddle-node bifurcation: dx/dt = r + x²
pub fn saddle_node_bifurcation(r: f64, x: f64, dt: f64) -> f64 {
    x + (r + x * x) * dt
}

/// Pitchfork bifurcation: dx/dt = r·x - x³
pub fn pitchfork_bifurcation(r: f64, x: f64, dt: f64) -> f64 {
    x + (r * x - x * x * x) * dt
}

// ============================================================
// 5. SOC Sandpile
// ============================================================

/// Bak-Tang-Wiesenfeld sandpile model
pub struct CrisisSandpile {
    pub stress: Vec<f64>,
    pub threshold: f64,
    pub avalanche_sizes: Vec<u64>,
    rng: rand::rngs::ThreadRng,
}

impl CrisisSandpile {
    pub fn new(size: usize, threshold: f64) -> Self {
        Self {
            stress: vec![0.0; size],
            threshold,
            avalanche_sizes: vec![],
            rng: rand::thread_rng(),
        }
    }

    /// Add random stress to a random cell
    pub fn add_random_stress(&mut self, amount: f64) -> u64 {
        let idx = self.rng.gen_range(0..self.stress.len());
        self.add_stress(idx, amount)
    }

    /// Add stress to a specific cell; returns avalanche size
    pub fn add_stress(&mut self, idx: usize, amount: f64) -> u64 {
        self.stress[idx] += amount;
        if self.stress[idx] >= self.threshold {
            self.stress[idx] = 0.0;
            let size = self.cascade(idx);
            self.avalanche_sizes.push(size);
            size
        } else { 0 }
    }

    fn cascade(&mut self, origin: usize) -> u64 {
        let mut queue = VecDeque::new();
        queue.push_back(origin);
        let mut count = 0;
        while let Some(current) = queue.pop_front() {
            count += 1;
            let n = self.stress.len();
            for &neighbor in &[current.wrapping_sub(1), current + 1] {
                if neighbor < n {
                    self.stress[neighbor] += 1.0;
                    if self.stress[neighbor] >= self.threshold {
                        self.stress[neighbor] = 0.0;
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        count
    }

    /// Estimate power-law exponent from avalanche size distribution
    pub fn power_law_exponent(&self) -> f64 {
        let n = self.avalanche_sizes.len() as f64;
        if n < 10.0 { return 0.0; }
        let sum_log: f64 = self.avalanche_sizes.iter().filter(|&&s| s > 0)
            .map(|&s| (s as f64).ln()).sum();
        1.0 + n / sum_log.max(1.0)
    }
}

// ============================================================
// 6. Power Law Distribution
// ============================================================

/// Sample from P(x) ~ x^(-α), x >= x_min
pub fn sample_power_law(alpha: f64, x_min: f64) -> f64 {
    let u: f64 = rand::thread_rng().gen();
    x_min * (1.0 - u).powf(-1.0 / (alpha - 1.0))
}

/// PDF of power law: P(x) = (α-1)/x_min * (x/x_min)^(-α)
pub fn power_law_pdf(x: f64, alpha: f64, x_min: f64) -> f64 {
    if x < x_min { return 0.0; }
    (alpha - 1.0) / x_min * (x / x_min).powf(-alpha)
}

// ============================================================
// 7. Lotka-Volterra Three Species
// ============================================================

/// Resources / Survivors / Zombies competition
pub struct ThreeSpecies {
    pub resource: f64, pub survivor: f64, pub zombie: f64,
    pub alpha: f64, pub beta: f64, pub delta: f64,
    pub gamma: f64, pub epsilon: f64, pub zeta: f64, pub eta: f64,
}

impl Default for ThreeSpecies {
    fn default() -> Self {
        Self {
            resource: 100.0, survivor: 10.0, zombie: 1.0,
            alpha: 0.5, beta: 0.01, delta: 0.02,
            gamma: 0.5, epsilon: 0.005, zeta: 0.01, eta: 0.1,
        }
    }
}

impl ThreeSpecies {
    pub fn step(&mut self, dt: f64) -> (f64, f64, f64) {
        let dr = self.alpha * self.resource - self.beta * self.resource * self.survivor;
        let ds = self.delta * self.resource * self.survivor - self.gamma * self.survivor - self.epsilon * self.survivor * self.zombie;
        let dz = self.zeta * self.survivor * self.zombie - self.eta * self.zombie;
        self.resource = (self.resource + dr * dt).max(0.0);
        self.survivor = (self.survivor + ds * dt).max(0.0);
        self.zombie = (self.zombie + dz * dt).max(0.0);
        (self.resource, self.survivor, self.zombie)
    }
}

// ============================================================
// 8. Edge of Chaos (Langton λ)
// ============================================================

/// λ = transitions / total_possible (0=rigid, 1=random)
pub fn langton_lambda(transitions: usize, total_possible: usize) -> f64 {
    if total_possible == 0 { return 0.0; }
    transitions as f64 / total_possible as f64
}

// ============================================================
// 9. Path Lock-In / Increasing Returns
// ============================================================

/// Probability of choosing A given fitness and adoption
pub fn increasing_returns(fitness_a: f64, fitness_b: f64, adoption_a: f64, adoption_b: f64, network_strength: f64) -> f64 {
    let perceived_a = fitness_a + adoption_a * network_strength;
    let perceived_b = fitness_b + adoption_b * network_strength;
    let total = perceived_a + perceived_b;
    if total <= 0.0 { 0.5 } else { perceived_a / total }
}

// ============================================================
// 10. Cascade Models
// ============================================================

/// Watts threshold cascade — returns final adoption states
pub fn watts_threshold_cascade(adopted: &[bool], thresholds: &[f64], adjacency: &[Vec<usize>]) -> Vec<bool> {
    let n = adopted.len();
    let mut result = adopted.to_vec();
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..n {
            if result[i] { continue; }
            let neighbor_count = adjacency[i].len();
            if neighbor_count == 0 { continue; }
            let adopted_count = adjacency[i].iter().filter(|&&j| result[j]).count();
            let fraction = adopted_count as f64 / neighbor_count as f64;
            if fraction >= thresholds[i] {
                result[i] = true;
                changed = true;
            }
        }
    }
    result
}

/// Antifragility score: diversity + redundancy + modularity + learning
pub fn antifragility_score(diversity: f64, redundancy: f64, modularity: f64, learning: f64) -> f64 {
    (diversity + redundancy + modularity + learning) / 4.0
}

/// Stress inoculation: moderate stress builds resilience
pub fn stress_inoculation(past_stress: f64, current_stress: f64) -> f64 {
    if current_stress <= 0.0 { return 0.0; }
    let ratio = past_stress / current_stress.max(0.01);
    if ratio < 1.0 {
        ratio.max(0.0) // overwhelmed
    } else if ratio < 3.0 {
        1.0 + ratio * 0.2 // growing
    } else {
        (1.0 + ratio * 0.3).min(3.0) // antifragile
    }
}
