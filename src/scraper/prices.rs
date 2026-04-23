use anyhow::{Context, Result};
use chrono::NaiveDate;
use pursuit_week4_automation::models::AlphaVantageResponse;
use rust_decimal::Decimal;
use std::sync::Arc;

pub async fn fetch_all_tickers(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
    limiter: Arc<governor::DefaultDirectRateLimiter>,
) {
    for ticker in crate::WATCHLIST {
        limiter.until_ready().await;
        match fetch_and_store(ticker, &pool, &client, &api_key).await {
            Ok(inserted) => {
                println!("[{ticker}] Inserted {inserted} new price rows");
                crate::log_fetch(&pool, "alpha_vantage", Some(ticker), "price_data", "ok", None).await;
            }
            Err(e) => {
                eprintln!("[{ticker}] Price error (skipping): {e:#}");
                crate::log_fetch(&pool, "alpha_vantage", Some(ticker), "price_data", "error", Some(&e.to_string())).await;
            }
        }
    }
}

/// Fetch price data for astro-priority tickers (top/bottom ranked by astro score).
/// Called before `fetch_all_tickers` so the most astrologically interesting tickers
/// get financial data first, before API budget is exhausted.
pub async fn fetch_priority_prices(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
    limiter: Arc<governor::DefaultDirectRateLimiter>,
    priority_tickers: &[String],
) {
    let watchlist_set: std::collections::HashSet<&str> =
        crate::WATCHLIST.iter().copied().collect();

    for ticker in priority_tickers {
        // Skip tickers already in the watchlist (they'll be fetched by fetch_all_tickers)
        if watchlist_set.contains(ticker.as_str()) {
            continue;
        }
        limiter.until_ready().await;
        match fetch_and_store(ticker, &pool, &client, &api_key).await {
            Ok(inserted) => {
                println!("[{ticker}] (astro priority) Inserted {inserted} new price rows");
                crate::log_fetch(&pool, "alpha_vantage", Some(ticker), "price_data", "ok", None).await;
            }
            Err(e) => {
                eprintln!("[{ticker}] (astro priority) Price error: {e:#}");
                crate::log_fetch(&pool, "alpha_vantage", Some(ticker), "price_data", "error", Some(&e.to_string())).await;
            }
        }
    }
}

async fn fetch_and_store(
    ticker: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<u64> {
    let url = format!(
        "https://www.alphavantage.co/query\
         ?function=TIME_SERIES_DAILY\
         &symbol={ticker}\
         &apikey={api_key}\
         &outputsize=compact"
    );

    let response = client
        .get(&url)
        .send()
        .await
        .context("HTTP request to Alpha Vantage failed")?;

    if !response.status().is_success() {
        anyhow::bail!("Alpha Vantage returned HTTP {}", response.status());
    }

    let body: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse Alpha Vantage response")?;

    if let Some(note) = body.get("Note") {
        anyhow::bail!("Rate limit message: {note}");
    }
    if let Some(msg) = body.get("Information") {
        anyhow::bail!("Alpha Vantage info: {msg}");
    }

    let parsed: AlphaVantageResponse =
        serde_json::from_value(body).context("Failed to parse time series")?;

    let mut inserted = 0u64;
    for (date_str, entry) in &parsed.time_series {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .context(format!("Invalid date: {date_str}"))?;

        let open: Decimal  = entry.open.parse().context("parse open")?;
        let high: Decimal  = entry.high.parse().context("parse high")?;
        let low: Decimal   = entry.low.parse().context("parse low")?;
        let close: Decimal = entry.close.parse().context("parse close")?;
        let volume: i64    = entry.volume.parse().context("parse volume")?;

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
