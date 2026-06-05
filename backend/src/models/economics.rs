//! Economics & decision theory models (doc 09)
//! Utility, prospect theory, time discounting, production functions, barter.
use rand::Rng;

// ============================================================
// 1. Utility Curves
// ============================================================

pub enum UtilityCurve {
    Logarithmic,
    Power(f64),
    ExponentialSaturation(f64),
}

impl UtilityCurve {
    pub fn evaluate(&self, x: f64) -> f64 {
        match self {
            Self::Logarithmic => if x <= 0.0 { 0.0 } else { (x + 1.0).ln() },
            Self::Power(a) => if x <= 0.0 { 0.0 } else { x.powf(*a) },
            Self::ExponentialSaturation(k) => 1.0 - (-k * x).exp(),
        }
    }

    pub fn marginal(&self, x: f64) -> f64 {
        let eps = 1e-6;
        (self.evaluate(x + eps) - self.evaluate(x)) / eps
    }
}

// ============================================================
// 2. Prospect Theory (Kahneman-Tversky)
// ============================================================

/// U(gain) = x^0.88,  U(loss) = -2.25 * |x|^0.88
pub fn prospect_utility(value: f64, reference: f64) -> f64 {
    let diff = value - reference;
    if diff >= 0.0 { diff.powf(0.88) } else { -2.25 * (-diff).powf(0.88) }
}

/// Probability weighting: w(p) = p^0.61 / (p^0.61 + (1-p)^0.61)^(1/0.61)
pub fn prospect_weight(probability: f64) -> f64 {
    if probability <= 0.0 || probability >= 1.0 { return probability; }
    let p = probability.powf(0.61);
    let q = (1.0 - probability).powf(0.61);
    p / (p + q).powf(1.0 / 0.61)
}

/// Reference point with exponential moving average
pub struct ReferencePoint {
    pub value: f64,
    pub adaptation_rate: f64,
}

impl ReferencePoint {
    pub fn new(initial: f64, rate: f64) -> Self {
        Self { value: initial, adaptation_rate: rate.clamp(0.01, 0.2) }
    }
    pub fn update(&mut self, current: f64) {
        self.value += self.adaptation_rate * (current - self.value);
    }
}

// ============================================================
// 3. Hedonic Treadmill
// ============================================================

/// Exponential decay to baseline: half-life determines decay rate
pub fn hedonic_adaptation(initial_response: f64, half_life_days: f64, days_passed: f64) -> f64 {
    if half_life_days <= 0.0 { return 0.0; }
    initial_response * (-0.693 * days_passed / half_life_days).exp()
}

// ============================================================
// 4. Satisficing (Simon, 1956)
// ============================================================

pub struct SatisficingDecision {
    pub threshold: f64,
    pub satisfied: bool,
}

impl SatisficingDecision {
    pub fn new(threshold: f64) -> Self {
        Self { threshold, satisfied: false }
    }
    pub fn evaluate(&mut self, current_value: f64) -> bool {
        self.satisfied = current_value >= self.threshold;
        self.satisfied
    }
}

// ============================================================
// 5. Opportunity Cost
// ============================================================

pub fn opportunity_cost(chosen: f64, forgone: f64) -> f64 {
    (forgone - chosen).max(0.0)
}

pub fn opportunity_cost_weight(cost: f64, chosen: f64) -> f64 {
    if chosen <= 0.0 { return 0.0; }
    (cost / chosen).min(1.0)
}

// ============================================================
// 6. Time Discounting
// ============================================================

/// PV = R / (1 + δ)^t
pub fn exponential_discount(value: f64, delay: f64, delta: f64) -> f64 {
    if delay < 0.0 { return value; }
    value / (1.0 + delta).powf(delay)
}

/// PV = R / (1 + k·t)  — hyperbolic, produces time-inconsistency
pub fn hyperbolic_discount(value: f64, delay: f64, k: f64) -> f64 {
    if delay < 0.0 { return value; }
    value / (1.0 + k * delay).max(0.01)
}

// ============================================================
// 7. Demand Elasticity
// ============================================================

/// ε = (ΔQ/Q) / (ΔP/P)
pub fn price_elasticity(dq: f64, q: f64, dp: f64, p: f64) -> f64 {
    if q <= 0.0 || p <= 0.0 || dp == 0.0 { return 0.0; }
    (dq / q) / (dp / p)
}

/// Cross-price elasticity
pub fn cross_elasticity(dq_a: f64, q_a: f64, dp_b: f64, p_b: f64) -> f64 {
    if q_a <= 0.0 || p_b <= 0.0 || dp_b == 0.0 { return 0.0; }
    (dq_a / q_a) / (dp_b / p_b)
}

pub fn is_luxury(elasticity: f64) -> bool {
    elasticity.abs() > 1.0
}

// ============================================================
// 8. Giffen & Veblen Goods
// ============================================================

/// Giffen good: price rise → demand rise (when poor and staple)
pub fn giffen_good_demand(price: f64, income: f64, staple_budget: f64, substitute_price: f64) -> f64 {
    if price <= 0.0 { return 0.0; }
    let base = staple_budget / price;
    let substitution_effect = if substitute_price > price {
        base * 0.3 // switch to substitute
    } else { 0.0 };
    base + substitution_effect * (1.0 - income / (income + 100.0))
}

/// Veblen good: price rise → demand rise (status signaling)
pub fn veblen_demand(price: f64, snob_value: f64, base_demand: f64) -> f64 {
    let status_signal = (price / snob_value).tanh(); // saturating at ~1
    base_demand * (1.0 + status_signal)
}

// ============================================================
// 9. Production Functions
// ============================================================

/// Q = L^α · K^β
pub fn cobb_douglas(labor: f64, capital: f64, alpha: f64, beta: f64) -> f64 {
    if labor <= 0.0 || capital <= 0.0 { return 0.0; }
    labor.powf(alpha) * capital.powf(beta)
}

/// MP(L) = ∂Q/∂L = α · L^(α-1) · K^β
pub fn marginal_product_labor(labor: f64, capital: f64, alpha: f64, beta: f64) -> f64 {
    if labor <= 0.0 || capital <= 0.0 { return 0.0; }
    alpha * labor.powf(alpha - 1.0) * capital.powf(beta)
}

/// Find optimal team size where MP >= marginal cost
pub fn optimal_team_size(labor_values: &[f64], marginal_cost: f64) -> usize {
    for (i, &mp) in labor_values.iter().enumerate() {
        if mp < marginal_cost { return i.max(1); }
    }
    labor_values.len().max(1)
}

// ============================================================
// 10. Barter Trade
// ============================================================

/// Nash bargaining: rate_{A→B} = (urgency_B/urgency_A) · (scarcity_B/scarcity_A) · (1+power_ratio)
pub fn nash_bargaining_rate(urgency_a: f64, urgency_b: f64, scarcity_a: f64, scarcity_b: f64, power_ratio: f64) -> f64 {
    if urgency_a <= 0.0 || scarcity_a <= 0.0 { return 0.0; }
    (urgency_b / urgency_a) * (scarcity_b / scarcity_a) * (1.0 + power_ratio)
}

/// Double coincidence of wants probability
pub fn double_coincidence_prob(p_a_wants_b: f64, p_a_has_x: f64, p_b_wants_a: f64, p_b_has_y: f64) -> f64 {
    (p_a_has_x * p_b_wants_a) * (p_b_has_y * p_a_wants_b)
}

// ============================================================
// 11. Common Pool Resources (Ostrom)
// ============================================================

/// Ostrom 8 principles sustainability score (0-1)
pub fn ostrom_sustainability(
    boundary: f64, matching: f64, collective: f64,
    monitoring: f64, sanctions: f64, conflict: f64,
    recognition: f64, nested: f64,
) -> f64 {
    let weights = vec![0.15, 0.15, 0.15, 0.15, 0.1, 0.1, 0.1, 0.1];
    let values = vec![boundary, matching, collective, monitoring, sanctions, conflict, recognition, nested];
    let total: f64 = values.iter().zip(&weights).map(|(v, w)| v * w).sum();
    total
}

/// Insurance pool stability check
pub fn insurance_pool_stability(claims: &[f64], reserve: f64, contribution_rate: f64, member_count: usize) -> bool {
    let expected_payouts: f64 = claims.iter().sum();
    let income = contribution_rate * member_count as f64;
    reserve + income - expected_payouts > reserve * 0.2
}

/// Cooperation phase transition: C(S) = 1 / (1 + e^{k(S - S_critical)})
pub fn cooperation_phase_transition(scarcity: f64, s_critical: f64, k: f64) -> f64 {
    1.0 / (1.0 + (k * (scarcity - s_critical)).exp())
}
