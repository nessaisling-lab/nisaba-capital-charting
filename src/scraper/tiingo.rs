//! Tiingo bulk price feed — fills historical OHLCV for all tickers.
//!
//! Tiingo free tier: 500 req/day, full OHLCV history per ticker.
//! We use up to 490 calls/day (leaving 10 as safety margin).
//!
//! Priority order:
//!   1. WATCHLIST tickers (always refreshed first)
//!   2. Tickers with natal charts but fewer than 26 price rows
//!      (26 is the Lagrange financial-component threshold)
//!
//! For each ticker we fetch from either:
//!   • 5 years ago (if no price data exists yet), or
//!   • the day after their latest stored price row
//!
//! This is the primary unlock for Lagrange scores: once a ticker has
//! 26+ daily price rows, its financial component (35% weight) activates.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;

// Free tier: 500 req/day, but burst limit is ~50/hour.
// Keep well inside the burst window so a single run never 429s.
const MAX_PER_RUN: usize = 45;
const LAGRANGE_MIN_ROWS: i64 = 26;

// Five years of history is enough for Lagrange and momentum indicators.
fn five_years_ago() -> NaiveDate {
    chrono::Utc::now().date_naive() - chrono::Duration::days(365 * 5)
}

// ---------------------------------------------------------------------------
// Tiingo response type
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct TiingoPriceRow {
    // "2024-01-02T00:00:00+00:00"
    date:   String,
    open:   Option<Decimal>,
    high:   Option<Decimal>,
    low:    Option<Decimal>,
    close:  Option<Decimal>,
    volume: Option<i64>,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn fetch_all_prices_tiingo(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
) {
    // Count Tiingo calls already made today
    let calls_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fetch_log \
         WHERE source = 'tiingo' AND fetched_at::date = CURRENT_DATE",
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0);

    let budget = (MAX_PER_RUN as i64 - calls_today).max(0) as usize;
    if budget == 0 {
        println!("[Tiingo] Daily budget exhausted ({calls_today} calls today). Skipping.");
        return;
    }

    // Build a prioritised ticker list:
    //   (a) watchlist tickers first
    //   (b) tickers with natal chart but < 26 price rows
    let order = crate::enrich_common::watchlist_priority_sql();
    let tickers: Vec<String> = sqlx::query_scalar(&format!(
        "SELECT cm.ticker
         FROM company_metadata cm
         WHERE cm.ipo_date IS NOT NULL
           AND (
             cm.ticker = ANY($1)
             OR (
               SELECT COUNT(*) FROM price_data pd WHERE pd.ticker = cm.ticker
             ) < {LAGRANGE_MIN_ROWS}
           )
         ORDER BY {order}
         LIMIT {budget}"
    ))
    .bind(crate::watchlist())
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    if tickers.is_empty() {
        println!("[Tiingo] All tickers have sufficient price history. Nothing to fetch.");
        return;
    }

    println!(
        "[Tiingo] Fetching price history for {} ticker(s) (budget: {budget}/day)...",
        tickers.len()
    );

    let mut total_rows = 0u64;
    let mut errors     = 0usize;

    for ticker in &tickers {
        // Find the day after the latest stored price, or fall back to 5 years ago
        let start_date: NaiveDate = sqlx::query_scalar(
            "SELECT MAX(date) + INTERVAL '1 day' FROM price_data WHERE ticker = $1",
        )
        .bind(ticker.as_str())
        .fetch_one(pool.as_ref())
        .await
        .ok()
        .flatten()
        .unwrap_or_else(five_years_ago);

        let today = chrono::Utc::now().date_naive();
        if start_date > today {
            // Already up to date
            crate::log_fetch(pool.as_ref(), "tiingo", Some(ticker), "price", "ok", None).await;
            continue;
        }

        match fetch_and_store(ticker, start_date, &client, &api_key, pool.as_ref()).await {
            Ok(inserted) => {
                total_rows += inserted;
                println!("[Tiingo] {ticker}: +{inserted} rows (from {start_date})");
                crate::log_fetch(pool.as_ref(), "tiingo", Some(ticker), "price", "ok", None).await;
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("429") {
                    // Burst limit hit — stop the run cleanly rather than
                    // hammering the API. The daily budget check at the top of
                    // the next run will account for calls already logged.
                    eprintln!("[Tiingo] 429 rate-limit hit — stopping run early. Remaining tickers will be fetched tomorrow.");
                    crate::log_fetch(
                        pool.as_ref(), "tiingo", Some(ticker), "price", "rate_limit",
                        Some("429 Too Many Requests — run stopped early"),
                    ).await;
                    break;
                }
                errors += 1;
                eprintln!("[Tiingo] {ticker}: {e:#}");
                crate::log_fetch(
                    pool.as_ref(), "tiingo", Some(ticker), "price", "error",
                    Some(&msg),
                ).await;
            }
        }

        // ~1 req/sec — stays comfortably inside the ~50/hour burst window
        tokio::time::sleep(std::time::Duration::from_millis(1_000)).await;
    }

    println!(
        "[Tiingo] Done — {total_rows} price rows inserted across {} tickers ({errors} errors).",
        tickers.len()
    );
}

// ---------------------------------------------------------------------------
// Fetch and store a single ticker's price history
// ---------------------------------------------------------------------------

async fn fetch_and_store(
    ticker: &str,
    start_date: NaiveDate,
    client: &reqwest::Client,
    api_key: &str,
    pool: &sqlx::PgPool,
) -> Result<u64> {
    let url = format!(
        "https://api.tiingo.com/tiingo/daily/{ticker}/prices\
         ?startDate={start_date}\
         &token={api_key}"
    );

    let resp = client
        .get(&url)
        .header("Content-Type", "application/json")
        .send()
        .await
        .context("Tiingo HTTP request failed")?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        // Ticker not in Tiingo — not an error worth logging loudly
        return Ok(0);
    }

    if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        anyhow::bail!("429 Too Many Requests");
    }

    if !resp.status().is_success() {
        anyhow::bail!("Tiingo HTTP {}", resp.status());
    }

    let rows: Vec<TiingoPriceRow> = resp.json().await
        .context("Failed to parse Tiingo price response")?;

    let mut inserted = 0u64;

    for row in &rows {
        // Tiingo date: "2024-01-02T00:00:00+00:00" — take first 10 chars
        let date_part = &row.date[..row.date.len().min(10)];
        let date = match date_part.parse::<NaiveDate>() {
            Ok(d)  => d,
            Err(_) => continue,
        };

        let open   = match row.open   { Some(v) => v, None => continue };
        let high   = match row.high   { Some(v) => v, None => continue };
        let low    = match row.low    { Some(v) => v, None => continue };
        let close  = match row.close  { Some(v) => v, None => continue };
        let volume = row.volume.unwrap_or(0);

        let result = sqlx::query(
            "INSERT INTO price_data (ticker, date, open, high, low, close, volume) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (ticker, date) DO NOTHING",
        )
        .bind(ticker)
        .bind(date)
        .bind(open)
        .bind(high)
        .bind(low)
        .bind(close)
        .bind(volume)
        .execute(pool)
        .await
        .context("DB insert failed")?;

        inserted += result.rows_affected();
    }

    Ok(inserted)
}
