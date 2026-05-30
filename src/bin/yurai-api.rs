//! HTTP API server wrapping the yurai investigation engine.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
};
use reqwest_middleware::ClientWithMiddleware;
use tokio::sync::{RwLock, Semaphore};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::Level;
use yurai::ModelInvestigation;

const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

struct AppState {
    /// Shared HTTP client for connection pooling across requests.
    client: ClientWithMiddleware,
    /// Cap concurrent investigations to avoid overwhelming HF.
    semaphore: Semaphore,
    /// In-memory TTL cache keyed by model id.
    cache: RwLock<HashMap<String, (Instant, ModelInvestigation)>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cors = if let Ok(origin) = std::env::var("CORS_ORIGIN") {
        CorsLayer::new()
            .allow_origin(
                origin
                    .parse::<axum::http::HeaderValue>()
                    .expect("invalid CORS_ORIGIN"),
            )
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let client = yurai::build_client().expect("failed to build HTTP client");

    let state = Arc::new(AppState {
        client,
        semaphore: Semaphore::new(4),
        cache: RwLock::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/api/investigate/{org}/{model}", get(investigate))
        .route("/api/health", get(health))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    eprintln!("yurai-api listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn investigate(
    State(state): State<Arc<AppState>>,
    Path((org, model)): Path<(String, String)>,
) -> Response {
    let model_id = format!("{org}/{model}");

    // Check cache first (no semaphore needed for reads).
    {
        let cache = state.cache.read().await;
        if let Some((inserted, inv)) = cache.get(&model_id)
            && inserted.elapsed() < CACHE_TTL
        {
            return Json(inv.clone()).into_response();
        }
    }

    let _permit = match state.semaphore.acquire().await {
        Ok(p) => p,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    // Re-check after acquiring permit (another request may have populated it).
    {
        let cache = state.cache.read().await;
        if let Some((inserted, inv)) = cache.get(&model_id)
            && inserted.elapsed() < CACHE_TTL
        {
            return Json(inv.clone()).into_response();
        }
    }

    match yurai::investigate_with_client(&state.client, &model_id).await {
        Ok(inv) => {
            let resp = Json(inv.clone()).into_response();
            let mut cache = state.cache.write().await;
            cache.insert(model_id, (Instant::now(), inv));
            cache.retain(|_, (inserted, _)| inserted.elapsed() < CACHE_TTL);
            resp
        }
        Err(yurai::InvestigationError::ModelNotFound(msg)) => {
            (StatusCode::NOT_FOUND, msg).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
