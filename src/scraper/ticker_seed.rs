//! Universal ticker seed — fetches all active US common stocks from Polygon.io
//! and populates `company_metadata` so the astrology engine has a birth chart
//! for every publicly traded company, not just the 10 in WATCHLIST.
//!
//! Run condition: skips automatically if `data_source = 'polygon'` rows already exist.
//! Safe to re-run at any time (ON CONFLICT DO NOTHING on both tables).
//!
//! Endpoint: GET /v3/reference/tickers?market=stocks&locale=us&type=CS&active=true&limit=1000
//! Pagination: cursor-based via `next_url` field in response.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Polygon.io response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct PolygonTickersResponse {
    results:  Option<Vec<PolygonTicker>>,
    status:   String,
    next_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PolygonTicker {
    ticker:           String,
    name:             String,
    primary_exchange: Option<String>,
    list_date:        Option<String>, // "YYYY-MM-DD" or null for very old companies
}

// ---------------------------------------------------------------------------
// Exchange MIC → human name + coordinates (all US exchanges are in NYC)
// ---------------------------------------------------------------------------

fn mic_to_exchange(mic: &str) -> &'static str {
    match mic {
        "XNAS" => "NASDAQ",
        "XNYS" => "NYSE",
        "ARCX" => "NYSE Arca",
        "BATS" => "CBOE/BATS",
        "XASE" => "NYSE American",
        _      => "NYSE",
    }
}

/// Returns (latitude, longitude) for a given exchange MIC code.
/// All major US stock exchanges operate in or route through New York City.
fn mic_to_coords(mic: &str) -> (f64, f64) {
    match mic {
        "XNAS" => (40.7589, -73.9851), // NASDAQ — 151 W 42nd St, Midtown NYC
        _      => (40.7069, -74.0089), // NYSE, NYSE Arca, BATS, NYSE American — 11 Wall St
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Runs the bulk ticker seed if it has not been done before.
/// Called as the first step of `run_all_fetches` when a Polygon key is available.
pub async fn run(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>, api_key: Arc<String>) {
    // Only skip if we have a substantial universe already loaded (>= 1000 polygon rows).
    // A handful of rows means the previous run was interrupted — re-seed.
    let polygon_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM company_metadata WHERE data_source = 'polygon'",
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0);

    if polygon_count >= 1000 {
        println!("[TickerSeed] Universe already seeded ({polygon_count} tickers) — skipping bulk fetch.");
        return;
    }

    if polygon_count > 0 {
        println!("[TickerSeed] Partial seed detected ({polygon_count} rows) — re-running to complete.");
    }

    println!("[TickerSeed] Cold start: fetching full US equity universe from Polygon.io...");
    println!("[TickerSeed] This runs once (~10-15 pages at 13s/page ≈ 2-3 min). Tickers without")
;   println!("[TickerSeed] a list_date will be inserted with NULL and enriched later via AV OVERVIEW.");

    match fetch_and_seed(pool, client, api_key).await {
        Ok((inserted, skipped)) => println!(
            "[TickerSeed] Complete — {} tickers seeded into company_metadata, {} skipped (null list_date).",
            inserted, skipped
        ),
        Err(e) => eprintln!("[TickerSeed] Failed: {e:#}"),
    }
}

// ---------------------------------------------------------------------------
// Internal — paginate Polygon and upsert into company_metadata
// ---------------------------------------------------------------------------

async fn fetch_and_seed(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
) -> Result<(usize, usize)> {
    let first_url = format!(
        "https://api.polygon.io/v3/reference/tickers\
         ?market=stocks&locale=us&type=CS&active=true&limit=1000\
         &apiKey={api_key}"
    );

    let mut url      = first_url;
    let mut page     = 0usize;
    let mut inserted = 0usize;
    let mut skipped  = 0usize;
    let mut seen     = HashSet::<String>::new();

    loop {
        page += 1;

        let resp = client.get(&url).send().await
            .context("Polygon tickers request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body   = resp.text().await.unwrap_or_default();
            anyhow::bail!("Polygon returned HTTP {status}: {body}");
        }

        let body: PolygonTickersResponse = resp.json().await
            .context("Failed to parse Polygon tickers response")?;

        if body.status != "OK" && body.status != "DELAYED" {
            anyhow::bail!("Polygon status: {}", body.status);
        }

        let results    = body.results.unwrap_or_default();
        let page_count = results.len();

        // On the first ticker of the first page, print a sample so we can
        // confirm what Polygon is actually returning (list_date null or not).
        if page == 1 {
            if let Some(first) = results.first() {
                println!(
                    "[TickerSeed] Sample — ticker: {}, list_date: {:?}, exchange: {:?}",
                    first.ticker, first.list_date, first.primary_exchange
                );
            }
        }

        for t in results {
            // Dedup — Polygon occasionally lists the same ticker on multiple exchanges
            if !seen.insert(t.ticker.clone()) {
                continue;
            }

            // Parse list_date if present — null is OK, we insert anyway and enrich later
            let ipo_date: Option<NaiveDate> = t.list_date
                .as_deref()
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse::<NaiveDate>().ok());

            if ipo_date.is_none() {
                skipped += 1; // count for logging, but still insert
            }

            let mic        = t.primary_exchange.as_deref().unwrap_or("XNYS");
            let exchange   = mic_to_exchange(mic);
            let (lat, lon) = mic_to_coords(mic);

            let result = sqlx::query(
                "INSERT INTO company_metadata \
                 (ticker, company_name, ipo_date, exchange, latitude, longitude, data_source, seeded_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, 'polygon', NOW()) \
                 ON CONFLICT (ticker) DO NOTHING",
            )
            .bind(&t.ticker)
            .bind(&t.name)
            .bind(ipo_date)       // Option<NaiveDate> — NULL when unknown
            .bind(exchange)
            .bind(lat)
            .bind(lon)
            .execute(pool.as_ref())
            .await;

            match result {
                Ok(r) if r.rows_affected() > 0 => inserted += 1,
                Ok(_)  => {} // already existed (manual seed from migration 0008)
                Err(e) => eprintln!("[TickerSeed] Insert failed for {}: {e}", t.ticker),
            }
        }

        println!(
            "[TickerSeed] Page {page}: {page_count} tickers processed \
             (inserted: {inserted}, no-date: {skipped})"
        );

        match body.next_url {
            Some(next) => {
                // Polygon's next_url does not include the API key — append it
                url = format!("{next}&apiKey={api_key}");
                // Polygon free tier: 5 req/min = 1 req/12s. Use 13s to stay safely under.
                tokio::time::sleep(std::time::Duration::from_millis(13_000)).await;
            }
            None => break,
        }
    }

    Ok((inserted, skipped))
}
