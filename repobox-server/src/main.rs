mod db;
mod git;
mod routes;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use clap::Parser;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Parser)]
#[command(name = "repobox-server", about = "Git Smart HTTP server for repo.box")]
struct Cli {
    /// Listen address
    #[arg(short = 'b', long = "bind", default_value = "127.0.0.1:3456")]
    bind: SocketAddr,
    /// Repository storage directory
    #[arg(short = 'd', long = "data-dir", default_value = "/var/lib/repobox/repos")]
    data_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    data_dir: PathBuf,
    db_path: PathBuf,
}

impl AppState {
    fn new(data_dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("repobox.db");
        db::init(&db_path)?;
        Ok(Self { data_dir, db_path })
    }
}

fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(routes::router())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let state = Arc::new(AppState::new(cli.data_dir)?);
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(cli.bind).await?;

    info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
