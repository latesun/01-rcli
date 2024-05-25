use anyhow::Result;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Serving {:?} on {}", path, addr);

    let state = HttpServeState { path };
    let router = axum::Router::new()
        .route("/*path", axum::routing::get(file_handler))
        .with_state(Arc::new(state));

    let lister = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(lister, router).await?;
    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> (StatusCode, String) {
    let p = std::path::Path::new(&state.path).join(path);
    info!("Reading file {:?}", p);
    if !p.exists() {
        return (StatusCode::NOT_FOUND, format!("File not found: {:?}", p));
    }

    match tokio::fs::read_to_string(p).await {
        Ok(content) => {
            info!("Read {} bytes", content.len());
            (StatusCode::OK, content)
        }
        Err(e) => {
            warn!("Error reading file: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}
