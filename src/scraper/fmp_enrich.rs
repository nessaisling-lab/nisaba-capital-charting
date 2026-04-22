//! Financial Modeling Prep integration — two jobs in one module:
//!
//! 1. `seed_ticker_universe()` — calls `/v3/stock/list` once, inserts every
//!    US common stock / ETF into company_metadata with NULL ipo_date.
//!    This gives us the full Robinhood / Bloomberg tradeable universe.
//!    Skips if >= 1000 FMP rows already exist.
//!
//! 2. `enrich_ipo_dates()` — calls `/v3/profile/{ticker}` for up to 240
//!    tickers per day that have NULL ipo_date.  Fills the date, then seeds
//!    the natal chart so those tickers become fully operational.
//!
//! Free tier: 250 req/day.  We use 1 for the stock list + up to 240 for
//! profile enrichment, leaving 9 as safety margin.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;

const MAX_PROFILE_PER_RUN: usize = 240;

// ---------------------------------------------------------------------------
// FMP response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FmpStock {
    symbol:            String,
    name:              Option<String>,
    #[serde(rename = "exchangeShortName")]
    exchange:          Option<String>,
    #[serde(rename = "type")]
    asset_type:        Option<String>,
}

#[derive(Debug, Deserialize)]
struct FmpProfile {
    #[serde(rename = "ipoDate")]
    ipo_date: Option<String>,
    sector:   Option<String>,
    industry: Option<String>,
}

// FMP wraps profile results in an array
type FmpProfileResponse = Vec<FmpProfile>;

// ---------------------------------------------------------------------------
// Coordinate lookup — same NYSE/NASDAQ split as Polygon
// ---------------------------------------------------------------------------

fn exchange_coords(exchange: &str) -> (f64, f64) {
    match exchange {
        "NASDAQ" | "NASDAQ Capital Market" => (40.7589, -73.9851),
        _ => (40.7069, -74.0089), // NYSE, AMEX, OTC, etc.
    }
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Seeds the full US equity universe from FMP's stock list.
/// Runs once; after that, only new listings need adding.
pub async fn seed_ticker_universe(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
) {
    // Only run if we have fewer than 1000 FMP-sourced rows
    let fmp_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM company_metadata WHERE data_source = 'fmp'",
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0);

    if fmp_count >= 1000 {
        println!("[FMP] Universe already seeded ({fmp_count} FMP tickers) — skipping stock list fetch.");
        return;
    }

    println!("[FMP] Fetching full stock list (all US equities)...");

    match fetch_stock_list(&pool, &client, &api_key).await {
        Ok((inserted, skipped)) => println!(
            "[FMP] Stock list complete — {inserted} new tickers added, {skipped} already existed."
        ),
        Err(ref e) if e.to_string().contains("403") => {
            println!("[FMP] Stock list requires a paid FMP plan — skipping. Upgrade at financialmodelingprep.com.");
        }
        Err(e) => eprintln!("[FMP] Stock list failed: {e:#}"),
    }
}

/// Fills in NULL ipo_date values using FMP profile endpoint.
/// Called nightly after `seed_ticker_universe`.
pub async fn enrich_ipo_dates(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
) {
    // Count how many FMP calls used today (tracked in fetch_log)
    let fmp_calls_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fetch_log \
         WHERE source = 'fmp' AND fetched_at::date = CURRENT_DATE",
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0);

    // Free tier: 250/day.  Reserve 9 as safety margin.
    let budget = (241_i64 - fmp_calls_today)
        .max(0)
        .min(MAX_PROFILE_PER_RUN as i64) as usize;

    if budget == 0 {
        println!("[FMP] Daily budget exhausted ({fmp_calls_today} calls today). Skipping enrichment.");
        return;
    }

    // Watchlist tickers first, then alphabetical
    let order = crate::enrich_common::watchlist_priority_sql();
    let to_enrich: Vec<String> = sqlx::query_scalar(&format!(
        "SELECT ticker FROM company_metadata \
         WHERE ipo_date IS NULL \
         ORDER BY {order} \
         LIMIT {budget}"
    ))
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    if to_enrich.is_empty() {
        println!("[FMP] All tickers have IPO dates. Nothing to enrich.");
        return;
    }

    println!(
        "[FMP] Enriching {} ticker(s) with IPO dates (budget: {budget}/day)...",
        to_enrich.len()
    );

    let mut enriched = 0usize;
    let mut not_found = 0usize;

    for ticker in &to_enrich {
        match fetch_profile_ipo_date(ticker, &client, &api_key).await {
            Ok((ipo_date_opt, sector, industry)) => {
                // Always write sector/industry if available (regardless of ipo_date)
                if sector.is_some() || industry.is_some() {
                    let _ = sqlx::query(
                        "UPDATE company_metadata SET sector = COALESCE($1, sector), \
                         industry = COALESCE($2, industry) WHERE ticker = $3",
                    )
                    .bind(&sector)
                    .bind(&industry)
                    .bind(ticker)
                    .execute(pool.as_ref())
                    .await;
                }

                if let Some(ipo_date) = ipo_date_opt {
                    let updated = sqlx::query(
                        "UPDATE company_metadata SET ipo_date = $1 WHERE ticker = $2",
                    )
                    .bind(ipo_date)
                    .bind(ticker)
                    .execute(pool.as_ref())
                    .await;

                    if updated.is_ok() {
                        crate::enrich_common::seed_one_natal_chart(pool.as_ref(), ticker, ipo_date).await;
                        println!("[FMP] {ticker}: {ipo_date} — natal chart seeded");
                        enriched += 1;
                    } else if let Err(e) = updated {
                        eprintln!("[FMP] {ticker}: DB update failed: {e}");
                    }
                } else {
                    not_found += 1;
                }
            }
            Err(ref e) if e.to_string().contains("403") => {
                // Whole endpoint is blocked — stop immediately, don't hammer remaining tickers
                eprintln!(
                    "[FMP] HTTP 403 Forbidden — /v3/profile requires a paid FMP plan.\n\
                     [FMP] Upgrade at financialmodelingprep.com to unlock IPO date enrichment.\n\
                     [FMP] Stopping (skipped {} remaining tickers).",
                    to_enrich.len().saturating_sub(enriched + not_found + 1)
                );
                break;
            }
            Err(e) => eprintln!("[FMP] {ticker}: profile error: {e}"),
        }

        crate::log_fetch(pool.as_ref(), "fmp", Some(ticker), "profile", "ok", None).await;

        // ~4 calls/sec to stay well under free tier limits
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    println!("[FMP] Enrichment done — {enriched} dated, {not_found} not found in FMP.");
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

async fn fetch_stock_list(
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<(usize, usize)> {
    let url = format!(
        "https://financialmodelingprep.com/api/v3/stock/list?apikey={api_key}"
    );

    let resp = client.get(&url).send().await
        .context("FMP stock/list request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("FMP stock/list HTTP {}", resp.status());
    }

    let stocks: Vec<FmpStock> = resp.json().await
        .context("Failed to parse FMP stock list")?;

    println!("[FMP] Received {} tickers from FMP stock list.", stocks.len());

    let mut inserted = 0usize;
    let mut skipped  = 0usize;

    for s in &stocks {
        // Only US common stocks and ETFs — skip warrants, rights, etc.
        let asset_type = s.asset_type.as_deref().unwrap_or("stock");
        if asset_type != "stock" && asset_type != "etf" {
            continue;
        }
        // Skip empty or very long ticker symbols (likely not real equities)
        if s.symbol.is_empty() || s.symbol.len() > 10 {
            continue;
        }

        let exchange    = s.exchange.as_deref().unwrap_or("NYSE");
        let (lat, lon)  = exchange_coords(exchange);
        let name        = s.name.as_deref().unwrap_or(&s.symbol);

        let result = sqlx::query(
            "INSERT INTO company_metadata \
             (ticker, company_name, exchange, latitude, longitude, data_source, seeded_at) \
             VALUES ($1, $2, $3, $4, $5, 'fmp', NOW()) \
             ON CONFLICT (ticker) DO NOTHING",
        )
        .bind(&s.symbol)
        .bind(name)
        .bind(exchange)
        .bind(lat)
        .bind(lon)
        .execute(pool)
        .await;

        match result {
            Ok(r) if r.rows_affected() > 0 => inserted += 1,
            Ok(_)  => skipped += 1,
            Err(e) => eprintln!("[FMP] Insert failed for {}: {e}", s.symbol),
        }
    }

    Ok((inserted, skipped))
}

/// Returns (ipo_date, sector, industry) from the FMP profile endpoint.
async fn fetch_profile_ipo_date(
    ticker: &str,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<(Option<NaiveDate>, Option<String>, Option<String>)> {
    let url = format!(
        "https://financialmodelingprep.com/api/v3/profile/{ticker}?apikey={api_key}"
    );

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("FMP profile HTTP {}", resp.status());
    }

    let profiles: FmpProfileResponse = resp.json().await
        .context("Failed to parse FMP profile")?;

    let profile = profiles.into_iter().next();

    let date = profile.as_ref()
        .and_then(|p| p.ipo_date.as_deref())
        .filter(|s| !s.is_empty() && *s != "null" && *s != "None" && *s != "0000-00-00")
        .and_then(|s| s.parse::<NaiveDate>().ok());

    let sector   = profile.as_ref().and_then(|p| p.sector.clone()).filter(|s| !s.is_empty());
    let industry = profile.as_ref().and_then(|p| p.industry.clone()).filter(|s| !s.is_empty());

    Ok((date, sector, industry))
}

