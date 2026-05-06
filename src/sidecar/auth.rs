//! Optional X-API-Key middleware. If SIDECAR_API_KEY env is unset,
//! middleware is a no-op (open access). If set, every request must
//! present a matching X-API-Key header.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::AppState;

pub async fn api_key_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for /health and /widgets.json (always public for monitors + manifests)
    let path = request.uri().path();
    if path == "/health" || path == "/widgets.json" {
        return Ok(next.run(request).await);
    }

    // No key configured = open access
    let Some(expected) = &state.api_key else {
        return Ok(next.run(request).await);
    };

    let provided = request
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());

    match provided {
        Some(p) if p == expected => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
