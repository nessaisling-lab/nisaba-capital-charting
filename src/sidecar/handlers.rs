//! Wave 8.2 + 8.3 — REST handlers for provider + ticker data.
//!
//! Returns OpenBB-Workspace-compatible JSON. Standard envelope:
//!   { data: [...], meta: { ... } }
//! Plain arrays also accepted by Workspace; we use the envelope so
//! pagination / metadata can extend later.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::AppState;

// ── Observation endpoints (Wave 7 data) ──────────────────────────────

pub async fn list_providers(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT provider FROM provider_observations ORDER BY provider",
    )
    .fetch_all(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let providers: Vec<String> = rows.into_iter().map(|(p,)| p).collect();
    Ok(Json(serde_json::json!({ "data": providers })))
}

pub async fn list_provider_series(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows: Vec<(String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT DISTINCT series_id, label, region, unit
         FROM provider_observations
         WHERE provider = $1
         ORDER BY series_id, region",
    )
    .bind(&provider)
    .fetch_all(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let series: Vec<_> = rows.into_iter().map(|(s, l, r, u)| {
        serde_json::json!({
            "series_id": s, "label": l, "region": r, "unit": u,
        })
    }).collect();
    Ok(Json(serde_json::json!({ "provider": provider, "data": series })))
}

#[derive(Deserialize)]
pub struct SeriesQuery {
    pub region: Option<String>,
    pub limit: Option<i64>,
}

pub async fn series_observations(
    State(state): State<AppState>,
    Path((provider, series_id)): Path<(String, String)>,
    Query(q): Query<SeriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = q.limit.unwrap_or(500).min(5000);
    let rows: Vec<(chrono::NaiveDate, Option<f64>, Option<String>, Option<String>, Option<String>)> = if let Some(region) = &q.region {
        sqlx::query_as(
            "SELECT observation_date, value, label, region, unit
             FROM provider_observations
             WHERE provider = $1 AND series_id = $2 AND region = $3
             ORDER BY observation_date DESC
             LIMIT $4",
        )
        .bind(&provider)
        .bind(&series_id)
        .bind(region)
        .bind(limit)
        .fetch_all(state.pool.as_ref())
        .await
    } else {
        sqlx::query_as(
            "SELECT observation_date, value, label, region, unit
             FROM provider_observations
             WHERE provider = $1 AND series_id = $2
             ORDER BY observation_date DESC
             LIMIT $3",
        )
        .bind(&provider)
        .bind(&series_id)
        .bind(limit)
        .fetch_all(state.pool.as_ref())
        .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data: Vec<_> = rows.into_iter().map(|(d, v, l, r, u)| {
        serde_json::json!({
            "date": d.to_string(),
            "value": v,
            "label": l,
            "region": r,
            "unit": u,
        })
    }).collect();
    Ok(Json(serde_json::json!({
        "provider": provider,
        "series_id": series_id,
        "data": data,
    })))
}

// ── Ticker endpoints ─────────────────────────────────────────────────

pub async fn list_tickers(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT ticker FROM tickers WHERE active = true ORDER BY ticker LIMIT 5000",
    )
    .fetch_all(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tickers: Vec<String> = rows.into_iter().map(|(t,)| t).collect();
    Ok(Json(serde_json::json!({ "count": tickers.len(), "data": tickers })))
}

#[derive(Deserialize)]
pub struct PricesQuery {
    pub limit: Option<i64>,
}

pub async fn ticker_prices(
    State(state): State<AppState>,
    Path(ticker): Path<String>,
    Query(q): Query<PricesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = q.limit.unwrap_or(252).min(5000);
    let rows: Vec<(chrono::NaiveDate, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal, i64)> = sqlx::query_as(
        "SELECT date, open, high, low, close, volume
         FROM price_data
         WHERE ticker = $1
         ORDER BY date DESC
         LIMIT $2",
    )
    .bind(&ticker)
    .bind(limit)
    .fetch_all(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data: Vec<_> = rows.into_iter().map(|(d, o, h, l, c, v)| {
        serde_json::json!({
            "date": d.to_string(),
            "open": o.to_string().parse::<f64>().unwrap_or(0.0),
            "high": h.to_string().parse::<f64>().unwrap_or(0.0),
            "low":  l.to_string().parse::<f64>().unwrap_or(0.0),
            "close": c.to_string().parse::<f64>().unwrap_or(0.0),
            "volume": v,
        })
    }).collect();
    Ok(Json(serde_json::json!({ "ticker": ticker, "data": data })))
}

pub async fn ticker_lagrange(
    State(state): State<AppState>,
    Path(ticker): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows: Vec<(chrono::NaiveDate, f32, String, Option<f32>, Option<f32>, Option<f32>, Option<f32>, Option<String>)> = sqlx::query_as(
        "SELECT score_date, score, label, fin_score, astro_score, macro_score, short_score, concordance
         FROM lagrange_history
         WHERE ticker = $1
         ORDER BY score_date DESC
         LIMIT 365",
    )
    .bind(&ticker)
    .fetch_all(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data: Vec<_> = rows.into_iter().map(|(d, s, l, fin, astro, mac, sht, conc)| {
        serde_json::json!({
            "date": d.to_string(),
            "score": s,
            "label": l,
            "fin_score": fin,
            "astro_score": astro,
            "macro_score": mac,
            "short_score": sht,
            "concordance": conc,
        })
    }).collect();
    Ok(Json(serde_json::json!({ "ticker": ticker, "data": data })))
}

pub async fn ticker_astro(
    State(state): State<AppState>,
    Path(ticker): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row: Option<(Option<f64>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT astro_score, astro_label, dominant_theme FROM astro_scores WHERE ticker = $1
         ORDER BY score_date DESC LIMIT 1",
    )
    .bind(&ticker)
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(match row {
        Some((s, l, t)) => serde_json::json!({
            "ticker": ticker, "data": { "astro_score": s, "astro_label": l, "dominant_theme": t }
        }),
        None => serde_json::json!({ "ticker": ticker, "data": null }),
    }))
}

pub async fn ticker_fundamentals(
    State(state): State<AppState>,
    Path(ticker): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<i64>)> = sqlx::query_as(
        "SELECT pe_ratio, pb_ratio, ps_ratio, ev_ebitda, peg_ratio, roe, market_cap
         FROM fundamental_metrics
         WHERE ticker = $1
         ORDER BY metric_date DESC
         LIMIT 1",
    )
    .bind(&ticker)
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(match row {
        Some((pe, pb, ps, ev, peg, roe, mcap)) => serde_json::json!({
            "ticker": ticker, "data": {
                "pe_ratio": pe, "pb_ratio": pb, "ps_ratio": ps,
                "ev_ebitda": ev, "peg_ratio": peg, "roe": roe,
                "market_cap": mcap,
            }
        }),
        None => serde_json::json!({ "ticker": ticker, "data": null }),
    }))
}
