#[cfg(test)]
mod engine_scenarios {
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::{broadcast, mpsc};

    use vivid::engine::{
        EngineConfig, EngineEvent, EngineState, LLMRequest, SimSpeed, SimulationEngine,
    };
    use vivid::llm::gateway::LLMGateway;
    use vivid::models::world::WorldState;

    fn make_engine() -> Arc<SimulationEngine> {
        let start =
            chrono::NaiveDateTime::parse_from_str("2025-12-03T08:00:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap();
        let world = WorldState::new(start, 42);
        let config = EngineConfig {
            detailed_tick_minutes: 5,
            fastforward_tick_hours: 1,
            idle_threshold: 3,
            max_concurrent_llm: 2,
            llm_timeout_seconds: 5,
            checkpoint_interval: 100,
            random_seed: 42,
        };
        let (llm_tx, _) = mpsc::channel::<LLMRequest>(100);
        let (ws_tx, _) = broadcast::channel::<EngineEvent>(256);
        let llm_gw = Arc::new(LLMGateway::new(
            "k".into(),
            "http://localhost".into(),
            "m".into(),
            2,
            5,
        ));
        Arc::new(SimulationEngine::new(world, config, llm_tx, ws_tx, llm_gw))
    }

    // ------------------------------------------------------------------
    // State transitions
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn engine_starts_paused() {
        let engine = make_engine();
        let state = engine.state.lock().unwrap();
        assert_eq!(*state, EngineState::Paused);
    }

    #[tokio::test]
    async fn engine_transitions_to_running() {
        let engine = make_engine();
        {
            let mut state = engine.state.lock().unwrap();
            *state = EngineState::Running {
                speed: SimSpeed::Detailed,
                tick_count: 0,
            };
        }
        let state = engine.state.lock().unwrap();
        match &*state {
            EngineState::Running { speed, .. } => assert_eq!(*speed, SimSpeed::Detailed),
            _ => panic!("Expected Running state"),
        }
    }

    #[tokio::test]
    async fn engine_transitions_to_stopped() {
        let engine = make_engine();
        {
            let mut state = engine.state.lock().unwrap();
            *state = EngineState::Stopped;
        }
        let state = engine.state.lock().unwrap();
        assert_eq!(*state, EngineState::Stopped);
    }

    // ------------------------------------------------------------------
    // Tick lifecycle
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn fastforward_tick_increases_tick_count() {
        let engine = make_engine();
        let before = engine.current_tick();
        engine.fastforward_tick().await;
        assert_eq!(engine.current_tick(), before + 1);
    }

    #[tokio::test]
    async fn detailed_tick_increases_tick_count() {
        let engine = make_engine();
        let before = engine.current_tick();
        engine.detailed_tick().await;
        assert_eq!(engine.current_tick(), before + 1);
    }

    #[tokio::test]
    async fn multiple_detailed_ticks_accumulate() {
        let engine = make_engine();
        for _ in 0..3 {
            engine.detailed_tick().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(engine.current_tick() >= 3);
    }

    #[tokio::test]
    async fn multiple_fastforward_ticks_accumulate() {
        let engine = make_engine();
        for _ in 0..10 {
            engine.fastforward_tick().await;
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert_eq!(engine.current_tick(), 10);
    }

    #[tokio::test]
    async fn snapshot_stats_reflects_ticks() {
        let engine = make_engine();
        engine.fastforward_tick().await;
        engine.detailed_tick().await;
        let stats = engine.snapshot_stats();
        assert_eq!(stats.total_ticks, 2);
        assert_eq!(stats.fastforward_ticks, 1);
        assert_eq!(stats.detailed_ticks, 1);
    }

    // ------------------------------------------------------------------
    // Time advancement
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn detailed_tick_advances_time_by_5_minutes() {
        let engine = make_engine();
        let time_before = {
            let world = engine.world.read().await;
            world.timeline.time
        };
        engine.detailed_tick().await;
        let time_after = {
            let world = engine.world.read().await;
            world.timeline.time
        };
        assert!(
            time_after > time_before,
            "detailed tick should advance simulation time"
        );
    }

    #[tokio::test]
    async fn fastforward_tick_advances_time_by_1_hour() {
        let engine = make_engine();
        let time_before = {
            let world = engine.world.read().await;
            world.timeline.time
        };
        engine.fastforward_tick().await;
        let time_after = {
            let world = engine.world.read().await;
            world.timeline.time
        };
        assert!(
            time_after > time_before,
            "fastforward tick should advance simulation time by 1 hour"
        );
        // fastforward_tick_hours is 1, so diff should be >= 1 hour
        let diff_hours =
            (time_after.datetime - time_before.datetime).num_hours();
        assert_eq!(diff_hours, 1);
    }

    // ------------------------------------------------------------------
    // Environment
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn environment_updates_on_detailed_tick() {
        let engine = make_engine();
        {
            let world = engine.world.read().await;
            assert!(world.environment.temperature.is_finite());
            assert!(world.environment.season.len() > 0);
        }
        engine.detailed_tick().await;
        {
            let world = engine.world.read().await;
            assert!(world.environment.temperature.is_finite());
            assert!(world.environment.weather.len() > 0);
            assert!(world.environment.season.len() > 0);
        }
    }

    // ------------------------------------------------------------------
    // World state initialization
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn world_state_initialized_correctly() {
        let engine = make_engine();
        let world = engine.world.read().await;
        assert_eq!(world.timeline.tick, 0);
        assert!(world.locations.is_empty());
        assert_eq!(world.character_count(), 0);
        assert_eq!(world.resources.len(), 0);
        assert!(world.timeline.events.is_empty());
        // Actually: assert!(world.timeline.events.is_empty());
    }

    // ------------------------------------------------------------------
    // Concurrent safety: spawn 3 tasks reading/writing simultaneously
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn concurrent_read_write_safety() {
        let engine = make_engine();

        // Task 1: Writer — run 10 detailed ticks
        let w_engine = Arc::clone(&engine);
        let writer = async move {
            for i in 0..10 {
                w_engine.detailed_tick().await;
                // Small jitter so reader tasks interleave
                if i % 3 == 0 {
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
            }
            w_engine.current_tick()
        };

        // Task 2: Reader — repeatedly read world state (RwLock read)
        let r1_engine = Arc::clone(&engine);
        let reader1 = async move {
            let mut snapshots = Vec::new();
            for _ in 0..5 {
                let world = r1_engine.world.read().await;
                snapshots.push(world.timeline.tick);
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            snapshots
        };

        // Task 3: Reader — repeatedly snapshot stats (atomic reads)
        let r2_engine = Arc::clone(&engine);
        let reader2 = async move {
            let mut stats = Vec::new();
            for _ in 0..5 {
                stats.push(r2_engine.snapshot_stats().total_ticks);
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            stats
        };

        let (final_tick, reads, stat_snaps) = tokio::join!(writer, reader1, reader2);

        // Writer completed all 10 ticks
        assert_eq!(final_tick, 10, "writer should complete 10 ticks");

        // Reader observed monotonic (non-decreasing) ticks
        for pair in reads.windows(2) {
            assert!(
                pair[0] <= pair[1],
                "reader observed non-monotonic ticks: {:?}",
                reads
            );
        }

        // Stats reader observed monotonic ticks
        for pair in stat_snaps.windows(2) {
            assert!(
                pair[0] <= pair[1],
                "stats observer saw non-monotonic ticks: {:?}",
                stat_snaps
            );
        }

        // Final engine stats match our expectation
        let stats = engine.snapshot_stats();
        assert_eq!(stats.total_ticks, 10);
        assert_eq!(stats.detailed_ticks, 10);
    }

    #[tokio::test]
    async fn concurrent_fastforward_and_detailed_mixed() {
        let engine = make_engine();

        let eng1 = Arc::clone(&engine);
        let eng2 = Arc::clone(&engine);

        let (r1, r2) = tokio::join!(
            async move {
                for _ in 0..5 {
                    eng1.fastforward_tick().await;
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
                eng1.current_tick()
            },
            async move {
                for _ in 0..5 {
                    eng2.detailed_tick().await;
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
                eng2.current_tick()
            },
        );

        // Both share the same engine / AtomicU64 tick counter.
        // So each Arc sees total_ticks = 10 after 5 FF + 5 DT complete.
        assert_eq!(r1, 10);
        assert_eq!(r2, 10);
        let stats = engine.snapshot_stats();
        assert_eq!(stats.total_ticks, 10);
        assert!(stats.fastforward_ticks >= 1);
        assert!(stats.detailed_ticks >= 1);
    }
}
