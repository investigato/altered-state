use anyhow::Result;
use axum::{Router, routing::get};
use tower_http::services::ServeDir;

use crate::{config::app::AppConfig,cleanup_crew::serve::ServerState, web::api};

pub async fn run(config: AppConfig,state:ServerState, port: u16) -> Result<()> {
	tracing::info!("Retcon web UI listening on http://127.0.0.1:{port}");
    let app = Router::new()
		.fallback_service(ServeDir::new("wwwroot")).merge(api::router(config,state));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;

    tracing::info!("Retcon web UI listening on http://127.0.0.1:{port}");

    axum::serve(listener, app).await?;

    Ok(())
}
