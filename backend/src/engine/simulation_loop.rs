use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use tracing::{info, span, Level};

use crate::engine::*;
use crate::models::event::EventSummary;
use crate::models::timeline::SimSpeed;

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

pub async fn run_simulation(engine: Arc<SimulationEngine>) {
    info!("模拟循环启动 (Paused 等待指令)");

    loop {
        let action = {
            let state = engine.state.lock().unwrap();
            match &*state {
                EngineState::Stopped => Action::Stop,
                EngineState::Paused => Action::WaitPaused,
                EngineState::Running { speed, .. } => match speed {
                    SimSpeed::FastForward => Action::RunFF,
                    SimSpeed::Detailed => Action::RunDetailed,
                    SimSpeed::Paused => Action::WaitPaused,
                },
            }
        };

        match action {
            Action::Stop => break,
            Action::WaitPaused => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Action::RunFF => {
                engine.fastforward_tick().await;
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Action::RunDetailed => {
                engine.detailed_tick().await;
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }

    info!("模拟循环退出");
}

enum Action {
    Stop,
    WaitPaused,
    RunFF,
    RunDetailed,
}

// ---------------------------------------------------------------------------
// Tick implementations
// ---------------------------------------------------------------------------

impl SimulationEngine {
    pub async fn fastforward_tick(self: &Arc<Self>) {
        let span = span!(Level::INFO, "ff_tick", tick = self.current_tick());
        let _guard = span.enter();

        let hours = { self.config.read().await.fastforward_tick_hours };

        // Phase 1: Advance time
        let (current_tick, current_time) = {
            let mut world = self.world.write().await;
            world.timeline.advance_hours(hours);
            let tick = world.timeline.tick;
            let time = world.timeline.time;
            world.environment.update(tick, &time);
            (tick, time)
        };

        // Phase 2: Resource consumption
        {
            let mut world = self.world.write().await;
            for loc in &mut world.locations {
                for (_k, stock) in &mut loc.resources {
                    stock.current =
                        (stock.current - stock.daily_consumption / 24.0 * hours as f64).max(0.0);
                }
            }
        }

        // Phase 3: Check due events
        let trigger_count = {
            let mut world = self.world.write().await;
            let mut count = 0u64;
            while let Some(event) = world
                .event_system
                .queue
                .pop_due(&current_time)
            {
                count += 1;
                world.event_system.queue.archive(
                    crate::models::event::HistoricalEvent {
                        id: event.id,
                        tick: current_tick,
                        sim_time: current_time.datetime,
                        event_type: event.event_type.api_name().to_string(),
                        title: event.title,
                        description: event.description,
                        severity: format!("{:?}", event.severity),
                        participants: vec![],
                        location_id: None,
                    },
                );
            }
            count
        };

        let time_str = current_time.to_string();
        let _ = self.ws_broadcaster.send(EngineEvent::Tick {
            tick: self.current_tick(),
            time: time_str,
            speed: SimSpeed::FastForward,
        });

        self.stats.fastforward_ticks.fetch_add(1, Ordering::Relaxed);
        self.stats.total_ticks.fetch_add(1, Ordering::Relaxed);
        self.stats
            .events_triggered
            .fetch_add(trigger_count, Ordering::Relaxed);

        let interval = self.config.read().await.checkpoint_interval;
        if self.current_tick() > 0 && self.current_tick() % interval == 0 {
            info!(tick = self.current_tick(), "检查点触发");
        }
    }

    pub async fn detailed_tick(self: &Arc<Self>) {
        let span = span!(Level::INFO, "detailed_tick", tick = self.current_tick());
        let _guard = span.enter();

        let minutes = { self.config.read().await.detailed_tick_minutes };

        // Phase 1: Advance time
        let (current_tick, current_time) = {
            let mut world = self.world.write().await;
            world.timeline.advance_minutes(minutes);
            let tick = world.timeline.tick;
            let time = world.timeline.time;
            world.environment.update(tick, &time);
            (tick, time)
        };

        // Phase 2: Determine which characters need LLM
        let need_llm: Vec<uuid::Uuid> = {
            let world = self.world.read().await;
            world
                .characters
                .iter()
                .filter_map(|entity_ref| {
                    if entity_ref
                        .get::<&crate::models::world::CharacterState>()
                        .is_some()
                        && rand::thread_rng().gen_bool(0.3)
                    {
                        entity_ref.get::<&uuid::Uuid>().map(|r| *r)
                    } else {
                        None
                    }
                })
                .collect()
        };

        if need_llm.is_empty() {
            self.consecutive_idle.fetch_add(1, Ordering::Relaxed);
            let threshold = { self.config.read().await.idle_threshold };
            if self.consecutive_idle.load(Ordering::Relaxed) >= threshold {
                info!(
                    "连续 {} tick 无行动",
                    self.consecutive_idle.load(Ordering::Relaxed)
                );
            }
        } else {
            self.consecutive_idle.store(0, Ordering::Relaxed);
            info!(count = need_llm.len(), "LLM 决策阶段");
            self.stats
                .total_llm_calls
                .fetch_add(need_llm.len() as u64, Ordering::Relaxed);
        }

        // Phase 3: Resource consumption
        {
            let mut world = self.world.write().await;
            for loc in &mut world.locations {
                for (_k, stock) in &mut loc.resources {
                    stock.current = (stock.current - stock.daily_consumption / 288.0).max(0.0);
                }
            }
        }

        // Phase 4: Check due events
        let mut summaries = vec![];
        {
            let mut world = self.world.write().await;
            while let Some(event) = world
                .event_system
                .queue
                .pop_due(&current_time)
            {
                summaries.push(EventSummary {
                    event_id: event.id,
                    event_type: event.event_type.api_name().to_string(),
                    title: event.title.clone(),
                    severity: format!("{:?}", event.severity),
                });
                world.event_system.queue.archive(
                    crate::models::event::HistoricalEvent {
                        id: event.id,
                        tick: current_tick,
                        sim_time: current_time.datetime,
                        event_type: event.event_type.api_name().to_string(),
                        title: event.title,
                        description: event.description,
                        severity: format!("{:?}", event.severity),
                        participants: vec![],
                        location_id: None,
                    },
                );
            }
        }

        // Council every 50 ticks
        if self.current_tick() > 0 && self.current_tick() % 50 == 0 {
            info!("叙事议会触发 (占位)");
        }

        let time_str = current_time.to_string();
        let _ = self.ws_broadcaster.send(EngineEvent::DetailedTick {
            tick: self.current_tick(),
            time: time_str,
            decisions: vec![],
            events: summaries,
        });

        self.stats.detailed_ticks.fetch_add(1, Ordering::Relaxed);
        self.stats.total_ticks.fetch_add(1, Ordering::Relaxed);

        let interval = self.config.read().await.checkpoint_interval;
        if self.current_tick() > 0 && self.current_tick() % interval == 0 {
            info!(tick = self.current_tick(), "详细 tick 检查点");
        }
    }
}
