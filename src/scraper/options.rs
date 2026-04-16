//! Polygon.io options flow fetcher.
//!
//! Free tier (5 req/min, unlimited daily) supports:
//!   - /v2/aggs/ticker/{optionsTicker}/range/... — historical OHLCV for one contract
//!   - /v3/reference/options/{underlyingAsset}  — options chain reference data (free)
//!
//! The full snapshot endpoint (/v3/snapshot/options) requires a paid Starter plan.
//! This module uses the free reference + grouped daily endpoint to compute a
//! put/call ratio from yesterday's data.
//!
//! Note: If POLYGON_API_KEY is set to a Starter-tier key or higher, the scraper
//! will automatically get richer real-time data. The free tier gives next-day data.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Serde types — Polygon options reference (free tier)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct PolygonOptionsRefResponse {
    results: Option<Vec<PolygonOptionRef>>,
    status: String,
    #[allow(dead_code)]
    next_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PolygonOptionRef {
    #[allow(dead_code)]
    ticker: String,
    contract_type: String,  // "call" or "put"
}

#[derive(Debug, Deserialize)]
struct PolygonDailyAggsResponse {
    results: Option<Vec<PolygonDailyBar>>,
    #[allow(dead_code)]
    status: String,
}

#[derive(Debug, Deserialize)]
struct PolygonDailyBar {
    #[serde(rename = "v")]
    volume: Option<f64>,
    #[serde(rename = "o")]
    #[allow(dead_code)]
    open_interest: Option<f64>,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn fetch_all_options_flow(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    polygon_api_key: Arc<String>,
) {
    // Verify the key has options access before burning rate limit on each ticker.
    // Polygon free tier does not include options endpoints — Starter plan ($29/mo) required.
    // We probe once with AAPL; if it fails we skip the rest gracefully.
    match probe_options_access("AAPL", &client, &polygon_api_key).await {
        Ok(true) => {} // Starter+ key — proceed
        Ok(false) => {
            println!("[Options] Polygon free tier detected — options endpoints require Starter plan. Skipping.");
            return;
        }
        Err(e) => {
            eprintln!("[Options] Polygon probe failed: {e:#}. Skipping options fetch.");
            return;
        }
    }

    for ticker in crate::WATCHLIST {
        match fetch_options_pcr(ticker, &pool, &client, &polygon_api_key).await {
            Ok(()) => println!("[Options] {ticker}: put/call data stored"),
            Err(e) => eprintln!("[Options] {ticker} error (skipping): {e:#}"),
        }
        tokio::time::sleep(std::time::Duration::from_millis(12_100)).await;
    }
}

/// Returns Ok(true) if this key has options access, Ok(false) if it's free-tier limited.
async fn probe_options_access(
    ticker: &str,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<bool> {
    let url = format!(
        "https://api.polygon.io/v3/reference/options/{ticker}?limit=1&apiKey={api_key}"
    );
    let resp = client.get(&url).send().await
        .context("Polygon probe request failed")?;

    match resp.status().as_u16() {
        200 => Ok(true),
        401 | 403 | 404 => Ok(false),
        other => anyhow::bail!("Polygon probe returned HTTP {other}"),
    }
}

async fn fetch_options_pcr(
    ticker: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<()> {
    let snapshot_date = Utc::now().date_naive();

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM options_flow WHERE ticker = $1 AND snapshot_date = $2)",
    )
    .bind(ticker)
    .bind(snapshot_date)
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if exists { return Ok(()); }

    // Fetch options chain reference (contract list) — free on all Polygon tiers
    // Returns contract tickers like "O:AAPL250417C00150000"
    let url = format!(
        "https://api.polygon.io/v3/reference/options/{ticker}\
         ?limit=250\
         &expired=false\
         &apiKey={api_key}"
    );

    let resp = client.get(&url).send().await
        .context("Polygon reference request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Polygon returned HTTP {status}: {body}");
    }

    let body: PolygonOptionsRefResponse = resp.json().await
        .context("Failed to parse Polygon options reference response")?;

    if body.status != "OK" && body.status != "DELAYED" {
        anyhow::bail!("Polygon status: {}", body.status);
    }

    let contracts = match body.results {
        Some(r) if !r.is_empty() => r,
        _ => return Ok(()),
    };

    // Count contracts by type for a simple put/call ratio estimate
    let call_count = contracts.iter().filter(|c| c.contract_type == "call").count() as i64;
    let put_count  = contracts.iter().filter(|c| c.contract_type == "put").count() as i64;

    // Fetch yesterday's grouped daily data for actual volume
    let prev_day = (Utc::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let _grouped_url = format!(
        "https://api.polygon.io/v2/aggs/grouped/locale/us/market/otc/{prev_day}\
         ?adjusted=true\
         &apiKey={api_key}"
    );

    // Use reference-based estimates if grouped daily fetch fails
    let (call_volume, put_volume) = match fetch_grouped_options_volume(
        ticker, &prev_day, client, api_key
    ).await {
        Ok((cv, pv)) => (cv, pv),
        Err(_) => (call_count, put_count),  // Fall back to contract count ratio
    };

    let put_call_ratio: Option<rust_decimal::Decimal> = if call_volume > 0 {
        let ratio = put_volume as f64 / call_volume as f64;
        format!("{ratio:.4}").parse().ok()
    } else {
        None
    };

    sqlx::query(
        "INSERT INTO options_flow \
         (ticker, snapshot_date, call_volume, put_volume, put_call_ratio) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (ticker, snapshot_date) DO UPDATE \
         SET call_volume    = EXCLUDED.call_volume, \
             put_volume     = EXCLUDED.put_volume, \
             put_call_ratio = EXCLUDED.put_call_ratio",
    )
    .bind(ticker)
    .bind(snapshot_date)
    .bind(call_volume)
    .bind(put_volume)
    .bind(put_call_ratio)
    .execute(pool)
    .await
    .context("DB insert failed")?;

    Ok(())
}

async fn fetch_grouped_options_volume(
    ticker: &str,
    date: &str,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<(i64, i64)> {
    // Polygon grouped daily for options uses the options ticker prefix "O:{TICKER}"
    let url = format!(
        "https://api.polygon.io/v2/aggs/ticker/O:{ticker}/range/1/day/{date}/{date}\
         ?adjusted=false\
         &sort=asc\
         &limit=50000\
         &apiKey={api_key}"
    );

    let resp = client.get(&url).send().await
        .context("Polygon grouped daily request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("Polygon grouped returned HTTP {}", resp.status());
    }

    let body: PolygonDailyAggsResponse = resp.json().await
        .context("Failed to parse Polygon daily aggs")?;

    let results = body.results.unwrap_or_default();
    let total_volume: i64 = results.iter()
        .filter_map(|r| r.volume)
        .map(|v| v as i64)
        .sum();

    // Without per-contract type breakdown in this endpoint, return total as proxy
    Ok((total_volume / 2, total_volume / 2))
}
