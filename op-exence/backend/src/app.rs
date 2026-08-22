use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, Method, StatusCode},
    routing::get,
};
use serde::Serialize;
use sqlx::PgPool;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::categories;
use crate::config::Config;
use crate::db;
use crate::expenses;
use crate::shops;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub async fn ready(State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    db::ping(&state.pool).await.map_err(|err| {
        tracing::warn!(error = %err, "readiness check failed");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    Ok(StatusCode::OK)
}

fn cors_layer(origin: &str) -> CorsLayer {
    let allowed = origin
        .parse::<HeaderValue>()
        .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:8081"));

    CorsLayer::new()
        .allow_origin(allowed)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([axum::http::header::CONTENT_TYPE])
}

pub fn build_router(state: AppState, cors_origin: &str) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .nest("/categories", categories::routes::routes())
        .nest("/shops", shops::routes::routes())
        .nest("/expenses", expenses::routes::routes())
        .layer(cors_layer(cors_origin))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let pool = db::create_pool(&config).await?;
    db::run_migrations(&pool).await?;

    let state = AppState { pool };
    let router = build_router(state, &config.cors_origin);

    let listener = tokio::net::TcpListener::bind(&config.addr()).await?;
    tracing::info!(addr = %config.addr(), "server listening");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
