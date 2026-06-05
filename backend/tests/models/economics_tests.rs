#[cfg(test)]
mod economics_tests {
    use vivid::models::economics::*;

    #[test]
    fn prospect_theory_loss_aversion() {
        let gain = prospect_utility(10.0, 0.0);
        let loss = prospect_utility(0.0, 10.0);
        let ratio = loss.abs() / gain;
        assert!((ratio - 2.25).abs() < 0.3);
    }

    #[test]
    fn hyperbolic_discount_present_bias() {
        let near = hyperbolic_discount(100.0, 1.0, 0.5);
        let far = hyperbolic_discount(100.0, 100.0, 0.5);
        assert!(near > far, "nearer rewards should be valued more");
    }

    #[test]
    fn exponential_discount_consistent() {
        let t1 = exponential_discount(100.0, 10.0, 0.1);
        let t2 = exponential_discount(100.0, 20.0, 0.1);
        let ratio1 = t2 / t1;
        let t3 = exponential_discount(100.0, 30.0, 0.1);
        let ratio2 = t3 / t2;
        assert!((ratio1 - ratio2).abs() < 0.01, "exponential discount should be time-consistent");
    }

    #[test]
    fn cobb_douglas_positive() {
        let q = cobb_douglas(10.0, 5.0, 0.6, 0.3);
        assert!(q > 0.0, "production should be positive");
    }

    #[test]
    fn gini_matches_social() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let g = vivid::models::social::gini_coefficient(&values);
        assert!(g > 0.2 && g < 0.3);
    }

    #[test]
    fn diminishing_marginal_utility() {
        let curve = UtilityCurve::Logarithmic;
        let m1 = curve.marginal(1.0);
        let m2 = curve.marginal(10.0);
        assert!(m1 > m2, "marginal utility should decrease");
    }

    #[test]
    fn cooperation_phase_transition_basic() {
        let low = cooperation_phase_transition(0.3, 0.75, 5.0);
        let high = cooperation_phase_transition(0.9, 0.75, 5.0);
        assert!(low > 0.9, "low scarcity → high cooperation, got {}", low);
        assert!(high < 0.5, "high scarcity → low cooperation, got {}", high);
    }
}
