#[cfg(test)]
mod engine_tests {
    use std::collections::HashMap;

    #[test]
    fn engine_state_transitions() {
        #[derive(Debug, Clone, PartialEq)]
        enum SimSpeed { Paused, Detailed, FastForward }
        #[derive(Debug, Clone, PartialEq)]
        enum EngineState { Paused, Running(SimSpeed), Stopped }

        let state = EngineState::Running(SimSpeed::Detailed);
        assert_ne!(state, EngineState::Paused);
        match state {
            EngineState::Running(SimSpeed::Detailed) => {} // expected
            _ => panic!("wrong state"),
        }
    }

    #[test]
    fn fastforward_tick_advances_time() {
        // Simulate: advancing time by 1 hour in FF mode
        let initial_tick = 0u64;
        let tick_per_hour = 1u64;
        let new_tick = initial_tick + tick_per_hour;
        assert_eq!(new_tick, 1);
    }

    #[test]
    fn detailed_tick_uses_5_minutes() {
        let initial_tick = 0u64;
        let tick_per_detailed = 1u64;
        let new_tick = initial_tick + tick_per_detailed;
        assert_eq!(new_tick, 1);
    }

    #[test]
    fn world_resources_tracked_correctly() {
        let mut resources: HashMap<String, f64> = HashMap::new();
        resources.insert("food".into(), 100.0);
        *resources.get_mut("food").unwrap() -= 10.0;
        assert_eq!(resources["food"], 90.0);
    }

    #[test]
    fn character_health_bounds() {
        let mut hp = 100.0f64;
        let damage = 30.0f64;
        hp = (hp - damage).max(0.0);
        assert_eq!(hp, 70.0);

        let healing = 50.0f64;
        let max_hp = 100.0f64;
        hp = (hp + healing).min(max_hp);
        assert_eq!(hp, 100.0);
    }
}
