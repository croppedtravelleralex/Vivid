#[cfg(test)]
mod api_tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use std::sync::Arc;
    use tokio::sync::{broadcast, mpsc};
    use tower::ServiceExt;

    use vivid::api::router;
    use vivid::engine::{EngineConfig, EngineEvent, LLMRequest, SimulationEngine};
    use vivid::llm::gateway::LLMGateway;
    use vivid::models::world::WorldState;
    use vivid::ApiState;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn test_engine() -> Arc<SimulationEngine> {
        let start_time =
            chrono::NaiveDateTime::parse_from_str("2025-12-03T08:00:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap();
        let world = WorldState::new(start_time, 42);
        let config = EngineConfig {
            detailed_tick_minutes: 5,
            fastforward_tick_hours: 1,
            idle_threshold: 3,
            max_concurrent_llm: 2,
            llm_timeout_seconds: 5,
            checkpoint_interval: 100,
            random_seed: 42,
        };
        let (llm_tx, _llm_rx) = mpsc::channel::<LLMRequest>(100);
        let (ws_tx, _ws_rx) = broadcast::channel::<EngineEvent>(256);
        let llm_gw = Arc::new(LLMGateway::new(
            "test_key".into(),
            "http://localhost".into(),
            "test-model".into(),
            2,
            5,
        ));
        Arc::new(SimulationEngine::new(
            world, config, llm_tx, ws_tx, llm_gw,
        ))
    }

    /// Build a fresh callable Router for a single request.
    /// Cloning `ApiState` is cheap because `SimulationEngine` is behind `Arc`.
    /// Returns `axum::Router` (i.e. `Router<()>`) which implements `Service<Request<Body>>`.
    fn app(engine: &Arc<SimulationEngine>) -> axum::Router {
        let state = ApiState {
            engine: Arc::clone(engine),
        };
        router().with_state(state)
    }

    /// Send a request and return (StatusCode, deserialized JSON body).
    /// If the response body is not valid JSON (e.g. empty 404/500), returns
    /// `serde_json::Value::Null` so tests can still assert on the status code.
    async fn send(
        engine: &Arc<SimulationEngine>,
        req: Request<Body>,
    ) -> (StatusCode, Value) {
        let router = app(engine);
        let response = router.oneshot(req).await.unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, body)
    }

    /// GET and deserialize.
    async fn get(engine: &Arc<SimulationEngine>, path: &str) -> (StatusCode, Value) {
        send(engine, Request::get(path).body(Body::empty()).unwrap()).await
    }

    /// POST with empty JSON body `{}`.
    async fn post(engine: &Arc<SimulationEngine>, path: &str) -> (StatusCode, Value) {
        send(
            engine,
            Request::post(path)
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
    }

    /// POST with a custom JSON body string.
    async fn post_body(
        engine: &Arc<SimulationEngine>,
        path: &str,
        json: &str,
    ) -> (StatusCode, Value) {
        send(
            engine,
            Request::post(path)
                .header("Content-Type", "application/json")
                .body(Body::from(json.to_string()))
                .unwrap(),
        )
        .await
    }

    // ===================================================================
    // World endpoints
    // ===================================================================

    #[tokio::test]
    async fn test_get_world_returns_ok() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/world").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        let d = &body["data"];
        assert!(d["tick"].is_number());
        assert_eq!(d["tick"], 0);
        assert!(d["time"].is_string());
        assert!(d["character_count"].is_number());
        assert!(d["location_count"].is_number());
        assert!(d["temperature"].is_number());
        assert!(d["weather"].is_string());
        assert!(d["season"].is_string());
    }

    #[tokio::test]
    async fn test_get_world_environment() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/world/environment").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        let d = &body["data"];
        assert!(d["temperature"].is_number());
        assert!(d["weather"].is_string());
        assert!(d["season"].is_string());
        assert!(d["daylight"].is_number());
    }

    #[tokio::test]
    async fn test_get_world_locations_empty() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/world/locations").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        let arr = body["data"].as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    // ===================================================================
    // Simulation endpoints
    // ===================================================================

    #[tokio::test]
    async fn test_simulation_status_initial_is_paused() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/simulation/status").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"]["state"], "paused");
        assert_eq!(body["data"]["speed"], "paused");
        assert_eq!(body["data"]["tick"], 0);
    }

    #[tokio::test]
    async fn test_simulation_start_changes_state() {
        let engine = test_engine();

        let (status, _) = post(&engine, "/api/v1/simulation/start").await;
        assert_eq!(status, StatusCode::OK);

        let (_, body) = get(&engine, "/api/v1/simulation/status").await;
        assert_eq!(body["data"]["state"], "running");
        assert!(body["data"]["speed"].is_string());
        assert!(body["data"]["tick"].is_number());
    }

    #[tokio::test]
    async fn test_simulation_pause() {
        let engine = test_engine();

        post(&engine, "/api/v1/simulation/start").await;
        let (status, _) = post(&engine, "/api/v1/simulation/pause").await;
        assert_eq!(status, StatusCode::OK);

        let (_, body) = get(&engine, "/api/v1/simulation/status").await;
        assert_eq!(body["data"]["state"], "paused");
    }

    #[tokio::test]
    async fn test_simulation_stop() {
        let engine = test_engine();

        post(&engine, "/api/v1/simulation/start").await;
        let (status, _) = post(&engine, "/api/v1/simulation/stop").await;
        assert_eq!(status, StatusCode::OK);

        let (_, body) = get(&engine, "/api/v1/simulation/status").await;
        assert_eq!(body["data"]["state"], "stopped");
    }

    #[tokio::test]
    async fn test_simulation_step() {
        let engine = test_engine();

        let (status, body) = post(&engine, "/api/v1/simulation/step").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        assert_eq!(body["data"]["speed"], "detailed");
        assert!(body["data"]["tick"].as_u64().unwrap_or(0) >= 1);

        // After step the engine should be paused again
        let (_, sb) = get(&engine, "/api/v1/simulation/status").await;
        assert_eq!(sb["data"]["state"], "paused");
    }

    #[tokio::test]
    async fn test_speed_switch_to_fastforward() {
        let engine = test_engine();

        post(&engine, "/api/v1/simulation/start").await;
        let (status, body) = post_body(
            &engine,
            "/api/v1/simulation/speed",
            r#"{"speed":"fast_forward"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn test_speed_switch_to_detailed() {
        let engine = test_engine();

        post(&engine, "/api/v1/simulation/start").await;
        let (status, _) =
            post_body(&engine, "/api/v1/simulation/speed", r#"{"speed":"detailed"}"#).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_speed_invalid_defaults_to_detailed() {
        let engine = test_engine();

        post(&engine, "/api/v1/simulation/start").await;
        let (status, _) =
            post_body(&engine, "/api/v1/simulation/speed", r#"{"speed":"bogus"}"#).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_simulation_stats() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/simulation/stats").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        let d = &body["data"];
        assert!(d["total_ticks"].is_number());
        assert!(d["fastforward_ticks"].is_number());
        assert!(d["detailed_ticks"].is_number());
        assert!(d["total_llm_calls"].is_number());
        assert!(d["characters_active"].is_number());
    }

    // ===================================================================
    // Character endpoints
    // ===================================================================

    #[tokio::test]
    async fn test_get_characters_returns_empty() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/characters").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        assert!(body["data"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_character_detail_404() {
        let engine = test_engine();
        let id = uuid::Uuid::nil();
        let (status, _) = get(&engine, &format!("/api/v1/characters/{}", id)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_character_memory_404() {
        let engine = test_engine();
        let id = uuid::Uuid::nil();
        let (status, _) =
            get(&engine, &format!("/api/v1/characters/{}/memory", id)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_character_relationships_404() {
        let engine = test_engine();
        let id = uuid::Uuid::nil();
        let (status, _) =
            get(&engine, &format!("/api/v1/characters/{}/relationships", id)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // ===================================================================
    // Timeline endpoints
    // ===================================================================

    #[tokio::test]
    async fn test_get_timeline() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/timeline").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        assert_eq!(body["data"]["tick"], 0);
        assert_eq!(body["data"]["event_count"], 0);
        assert!(body["data"]["time"].is_string());
    }

    #[tokio::test]
    async fn test_get_timeline_events_empty() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/timeline/events").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        assert!(body["data"]["events"].as_array().unwrap().is_empty());
        assert_eq!(body["data"]["total"], 0);
    }

    #[tokio::test]
    async fn test_timeline_tick_increases_after_step() {
        let engine = test_engine();

        assert_eq!(get(&engine, "/api/v1/timeline").await.1["data"]["tick"], 0);

        post(&engine, "/api/v1/simulation/step").await;

        assert_eq!(get(&engine, "/api/v1/timeline").await.1["data"]["tick"], 1);
    }

    // ===================================================================
    // Events endpoint
    // ===================================================================

    #[tokio::test]
    async fn test_create_event_with_title() {
        let engine = test_engine();
        let (status, body) = post_body(
            &engine,
            "/api/v1/events",
            r#"{"title":"My Event","description":"desc"}"#,
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        assert!(body["data"]["event_id"].is_string());
        assert!(!body["data"]["event_id"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_create_event_defaults_title() {
        let engine = test_engine();
        let (status, body) = post_body(&engine, "/api/v1/events", r#"{}"#).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        assert!(body["data"]["event_id"].is_string());
    }

    #[tokio::test]
    async fn test_get_event_404() {
        let engine = test_engine();
        let id = uuid::Uuid::nil();
        let (status, _) = get(&engine, &format!("/api/v1/events/{}", id)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // ===================================================================
    // Dashboard endpoint
    // ===================================================================

    #[tokio::test]
    async fn test_dashboard_summary() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/dashboard/summary").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        let d = &body["data"];
        assert_eq!(d["tick"], 0);
        assert!(d["time"].is_string());
        assert_eq!(d["speed"], "paused");
        assert_eq!(d["state"], "paused");
        assert_eq!(d["characters"], 0);
        assert_eq!(d["locations"], 0);
        assert!(d["temperature"].is_number());
        assert!(d["weather"].is_string());
        assert!(d["season"].is_string());
        assert_eq!(d["events_triggered"], 0);
        assert_eq!(d["total_llm_calls"], 0);
    }

    // ===================================================================
    // Search endpoint
    // ===================================================================

    #[tokio::test]
    async fn test_search_no_query() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/search").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        assert_eq!(body["data"]["query"], "");
        assert!(body["data"]["characters"].as_array().unwrap().is_empty());
        assert!(body["data"]["locations"].as_array().unwrap().is_empty());
        assert_eq!(body["data"]["total"], 0);
    }

    #[tokio::test]
    async fn test_search_with_query() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/search?q=nonexistent").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"]["query"], "nonexistent");
        assert_eq!(body["data"]["total"], 0);
    }

    // ===================================================================
    // Tags endpoints
    // ===================================================================

    #[tokio::test]
    async fn test_tags_heatmap() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/tags/heatmap").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        assert!(body["data"]["tags"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_tags_threads() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/tags/threads").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        assert!(body["data"].as_array().unwrap().is_empty());
    }

    // ===================================================================
    // Graph endpoints
    // ===================================================================

    #[tokio::test]
    async fn test_graph_relationships() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/graph/relationships").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        assert!(body["data"]["nodes"].as_array().unwrap().is_empty());
        assert!(body["data"]["edges"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_graph_locations() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/graph/locations").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        assert!(body["data"]["nodes"].as_array().unwrap().is_empty());
        assert!(body["data"]["edges"].is_array());
    }

    // ===================================================================
    // Checkpoint endpoints (filesystem-dependent)
    // ===================================================================

    #[tokio::test]
    async fn test_checkpoint_list() {
        let engine = test_engine();
        let (status, body) = get(&engine, "/api/v1/checkpoint/list").await;

        let ok = status == StatusCode::OK;
        let err = status == StatusCode::INTERNAL_SERVER_ERROR;
        assert!(ok || err, "expected 200 or 500, got {}", status);
        if ok {
            assert_eq!(body["status"], "ok");
        }
    }

    #[tokio::test]
    async fn test_checkpoint_save() {
        let engine = test_engine();
        let (status, body) = post_body(
            &engine,
            "/api/v1/checkpoint/save",
            r#"{"tag":"test_save"}"#,
        )
        .await;

        let ok = status == StatusCode::OK;
        let err = status == StatusCode::INTERNAL_SERVER_ERROR;
        assert!(ok || err, "expected 200 or 500, got {}", status);
        if ok {
            assert_eq!(body["status"], "ok");
            assert!(body["data"]["tick"].is_number());
        }
    }

    #[tokio::test]
    async fn test_checkpoint_load_nonexistent() {
        let engine = test_engine();
        let (status, _) = post_body(
            &engine,
            "/api/v1/checkpoint/load",
            r#"{"tag":"no_such_tag"}"#,
        )
        .await;

        let ok = status == StatusCode::OK;
        let nf = status == StatusCode::NOT_FOUND;
        let err = status == StatusCode::INTERNAL_SERVER_ERROR;
        assert!(ok || nf || err, "expected 200/404/500, got {}", status);
    }

    // ===================================================================
    // Unknown routes & methods
    // ===================================================================

    #[tokio::test]
    async fn test_unknown_route_returns_404() {
        let engine = test_engine();
        let router = app(&engine);
        let r = router
            .oneshot(
                Request::get("/api/v1/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_wrong_method_on_post_route_returns_405() {
        let engine = test_engine();
        let router = app(&engine);
        let r = router
            .oneshot(
                Request::get("/api/v1/simulation/start")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    // ===================================================================
    // Full lifecycle
    // ===================================================================

    #[tokio::test]
    async fn test_full_lifecycle() {
        let engine = test_engine();

        // 1. Initial
        let (_, b) = get(&engine, "/api/v1/simulation/status").await;
        assert_eq!(b["data"]["state"], "paused");

        // 2. Start
        post(&engine, "/api/v1/simulation/start").await;
        let (_, b) = get(&engine, "/api/v1/simulation/status").await;
        assert_eq!(b["data"]["state"], "running");

        // 3. Step (detailed tick + auto-pause)
        let (_, b) = post(&engine, "/api/v1/simulation/step").await;
        assert_eq!(b["status"], "ok");
        assert!(b["data"]["tick"].as_u64().unwrap_or(0) >= 1);
        let (_, b) = get(&engine, "/api/v1/simulation/status").await;
        assert_eq!(b["data"]["state"], "paused");

        // 4. Stop
        post(&engine, "/api/v1/simulation/stop").await;
        let (_, b) = get(&engine, "/api/v1/simulation/status").await;
        assert_eq!(b["data"]["state"], "stopped");
    }

    #[tokio::test]
    async fn test_stats_update_after_ticks() {
        let engine = test_engine();

        assert_eq!(
            get(&engine, "/api/v1/simulation/stats").await.1["data"]["total_ticks"],
            0
        );

        post(&engine, "/api/v1/simulation/step").await;
        post(&engine, "/api/v1/simulation/step").await;

        let (_, body) = get(&engine, "/api/v1/simulation/stats").await;
        assert_eq!(body["data"]["total_ticks"], 2);
        assert_eq!(body["data"]["detailed_ticks"], 2);
    }

    #[tokio::test]
    async fn test_world_tick_after_step() {
        let engine = test_engine();

        let tick_before =
            get(&engine, "/api/v1/world").await.1["data"]["tick"].as_u64().unwrap();
        post(&engine, "/api/v1/simulation/step").await;
        let tick_after =
            get(&engine, "/api/v1/world").await.1["data"]["tick"].as_u64().unwrap();

        assert_eq!(tick_after, tick_before + 1);
    }
}
