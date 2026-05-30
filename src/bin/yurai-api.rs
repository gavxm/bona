//! HTTP API server wrapping the yurai investigation engine.

use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
};
use reqwest_middleware::ClientWithMiddleware;
use tokio::sync::Semaphore;
use tower_http::cors::{Any, CorsLayer};

struct AppState {
    /// Shared HTTP client for connection pooling across requests.
    client: ClientWithMiddleware,
    /// Cap concurrent investigations to avoid overwhelming HF.
    semaphore: Semaphore,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let client = yurai::build_client().expect("failed to build HTTP client");

    let state = Arc::new(AppState {
        client,
        semaphore: Semaphore::new(4),
    });

    let app = Router::new()
        .route("/api/investigate/{org}/{model}", get(investigate))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    eprintln!("yurai-api listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn investigate(
    State(state): State<Arc<AppState>>,
    Path((org, model)): Path<(String, String)>,
) -> Response {
    let _permit = match state.semaphore.acquire().await {
        Ok(p) => p,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    let model_id = format!("{org}/{model}");

    match yurai::investigate_with_client(&state.client, &model_id).await {
        Ok(inv) => Json(inv).into_response(),
        Err(yurai::InvestigationError::ModelNotFound(msg)) => {
            (StatusCode::NOT_FOUND, msg).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("429") || msg.contains("rate") {
                (StatusCode::TOO_MANY_REQUESTS, msg).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }
}
