use anyhow::Result;
use axum::Router;
use std::sync::{Arc, RwLock};
use tower_http::services::ServeDir;

use crate::{cleanup_crew::serve::ServerState, config::app::AppConfig, web::api};

pub async fn run(config: AppConfig, state: Arc<RwLock<ServerState>>, port: u16) -> Result<()> {
    tracing::info!("Retcon web UI listening on http://0.0.0.0:{port}");
    
    let app = Router::new()
        .fallback_service(ServeDir::new(config.paths.web_directory.clone()).append_index_html_on_directories(true))
        .merge(api::router(config, state));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("Web server listening on http://0.0.0.0:{port}");
    println!("Press ctrl-c to exit");
    axum::serve(listener, app).await?;

    Ok(())
}
