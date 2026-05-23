use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde_json::json;
use std::sync::Arc;
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
    pub server_state: ServerState,
    pub operation_lock: Arc<Mutex<()>>,
}

pub fn router(config: AppConfig, scenario_state: ServerState) -> Router {
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
    let active_scenario = state.server_state.active_scenario.clone();
    Json(json!({ "ok": true, "active_scenario": active_scenario }))
}

async fn get_scenarios(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let _guard = state.operation_lock.lock().await;
    let scenarios = state
        .server_state
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
    match activate::run(state.config.clone(), payload).await {
        Ok(_) => Json(json!({ "ok": true, "message": "scenario activated" })),
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
