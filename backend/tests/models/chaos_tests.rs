#[cfg(test)]
mod chaos_tests {
    use vivid::models::chaos::*;

    #[test]
    fn logistic_map_converges() {
        let mut m = LogisticMap::new(2.5, 0.5);
        for _ in 0..50 { m.step(); }
        assert!((m.x - 0.6).abs() < 0.01);
    }

    #[test]
    fn logistic_map_chaotic_does_not_converge() {
        let mut m = LogisticMap::new(3.9, 0.5);
        let mut vals = vec![];
        for _ in 0..200 { vals.push(m.step()); }
        let last_50: Vec<i64> = vals.iter().rev().take(50).map(|v| (v * 1000.0) as i64).collect();
        let unique: std::collections::HashSet<_> = last_50.into_iter().collect();
        assert!(unique.len() > 10, "chaotic regime should produce diverse values, got {} unique", unique.len());
    }

    #[test]
    fn sandpile_avalanche() {
        let mut pile = CrisisSandpile::new(10, 4.0);
        let size = pile.add_stress(0, 4.0);
        assert!(size >= 1);
    }

    #[test]
    fn sandpile_no_avalanche_below_threshold() {
        let mut pile = CrisisSandpile::new(10, 4.0);
        let size = pile.add_stress(0, 1.0);
        assert_eq!(size, 0);
    }

    #[test]
    fn lyapunov_positive_for_chaos() {
        let mut m = LogisticMap::new(3.6, 0.5);
        let mut orbit = vec![];
        for _ in 0..100 { orbit.push(m.step()); }
        let lyap = lyapunov_exponent(3.6, &orbit);
        assert!(lyap > 0.0, "chaotic regime should have positive Lyapunov exponent");
    }

    #[test]
    fn three_species_stable() {
        let mut sys = ThreeSpecies::default();
        for _ in 0..100 { sys.step(0.01); }
        assert!(sys.resource >= 0.0);
        assert!(sys.survivor >= 0.0);
        assert!(sys.zombie >= 0.0);
    }

    #[test]
    fn power_law_sample_valid() {
        for _ in 0..1000 {
            let s = sample_power_law(2.5, 1.0);
            assert!(s >= 1.0, "power law samples must >= x_min");
        }
    }

    #[test]
    fn critical_slowing_detection() {
        let rates = vec![0.1, 0.08, 0.05, 0.03, 0.01, 0.005];
        assert!(critical_slowing_down(&rates, 0.02));
    }
}
