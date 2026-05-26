use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde_json::json;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;

use crate::{
    cleanup_crew::{
        activate::{self, ActivateRequest},
        reset::{self, ResetRequest},
        serve::ServerState,
    },
    models::scenario::ScenarioState,
};

#[derive(Clone)]
pub struct ApiState {
    pub server_state: Arc<RwLock<ServerState>>,
    pub operation_lock: Arc<Mutex<()>>,
}

pub fn router(server_state: Arc<RwLock<ServerState>>) -> Router {
    let state = ApiState {
        server_state,
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
    let server_state = state
        .server_state
        .read()
        .expect("server_state RwLock not good right now");
    let active_scenario = server_state.context.scenario_state.get_active_scenario();
    if let Some(scenario) = &active_scenario {
        tracing::info!("Active scenario: {}", scenario.scenario);
        Json(json!({ "ok": true, "active_scenario": scenario.scenario }))
    } else {
        tracing::info!("No active scenario");
        Json(json!({ "ok": true, "active_scenario": null }))
    }
}

async fn get_scenarios(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let _guard = state.operation_lock.lock().await;
    let server_state = state
        .server_state
        .read()
        .expect("server_state RwLock not good right now");
    let scenarios = server_state
        .scenarios
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
    let context = state
        .server_state
        .read()
        .expect("server_state RwLock not good right now")
        .context
        .clone();
    match activate::run(context, payload).await {
        Ok(_) => {
            let scenario_state_file = {
                let server_state = state.server_state.read().expect("RwLock poisoned");
                server_state
                    .context
                    .config
                    .paths
                    .scenario_state_file
                    .clone()
            };

            let new_scenario_state = ScenarioState::load(&scenario_state_file).await;

            {
                let mut server_state = state.server_state.write().expect("RwLock poisoned");
                server_state.context.scenario_state = new_scenario_state;
            }

            let server_state = state
                .server_state
                .read()
                .expect("server_state RwLock not good right now");
            let active_scenario = server_state.context.scenario_state.get_active_scenario();
            if let Some(scenario) = &active_scenario {
                tracing::info!("Active scenario: {}", scenario.scenario);
                Json(json!({ "ok": true, "active_scenario": scenario.scenario }))
            } else {
                tracing::info!("No active scenario");
                Json(json!({ "ok": true, "active_scenario": null }))
            }
        }
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

async fn reset_scenario(
    State(state): State<ApiState>,
    Json(payload): Json<ResetRequest>,
) -> Json<serde_json::Value> {
    let _guard = state.operation_lock.lock().await;
    let context = state
        .server_state
        .read()
        .expect("server_state RwLock not good right now")
        .context
        .clone();
    match reset::run(context, payload).await {
        Ok(_) => Json(json!({ "ok": true, "message": "scenario reset" })),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}
