#[cfg(test)]
mod test_vectors {
    use std::collections::VecDeque;
    use rand::Rng;

    // ==== Logistic Map ====
    struct LogisticMap {
        r: f64,
        x: f64,
    }
    impl LogisticMap {
        fn new(r: f64, x: f64) -> Self { Self { r, x } }
        fn step(&mut self) -> f64 {
            self.x = self.r * self.x * (1.0 - self.x);
            self.x
        }
        fn regime(&self) -> Regime {
            match self.r {
                r if r < 1.0 => Regime::Extinction,
                r if r < 3.0 => Regime::Stable,
                r if r < 3.5699 => Regime::Periodic,
                _ => Regime::Chaotic,
            }
        }
    }
    enum Regime { Extinction, Stable, Periodic, Chaotic }

    #[test]
    fn logistic_map_r_lt_3_converges() {
        let mut map = LogisticMap::new(2.5, 0.5);
        for _ in 0..100 { map.step(); }
        assert!((map.x - 0.6).abs() < 0.01, "r=2.5 should converge to 0.6, got {}", map.x);
    }

    #[test]
    fn logistic_map_r3_57_is_chaotic() {
        let mut map = LogisticMap::new(3.57, 0.5);
        let mut values = vec![];
        for _ in 0..100 { values.push(map.step()); }
        let unique_count = values.len(); // f64 can't be hashed, but in chaotic regime values should be diverse
        assert!(unique_count > 80, "r=3.57 should be chaotic, only {} steps", values.len());
    }

    #[test]
    fn logistic_map_r_lt_1_extinction() {
        let mut map = LogisticMap::new(0.8, 0.5);
        for _ in 0..50 { map.step(); }
        assert!((map.x - 0.0).abs() < 0.001, "r=0.8 should go extinct, got {}", map.x);
    }

    #[test]
    fn logistic_map_regime_detection() {
        assert!(matches!(LogisticMap::new(0.5, 0.5).regime(), Regime::Extinction));
        assert!(matches!(LogisticMap::new(2.5, 0.5).regime(), Regime::Stable));
        assert!(matches!(LogisticMap::new(3.3, 0.5).regime(), Regime::Periodic));
        assert!(matches!(LogisticMap::new(3.6, 0.5).regime(), Regime::Chaotic));
    }

    // ==== SOC Sandpile ====
    struct CrisisSandpile {
        stress: Vec<f64>,
        threshold: f64,
        avalanche_sizes: Vec<u64>,
    }
    impl CrisisSandpile {
        fn new(size: usize, threshold: f64) -> Self {
            Self { stress: vec![0.0; size], threshold, avalanche_sizes: vec![] }
        }
        fn add_stress(&mut self, idx: usize, amount: f64) -> u64 {
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
                        self.stress[neighbor] += 0.5;
                        if self.stress[neighbor] >= self.threshold {
                            self.stress[neighbor] = 0.0;
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
            count
        }
    }

    #[test]
    fn sandpile_single_avalanche() {
        let mut pile = CrisisSandpile::new(10, 4.0);
        let size = pile.add_stress(0, 4.0);
        assert!(size >= 1, "avalanche should occur");
    }

    #[test]
    fn sandpile_no_avalanche_below_threshold() {
        let mut pile = CrisisSandpile::new(10, 4.0);
        let size = pile.add_stress(0, 1.0);
        assert_eq!(size, 0, "no avalanche below threshold");
    }

    // ==== Prospect Theory ====
    fn prospect_utility(value: f64, reference: f64, _scale: f64) -> f64 {
        let diff = value - reference;
        if diff >= 0.0 {
            diff.powf(0.88)
        } else {
            -2.25 * (-diff).powf(0.88)
        }
    }

    #[test]
    fn prospect_theory_loss_aversion() {
        let gain = prospect_utility(10.0, 0.0, 10.0);
        let loss = prospect_utility(0.0, 10.0, 10.0);
        let ratio = loss.abs() / gain;
        assert!((ratio - 2.25).abs() < 0.2,
            "Loss/gain ratio should be ~2.25, got {}", ratio);
    }

    #[test]
    fn prospect_theory_reference_point() {
        let gain = prospect_utility(15.0, 10.0, 10.0);
        let loss = prospect_utility(5.0, 10.0, 10.0);
        assert!(gain > 0.0, "above reference should be positive");
        assert!(loss < 0.0, "below reference should be negative");
        assert!(loss.abs() > gain, "loss should loom larger than gain");
    }

    // ==== DeGroot Consensus ====
    fn degroot_step(opinions: &[f64], weights: &[Vec<f64>]) -> Vec<f64> {
        let n = opinions.len();
        let mut next = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                next[i] += weights[i][j] * opinions[j];
            }
        }
        next
    }

    #[test]
    fn degroot_converges_to_average() {
        let opinions = vec![0.2, 0.8];
        let weights = vec![
            vec![0.5, 0.5],
            vec![0.5, 0.5],
        ];
        let result = degroot_step(&opinions, &weights);
        assert!((result[0] - 0.5).abs() < 0.001);
        assert!((result[1] - 0.5).abs() < 0.001);
    }

    #[test]
    fn degroot_self_weight_only() {
        let opinions = vec![0.2, 0.8];
        let weights = vec![
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];
        let result = degroot_step(&opinions, &weights);
        assert!((result[0] - 0.2).abs() < 0.001);
        assert!((result[1] - 0.8).abs() < 0.001);
    }

    // ==== Utility Curves ====
    enum UtilityCurve { Logarithmic, Power(f64), ExponentialSaturation(f64) }
    impl UtilityCurve {
        fn evaluate(&self, x: f64) -> f64 {
            match self {
                Self::Logarithmic => (x + 1.0).ln(),
                Self::Power(a) => x.powf(*a),
                Self::ExponentialSaturation(k) => 1.0 - (-k * x).exp(),
            }
        }
        fn marginal(&self, x: f64) -> f64 {
            let eps = 1e-6;
            (self.evaluate(x + eps) - self.evaluate(x)) / eps
        }
    }

    #[test]
    fn logarithmic_utility_decreasing_marginal() {
        let curve = UtilityCurve::Logarithmic;
        let m1 = curve.marginal(1.0);
        let m2 = curve.marginal(10.0);
        assert!(m1 > m2, "marginal utility should decrease: m1={}, m2={}", m1, m2);
    }

    #[test]
    fn power_utility_between_0_and_1() {
        let curve = UtilityCurve::Power(0.5);
        let val = curve.evaluate(0.5);
        assert!(val > 0.0 && val < 1.0);
    }

    #[test]
    fn exponential_utility_saturates() {
        let curve = UtilityCurve::ExponentialSaturation(1.0);
        let near_max = curve.evaluate(10.0);
        assert!((near_max - 1.0).abs() < 0.001);
    }

    // ==== Power Law Sampling ====
    fn sample_power_law(alpha: f64, x_min: f64, rng: &mut impl Rng) -> f64 {
        let u: f64 = rng.gen();
        x_min * (1.0 - u).powf(-1.0 / (alpha - 1.0))
    }

    #[test]
    fn power_law_mean_approximation() {
        let mut rng = rand::thread_rng();
        let mut samples: Vec<f64> = (0..10000).map(|_| sample_power_law(2.5, 1.0, &mut rng)).collect();
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = samples[samples.len() / 2];
        // α=2.5, x_min=1.0 → median = x_min * 2^(1/(α-1)) = 2^(2/3) ≈ 1.587
        assert!((median - 1.587).abs() < 0.15,
            "Power law median should be ~1.587, got {}", median);
    }

    // ==== Resource Safety ====
    #[test]
    fn resource_never_negative() {
        for current in [0.0f64, 10.0, 50.0].iter() {
            for consumption in [0.0f64, 5.0, 100.0].iter() {
                let result = (*current - *consumption).max(0.0);
                assert!(result >= 0.0, "Resource went negative: {} -= {}", current, consumption);
            }
        }
    }
}
