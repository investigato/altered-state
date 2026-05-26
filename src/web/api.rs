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
    context::AppContext,
};

#[derive(Clone)]
pub struct ApiState {
    pub context: AppContext,
    pub server_state: Arc<RwLock<ServerState>>,
    pub operation_lock: Arc<Mutex<()>>,
}

pub fn router(context: AppContext, scenario_state: Arc<RwLock<ServerState>>) -> Router {
    let state = ApiState {
        context,
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
    let server_state = state
        .server_state
        .read()
        .expect("server_state RwLock not good right now");
    let active_scenario = server_state.context.scenario_state.get_active_scenario();
    Json(json!({ "ok": true, "active_scenario": active_scenario }))
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
    match activate::run(state.context, payload).await {
        Ok(_) => {
            let server_state = state
                .server_state
                .read()
                .expect("server_state RwLock not good right now");
            let active_scenario = server_state.context.scenario_state.get_active_scenario();
            Json(json!({ "ok": true, "active_scenario": active_scenario }))
        }
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

async fn reset_scenario(
    State(state): State<ApiState>,
    Json(payload): Json<ResetRequest>,
) -> Json<serde_json::Value> {
    let _guard = state.operation_lock.lock().await;
    match reset::run(state.context, payload).await {
        Ok(_) => Json(json!({ "ok": true, "message": "scenario reset" })),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}
