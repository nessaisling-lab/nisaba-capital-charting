//! Wave 8 — Pursuit Astro REST sidecar.
//!
//! HTTP server exposing dashboard + scraper data so external clients
//! (OpenBB Workspace, custom dashboards, browser-based tools) can
//! consume the same data the desktop dashboard uses.
//!
//! Routes:
//!   GET  /health
//!   GET  /providers                          — distinct providers
//!   GET  /providers/:p                       — series under one provider
//!   GET  /series/:p/:s                       — observations for series
//!   GET  /tickers                            — active tickers
//!   GET  /tickers/:t/prices                  — OHLCV history
//!   GET  /tickers/:t/lagrange                — Lagrange composite history
//!   GET  /tickers/:t/astro                   — astrology score snapshot
//!   GET  /tickers/:t/fundamentals            — latest fundamentals
//!   GET  /widgets.json                       — OpenBB Workspace manifest
//!
//! Run: `cargo run --bin sidecar`
//! Port: env SIDECAR_PORT (default 8765)
//! Auth: env SIDECAR_API_KEY (optional X-API-Key header check)

mod auth;
mod handlers;
mod widgets;

use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderValue, Method},
    routing::get,
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<sqlx::PgPool>,
    pub api_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "sidecar=info,tower_http=info".to_string()))
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL not set");
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await?;
    let pool = Arc::new(pool);

    let api_key = std::env::var("SIDECAR_API_KEY").ok().filter(|k| !k.is_empty());
    if api_key.is_some() {
        tracing::info!("API key auth enabled (X-API-Key header required)");
    } else {
        tracing::info!("API key auth disabled — set SIDECAR_API_KEY env to enable");
    }

    let state = AppState { pool, api_key };

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderName::from_static("x-api-key"),
        ]);

    let app = Router::new()
        .route("/health", get(health))
        .route("/widgets.json", get(widgets::manifest))
        .route("/providers", get(handlers::list_providers))
        .route("/providers/:provider", get(handlers::list_provider_series))
        .route("/series/:provider/:series_id", get(handlers::series_observations))
        .route("/tickers", get(handlers::list_tickers))
        .route("/tickers/:ticker/prices", get(handlers::ticker_prices))
        .route("/tickers/:ticker/lagrange", get(handlers::ticker_lagrange))
        .route("/tickers/:ticker/astro", get(handlers::ticker_astro))
        .route("/tickers/:ticker/fundamentals", get(handlers::ticker_fundamentals))
        .layer(axum::middleware::from_fn_with_state(state.clone(), auth::api_key_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let port: u16 = std::env::var("SIDECAR_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8765);
    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Pursuit Astro sidecar listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let db_ok = sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(state.pool.as_ref())
        .await
        .is_ok();
    axum::Json(serde_json::json!({
        "service": "pursuit-astro-sidecar",
        "version": env!("CARGO_PKG_VERSION"),
        "database": if db_ok { "ok" } else { "down" },
    }))
}

// re-export so HeaderValue is referenced
#[allow(dead_code)]
fn _unused(_h: HeaderValue) {}
