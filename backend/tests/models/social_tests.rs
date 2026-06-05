#[cfg(test)]
mod social_tests {
    use vivid::models::social::*;

    #[test]
    fn degroot_converges_to_average() {
        let opinions = vec![0.2, 0.8];
        let weights = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let result = degroot_step(&opinions, &weights);
        assert!((result[0] - 0.5).abs() < 0.001);
        assert!((result[1] - 0.5).abs() < 0.001);
    }

    #[test]
    fn hk_bounded_confidence_ignores_outliers() {
        let opinions = vec![0.1, 0.9]; // far apart
        let result = hk_step(&opinions, 0.3);
        assert!((result[0] - 0.1).abs() < 0.001); // no influence
        assert!((result[1] - 0.9).abs() < 0.001);
    }

    #[test]
    fn trust_update_asymmetric() {
        let result = trust_update(5.0, 3.0, 5.0, 0.5, 2.5);
        assert!(result > 5.0, "reward should increase trust");

        let result2 = trust_update(5.0, 5.0, 2.0, 0.5, 2.5);
        assert!(result2 < 5.0, "penalty should decrease trust strongly");
        assert!((5.0 - result2) > (result - 5.0), "penalty should be larger than reward");
    }

    #[test]
    fn gini_coefficient_uniform() {
        let values = vec![10.0, 10.0, 10.0];
        assert!(gini_coefficient(&values) < 0.01, "uniform should have ~0 gini");
    }

    #[test]
    fn gini_coefficient_extreme() {
        let values = vec![100.0, 0.0, 0.0];
        assert!(gini_coefficient(&values) > 0.5, "extreme inequality");
    }

    #[test]
    fn relationship_label_test() {
        assert_eq!(relationship_label(9.0, 10.0), "挚友");
        assert_eq!(relationship_label(7.0, 8.0), "朋友");
        assert_eq!(relationship_label(5.0, 6.0), "熟人");
        assert_eq!(relationship_label(-3.0, 0.0), "有敌意");
        assert_eq!(relationship_label(-7.0, 0.0), "敌人");
    }

    #[test]
    fn friedkin_johnsen_lambda_1_never_changes() {
        let mut state = FriedkinJohnsenState {
            current: vec![0.3],
            initial: vec![0.3],
            lambda: vec![1.0],
            weights: vec![vec![1.0]],
        };
        for _ in 0..100 { friedkin_johnsen_step(&mut state); }
        assert!((state.current[0] - 0.3).abs() < 0.001);
    }

    #[test]
    fn asch_conformity_basic() {
        let p = asch_conformity(3, 1);
        assert!(p > 0.5 && p < 0.8);
    }
}
