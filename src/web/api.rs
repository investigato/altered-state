use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde_json::json;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;

use crate::{
    cleanup_crew::serve::ServerState,
    cleanup_crew::{
        activate::{self, ActivateRequest},
        reset::{self, ResetRequest},
    },
    config::app::AppConfig,
    config::scenarios::ScenarioConfig,
};

#[derive(Clone)]
pub struct ApiState {
    pub config: AppConfig,
    pub server_state: Arc<RwLock<ServerState>>,
    pub operation_lock: Arc<Mutex<()>>,
}

pub fn router(config: AppConfig, scenario_state: Arc<RwLock<ServerState>>) -> Router {
    let state = ApiState {
        config,
        server_state: scenario_state,
        operation_lock: Arc::new(Mutex::new(())),
    };

    Router::new()
        .route("/api/health", get(health))
        .route("/api/scenarios/activate", post(activate_scenario))
        .route("/api/scenarios/active", get(get_active_scenario))
        .route("/api/scenarios", get(get_scenarios))
        .route("/api/scenarios/reset", post(reset_scenario))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "retcon"
    }))
}

async fn get_active_scenario(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let _guard = state.operation_lock.lock().await;
    let active_scenario = state
        .server_state
        .read()
        .expect("server_state RwLock not good right now")
        .active_scenario
        .clone();
    if let Some(ref scenario_name) = active_scenario {
        tracing::info!("Current active scenario: {}", scenario_name);
        Json(json!({ "ok": true, "active_scenario": active_scenario }))
    } else {
        tracing::info!("No active scenario");
        Json(json!({ "ok": true, "active_scenario": null }))
    }
}

async fn get_scenarios(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let _guard = state.operation_lock.lock().await;
    let scenarios = state
        .server_state
        .read()
        .expect("server_state RwLock not good right now")
        .scenarios
        .clone()
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "description": s.description,
                "image_path": s.image_path,
            })
        })
        .collect::<Vec<_>>();
    // convert to ScenarioResponseDto with name, description, and image_path

    Json(json!({ "ok": true, "scenarios": scenarios }))
}
async fn activate_scenario(
    State(state): State<ApiState>,
    Json(payload): Json<ActivateRequest>,
) -> Json<serde_json::Value> {
    let _guard = state.operation_lock.lock().await;
    let scenario_name = payload.scenario.clone();
    match activate::run(state.config.clone(), payload).await {
        Ok(_) => {
            state
                .server_state
                .write()
                .expect("RwLock poisoned")
                .active_scenario = Some(scenario_name.clone());
            Json(json!({ "ok": true, "active_scenario": scenario_name }))
        }
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

async fn reset_scenario(
    State(state): State<ApiState>,
    Json(payload): Json<ResetRequest>,
) -> Json<serde_json::Value> {
    let _guard = state.operation_lock.lock().await;
    match reset::run(state.config.clone(), payload).await {
        Ok(_) => Json(json!({ "ok": true, "message": "scenario reset" })),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}
