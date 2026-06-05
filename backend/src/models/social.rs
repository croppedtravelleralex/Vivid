//! Social simulation core models (doc 07 — 68 formulas)
//! Opinion dynamics, norms, reciprocity, status, rumor, memory, identity, networks, collective behavior, trust.
use rand::Rng;
use std::collections::HashMap;

// ============================================================
// 1. Opinion Dynamics
// ============================================================

/// DeGroot linear consensus: x_i(t+1) = Σ_j w_ij · x_j(t)
pub fn degroot_step(opinions: &[f64], weights: &[Vec<f64>]) -> Vec<f64> {
    let n = opinions.len();
    let mut next = vec![0.0; n];
    for i in 0..n {
        for j in 0..n {
            next[i] += weights[i][j] * opinions[j];
        }
    }
    next
}

/// Friedkin-Johnsen: x_i(t+1) = λ_i·x_i(0) + (1-λ_i)·Σ_j w_ij·x_j(t)
pub struct FriedkinJohnsenState {
    pub current: Vec<f64>,
    pub initial: Vec<f64>,
    pub lambda: Vec<f64>,
    pub weights: Vec<Vec<f64>>,
}

pub fn friedkin_johnsen_step(state: &mut FriedkinJohnsenState) {
    let n = state.current.len();
    let consensus = degroot_step(&state.current, &state.weights);
    for i in 0..n {
        state.current[i] = state.lambda[i] * state.initial[i] + (1.0 - state.lambda[i]) * consensus[i];
    }
}

/// λ = (1-agreeableness·0.3) + neuroticism·0.25 + ruthlessness·0.25 - openness·0.2
pub fn lambda_from_personality(agreeableness: f64, neuroticism: f64, ruthlessness: f64, openness: f64) -> f64 {
    let raw = (1.0 - agreeableness * 0.3) + neuroticism * 0.25 + ruthlessness * 0.25 - openness * 0.2;
    raw.clamp(0.1, 0.95)
}

/// Hegselmann-Krause bounded confidence: only influence similar opinions
pub fn hk_step(opinions: &[f64], epsilon: f64) -> Vec<f64> {
    let n = opinions.len();
    opinions.iter().enumerate().map(|(i, &o)| {
        let neighbors: Vec<usize> = (0..n).filter(|&j| (opinions[j] - o).abs() < epsilon).collect();
        if neighbors.is_empty() { return o; }
        neighbors.iter().map(|&j| opinions[j]).sum::<f64>() / neighbors.len() as f64
    }).collect()
}

/// Information cascade: trigger when |#Accept - #Reject| >= 2
pub struct InformationCascade {
    pub accept_count: u32,
    pub reject_count: u32,
    pub cascaded: bool,
}

pub struct CascadeOutcome {
    pub accepted: bool,
    pub cascade: bool,
}

impl InformationCascade {
    pub fn new() -> Self { Self { accept_count: 0, reject_count: 0, cascaded: false } }
    pub fn step(&mut self, private_signal: bool) -> CascadeOutcome {
        if self.cascaded {
            let accepted = self.accept_count > self.reject_count;
            return CascadeOutcome { accepted, cascade: true };
        }
        if private_signal { self.accept_count += 1; } else { self.reject_count += 1; }
        if self.accept_count.abs_diff(self.reject_count) >= 2 {
            self.cascaded = true;
        }
        CascadeOutcome { accepted: private_signal, cascade: self.cascaded }
    }
}

/// Group polarization: discussions push groups to extremes
pub fn group_polarization_step(opinions: &mut [f64], alpha: f64) {
    let mean: f64 = opinions.iter().sum::<f64>() / opinions.len() as f64;
    for o in opinions.iter_mut() {
        let side = if *o > mean { 1.0 } else { -1.0 };
        *o += alpha * side * (mean - *o).abs();
    }
}

/// Social comparison: conform + differentiate
pub fn social_comparison_step(opinions: &[f64], beta: f64, gamma: f64, group_mode: f64, desired: f64) -> Vec<f64> {
    opinions.iter().map(|&o| {
        let conformity = beta * (group_mode - o);
        let differentiation = gamma * (desired - o);
        o + conformity + differentiation
    }).collect()
}

// ============================================================
// 2. Social Norms
// ============================================================

/// Bicchieri norm activation: E_i >= θ_E, N_i >= θ_N, P_i > Cost_i
pub struct NormActivation {
    pub empirical_threshold: f64,
    pub normative_threshold: f64,
}

impl NormActivation {
    pub fn new(empirical: f64, normative: f64) -> Self {
        Self { empirical_threshold: empirical, normative_threshold: normative }
    }
    pub fn is_active(&self, empirical: f64, normative: f64, preference: f64, cost: f64) -> bool {
        empirical >= self.empirical_threshold && normative >= self.normative_threshold && preference > cost
    }
    pub fn violation_utility(&self, material_gain: f64, punishment_risk: f64, severity: f64, guilt: f64, disapproval: f64) -> f64 {
        material_gain - punishment_risk * severity - guilt - disapproval
    }
}

/// Granovetter norm avalanche: f(t) < (I - R) / (S - R)
pub fn norm_avalanche_threshold(internal_motivation: f64, violation_reward: f64, social_pressure: f64) -> f64 {
    let denom = social_pressure - violation_reward;
    if denom <= 0.0 { return 0.0; }
    ((internal_motivation - violation_reward) / denom).clamp(0.0, 1.0)
}

/// Punishment decision: benefit > cost
pub fn should_punish(deterrence: f64, reputation_gain: f64, detection_cost: f64, action_cost: f64, retaliation_risk: f64) -> bool {
    let benefit = deterrence + reputation_gain;
    let cost = detection_cost + action_cost + retaliation_risk;
    benefit > cost
}

// ============================================================
// 3. Resource Sharing & Allocation
// ============================================================

/// x_i = (need_i / Σ need_j) * total
pub fn need_based_allocation(needs: &[f64], total: f64) -> Vec<f64> {
    let sum: f64 = needs.iter().sum();
    if sum <= 0.0 { return vec![0.0; needs.len()]; }
    needs.iter().map(|&n| (n / sum) * total).collect()
}

/// x_i = α·(1/n) + β·(need/Σneed) + γ·(contribution/Σcontribution)
pub fn mixed_allocation(count: usize, needs: &[f64], contributions: &[f64], total: f64, alpha: f64, beta: f64, gamma: f64) -> Vec<f64> {
    let n = count.max(1) as f64;
    let need_sum: f64 = needs.iter().sum();
    let contrib_sum: f64 = contributions.iter().sum();
    (0..count).map(|i| {
        let equal = 1.0 / n;
        let need_share = if need_sum > 0.0 { needs[i] / need_sum } else { 0.0 };
        let contrib_share = if contrib_sum > 0.0 { contributions[i] / contrib_sum } else { 0.0 };
        (alpha * equal + beta * need_share + gamma * contrib_share) * total
    }).collect()
}

/// Gini coefficient
pub fn gini_coefficient(values: &[f64]) -> f64 {
    let n = values.len();
    if n == 0 { return 0.0; }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let sum: f64 = sorted.iter().sum();
    if sum <= 0.0 { return 0.0; }
    let cumulative: f64 = sorted.iter().enumerate().map(|(i, &v)| (i + 1) as f64 * v).sum();
    (2.0 * cumulative / (n as f64 * sum) - (n as f64 + 1.0) / n as f64).clamp(0.0, 1.0)
}

/// Redistribution trigger: Gini > 0.3 - 0.1·cohesion - 0.15·threat + 0.2·individualism
pub fn redistribution_trigger(gini: f64, cohesion: f64, threat: f64, individualism: f64) -> bool {
    let threshold = 0.3 - 0.1 * cohesion - 0.15 * threat + 0.2 * individualism;
    gini > threshold
}

/// Scarcity mindset: bandwidth = W_max - k · scarcity^p
pub fn cognitive_bandwidth(scarcity: f64, w_max: f64, k: f64, p: f64) -> f64 {
    (w_max - k * scarcity.powf(p)).max(0.0)
}

/// Decision quality degrades with low bandwidth
pub fn decision_quality(bandwidth: f64, q_max: f64, gamma: f64) -> f64 {
    q_max * (bandwidth / q_max.max(0.01)).powf(gamma)
}

// ============================================================
// 4. Reciprocity
// ============================================================

/// Tit-for-tat
pub struct TitForTat {
    pub last_opponent_cooperated: bool,
}

impl TitForTat {
    pub fn new() -> Self { Self { last_opponent_cooperated: true } }
    pub fn decide(&self, round: u32) -> bool {
        if round == 0 { return true; } // cooperate first
        self.last_opponent_cooperated
    }
    pub fn update(&mut self, opponent_cooperated: bool) {
        self.last_opponent_cooperated = opponent_cooperated;
    }
}

/// Image scoring (Nowak & Sigmund)
pub struct ImageScore {
    pub score: f64,
}

impl ImageScore {
    pub fn new() -> Self { Self { score: 0.0 } }
    pub fn update(&mut self, helped: bool, benefit: f64, cost: f64) {
        if helped { self.score += benefit - cost; } else { self.score -= 0.1; }
    }
    pub fn help_probability(&self, gamma: f64, threshold: f64) -> f64 {
        1.0 / (1.0 + (-gamma * (self.score - threshold)).exp())
    }
}

/// Strong reciprocity: U = material - c_punish + β·(norm_enforcement - cost)
pub fn strong_reciprocity(material: f64, punish_cost: f64, norm_enforcement: f64, cost_to_self: f64, beta: f64) -> f64 {
    material - punish_cost + beta * (norm_enforcement - cost_to_self)
}

// ============================================================
// 5. Status & Leadership
// ============================================================

/// Prestige + Dominance dual-channel status
pub struct StatusModel {
    pub prestige: f64,
    pub dominance: f64,
    pub gamma: f64,         // weight for overall = γ·prestige + (1-γ)·dominance
}

impl StatusModel {
    pub fn new(gamma: f64) -> Self { Self { prestige: 0.0, dominance: 0.0, gamma } }
    pub fn update_prestige(&mut self, deference: f64, skill: f64, decay: f64) {
        self.prestige = self.prestige * (1.0 - decay) + deference * skill;
    }
    pub fn update_dominance(&mut self, coercion: f64, resistance: f64, decay: f64) {
        self.dominance = self.dominance * (1.0 - decay) + coercion * (1.0 - resistance);
    }
    pub fn overall_status(&self) -> f64 { self.gamma * self.prestige + (1.0 - self.gamma) * self.dominance }
}

/// Matthew effect: dS/dt = a·S + b·contribution
pub fn matthew_effect(status: f64, contribution: f64, a: f64, b: f64) -> f64 {
    a * status + b * contribution
}

/// Hollander idiosyncrasy credit
pub fn idiosyncrasy_credit(conformity: f64, competence: f64, deviance: f64, w_c: f64, w_comp: f64, w_d: f64, gamma: f64, tau: u64) -> f64 {
    let base = conformity * w_c + competence * w_comp - deviance * w_d;
    base * gamma.powi(tau as i32)
}

/// Keltner power corruption: P(abuse) = σ(θ₁·Power + θ₂·Lack_of_Accountability - θ₃·Institutional_Constraints)
pub fn abuse_likelihood(power: f64, accountability: f64, constraints: f64, theta1: f64, theta2: f64, theta3: f64) -> f64 {
    let raw = theta1 * power + theta2 * (1.0 - accountability) - theta3 * constraints;
    1.0 / (1.0 + (-raw).exp())
}

// ============================================================
// 6. Rumor & Information Spread
// ============================================================

/// DK rumor model (simplified SIR)
pub struct RumorState {
    pub ignorant: f64, pub spreader: f64, pub stifler: f64,
    pub beta: f64, pub alpha: f64,
}

impl RumorState {
    pub fn new(beta: f64, alpha: f64) -> Self {
        Self { ignorant: 0.99, spreader: 0.01, stifler: 0.0, beta, alpha }
    }
    pub fn step(&mut self, dt: f64) {
        let di = -self.beta * self.spreader * self.ignorant;
        let ds = self.beta * self.spreader * self.ignorant - self.alpha * self.spreader * (self.spreader + self.stifler);
        let dr = self.alpha * self.spreader * (self.spreader + self.stifler);
        self.ignorant = (self.ignorant + di * dt).clamp(0.0, 1.0);
        self.spreader = (self.spreader + ds * dt).clamp(0.0, 1.0);
        self.stifler = (self.stifler + dr * dt).clamp(0.0, 1.0);
    }
}

/// Rumor mutation: M_{t+1} = M_t · (1 - λ) + δ + ε
pub fn rumor_mutation(msg: &str, leveling: f64, sharpening: f64, assimilation: f64) -> String {
    let len = msg.len();
    if len < 3 { return msg.to_string(); }
    let retained = (len as f64 * (1.0 - leveling)).max(3.0) as usize;
    let mut result: String = msg.chars().take(retained).collect();
    if rand::thread_rng().gen_bool(sharpening as f64) {
        result.push_str("!!");
    }
    if rand::thread_rng().gen_bool(assimilation as f64) {
        result = format!("[扭曲] {}", result);
    }
    result
}

/// Panic cascade: P(panic_i) = σ(α_i · (visible_panic - T_i))
pub fn panic_cascade(visible_panic: f64, thresholds: &[f64], sensitivity: f64) -> Vec<f64> {
    thresholds.iter().map(|&t| {
        let raw = sensitivity * (visible_panic - t);
        1.0 / (1.0 + (-raw).exp())
    }).collect()
}

// ============================================================
// 7. Collective Memory
// ============================================================

/// Ebbinghaus forgetting curve: R(t) = exp(-t / S)
pub fn forgetting_curve(t: f64, stability: f64) -> f64 {
    if stability <= 0.0 { return 0.0; }
    (-t / stability).exp()
}

/// Socially-retrieved forgetting: mentioned → strengthen, omitted → suppress
pub fn social_memory_update(memory: f64, mentioned: bool, mu: f64, nu: f64) -> f64 {
    if mentioned {
        memory + mu * (1.0 - memory)
    } else {
        memory * (1.0 - nu)
    }.clamp(0.0, 1.0)
}

// ============================================================
// 8. Social Identity
// ============================================================

/// ID = α·ingroup_density + β·outgroup_salience + γ·common_fate
pub fn social_identity(ingroup_density: f64, outgroup_salience: f64, common_fate: f64, alpha: f64, beta: f64, gamma: f64) -> f64 {
    (alpha * ingroup_density + beta * outgroup_salience + gamma * common_fate).clamp(0.0, 1.0)
}

/// Allocation bias: in = base + bias·ID, out = base - bias·ID
pub fn allocation_bias(identity: f64, base: f64, bias: f64) -> (f64, f64) {
    (base + bias * identity, (base - bias * identity).max(0.0))
}

// ============================================================
// 9. Social Network Dynamics
// ============================================================

/// Triadic closure: friend of friend becomes friend
pub fn triadic_closure(trust_ab: f64, trust_ac: f64, current_bc: f64, delta: f64) -> f64 {
    let min_trust = trust_ab.min(trust_ac);
    if min_trust > 0.5 { (current_bc + delta).min(10.0) } else { current_bc }
}

/// Balance theory: enemy of enemy is friend
pub fn balance_theory(attitude_ab: f64, attitude_bc: f64, current_ac: f64, delta: f64) -> f64 {
    if attitude_ab < 0.0 && attitude_bc < 0.0 {
        (current_ac + delta).min(10.0) // enemies' enemy
    } else if attitude_ab < 0.0 || attitude_bc < 0.0 {
        (current_ac - delta).max(-10.0) // friend of enemy
    } else {
        current_ac
    }
}

// ============================================================
// 10. Collective Behavior
// ============================================================

/// Granovetter threshold cascade: join if fraction ≥ threshold
pub fn threshold_cascade(thresholds: &[f64]) -> Vec<f64> {
    let n = thresholds.len();
    let mut joined = vec![false; n];
    joined[0] = true; // seed
    loop {
        let fraction = joined.iter().filter(|&&j| j).count() as f64 / n as f64;
        let mut changed = false;
        for i in 0..n {
            if !joined[i] && fraction >= thresholds[i] {
                joined[i] = true;
                changed = true;
            }
        }
        if !changed { break; }
    }
    joined.iter().map(|&j| if j { 1.0 } else { 0.0 }).collect()
}

/// Complex contagion: need multiple exposures
pub fn complex_contagion(exposures: &[u32], threshold: u32) -> Vec<bool> {
    exposures.iter().map(|&e| e >= threshold).collect()
}

/// Asch conformity: P = majority / (majority + minority + 1)
pub fn asch_conformity(majority: usize, minority: usize) -> f64 {
    majority as f64 / (majority + minority + 1) as f64
}

// ============================================================
// 11. Trust & Relationship Evolution
// ============================================================

/// Δtrust with asymmetry: |penalty| > |reward|
pub fn trust_update(current: f64, expected: f64, actual: f64, reward_weight: f64, penalty_weight: f64) -> f64 {
    if actual >= expected {
        (current + reward_weight * (actual - expected)).min(10.0)
    } else {
        (current - penalty_weight * (expected - actual)).max(-10.0)
    }
}

/// Relationship decay over days without interaction
pub fn relationship_decay(trust: f64, familiarity: f64, loyalty: f64, days_since_interaction: f64) -> (f64, f64) {
    let decay = 1.0 - loyalty * 0.5;
    let decay_amount = decay * days_since_interaction * 0.1;
    (
        (trust - decay_amount).max(-10.0),
        (familiarity - decay_amount * 2.0).max(0.0),
    )
}

/// Label based on trust & familiarity thresholds
pub fn relationship_label(trust: f64, familiarity: f64) -> &'static str {
    if trust >= 8.0 && familiarity >= 9.0 { "挚友" }
    else if trust >= 6.0 && familiarity >= 7.0 { "朋友" }
    else if trust >= 4.0 && familiarity >= 5.0 { "熟人" }
    else if trust < 0.0 && trust >= -6.0 { "有敌意" }
    else if trust < -6.0 { "敌人" }
    else { "认识" }
}

// ============================================================
// 12. Helper: Build trust weights from relationship values
// ============================================================

pub fn build_trust_weights(relationships: &[(f64, f64)], n: usize) -> Vec<Vec<f64>> {
    let mut w = vec![vec![0.0; n]; n];
    for i in 0..n {
        let mut total = 0.0;
        for j in 0..n {
            if i != j {
                let idx = if j > i { i * n + j - i - 1 } else { j * n + i - j - 1 };
                if let Some(&(trust, _)) = relationships.get(idx) {
                    if trust > 0.0 { w[i][j] = trust / 10.0; total += w[i][j]; }
                }
            }
        }
        w[i][i] = (1.0 - total).max(0.0);
    }
    w
}
