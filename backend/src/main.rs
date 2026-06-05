use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc};
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};
use vivid::api::router;
use vivid::config::Config;
use vivid::engine::simulation_loop::run_simulation;
use vivid::engine::{EngineConfig, EngineEvent, LLMRequest, SimulationEngine};
use vivid::llm::gateway::LLMGateway;
use vivid::models::world::WorldState;
use vivid::storage::checkpoint::CrashRecovery;
use vivid::telemetry;
use vivid::ApiState;

#[tokio::main]
async fn main() {
    // 1. Load .env
    dotenvy::dotenv().ok();

    // 2. Init logging
    telemetry::init_logging();
    info!("Vivid 小说模拟引擎 v0.1.0 启动中...");

    // 3. Load config
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "data/config.yaml".into());
    let config = match Config::from_file(&config_path) {
        Ok(cfg) => {
            info!("配置已加载: {}", config_path);
            cfg
        }
        Err(errors) => {
            for e in &errors {
                error!("配置错误: {}", e);
            }
            std::process::exit(1);
        }
    };

    // 4. Crash recovery check
    let crashed = CrashRecovery::check("data/checkpoints").await;
    if crashed {
        warn!("引擎上次异常退出，已从最新检查点恢复");
    }

    // 5. Create LLM gateway
    let llm_gateway = Arc::new(LLMGateway::new(
        config.llm.api_key.clone(),
        config.llm.base_url.clone(),
        config.llm.model.clone(),
        config.llm.max_concurrent,
        config.llm.timeout_seconds,
    ));
    info!(
        "LLM 网关已初始化: {} (并发 {})",
        config.llm.model, config.llm.max_concurrent
    );

    // 6. Create broadcast channel (WS events)
    let (ws_tx, _ws_rx) = broadcast::channel::<EngineEvent>(256);

    // 7. Create mpsc channel (LLM requests)
    let (llm_tx, _llm_rx) = mpsc::channel::<LLMRequest>(100);

    // 8. Load world state
    let engine_cfg = EngineConfig {
        detailed_tick_minutes: config.engine.detailed_tick_minutes,
        fastforward_tick_hours: config.engine.fastforward_tick_hours,
        idle_threshold: config.engine.idle_threshold,
        max_concurrent_llm: config.llm.max_concurrent,
        llm_timeout_seconds: config.llm.timeout_seconds,
        checkpoint_interval: config.checkpoint.interval,
        random_seed: config.engine.random_seed,
    };

    let world = match WorldState::load_from_files(
        "data/characters",
        "data/world/locations.json",
        "data/world/environment.json",
        "data/plot/events.json",
        engine_cfg.random_seed,
    ) {
        Ok(w) => {
            info!(
                "世界状态已加载: {} 角色, {} 地点",
                w.character_count(),
                w.location_count()
            );
            w
        }
        Err(errors) => {
            for e in &errors {
                error!("世界加载错误: {}", e);
            }
            std::process::exit(1);
        }
    };

    // 9. Create engine (starts in Paused state)
    let engine = Arc::new(SimulationEngine::new(
        world,
        engine_cfg,
        llm_tx,
        ws_tx,
        llm_gateway,
    ));
    info!("模拟引擎已创建 (Paused)");

    // 10. Spawn the simulation loop
    let engine_clone = engine.clone();
    tokio::spawn(async move {
        run_simulation(engine_clone).await;
    });

    // 11. Build router
    let state = ApiState {
        engine: engine.clone(),
    };
    let app = router().with_state(state).layer(CorsLayer::permissive());

    // 12. Graceful shutdown
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

    // SIGTERM (Unix)
    #[cfg(unix)]
    {
        let tx = shutdown_tx.clone();
        tokio::spawn(async move {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to register SIGTERM handler")
                .recv()
                .await;
            info!("收到 SIGTERM，正在关闭...");
            tx.send(()).await.ok();
        });
    }

    // SIGINT
    {
        let tx = shutdown_tx.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to register Ctrl+C handler");
            info!("收到 Ctrl+C，正在关闭...");
            tx.send(()).await.ok();
        });
    }

    // 13. Bind and serve
    let addr = "0.0.0.0:3000";
    info!("API 服务器启动于 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_rx.recv().await;
            info!("执行优雅关闭...");
            tokio::time::sleep(Duration::from_millis(500)).await;
            info!("引擎已停止");
        })
        .await
        .expect("Server error");

    info!("Vivid 引擎已关闭");
}
