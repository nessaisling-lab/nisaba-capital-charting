//! FINRA Developer API short sale volume fetcher.
//!
//! Endpoint: https://api.finra.org/data/group/consolidatedShortSaleVolume/name/consolidatedShortSaleVolumeDailyData
//! Auth: Bearer token in Authorization header.
//!
//! This is the consolidated (exchange + OTC) short sale volume dataset.
//! Data comes back with multiple rows per ticker per day (one per reporting
//! facility). We aggregate (sum) them, but also guard on the symbol field
//! to ensure we only include records for the requested ticker.
//!
//! Key field names from the API:
//!   securitiesInformationProcessorSymbolIdentifier — ticker symbol
//!   tradeReportDate — settlement date (YYYY-MM-DD)
//!   shortParQuantity   — short volume for this facility
//!   totalParQuantity   — total volume for this facility

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Serde types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FinraShortRecord {
    #[serde(rename = "securitiesInformationProcessorSymbolIdentifier")]
    symbol: Option<String>,
    #[serde(rename = "tradeReportDate")]
    trade_date: Option<String>,
    #[serde(rename = "shortParQuantity")]
    short_volume: Option<serde_json::Value>,
    #[serde(rename = "totalParQuantity")]
    total_volume: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn fetch_all_short_interest(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    finra_api_key: Arc<String>,
) {
    let since = (Utc::now() - chrono::Duration::days(14))
        .format("%Y-%m-%d")
        .to_string();

    // Fetch per ticker — FINRA API exact-match filter on the SIP symbol field
    for ticker in crate::WATCHLIST {
        match fetch_finra_ticker(ticker, &since, &pool, &client, &finra_api_key).await {
            Ok(n) => println!("[ShortInt] {ticker}: {n} new short volume rows"),
            Err(e) => eprintln!("[ShortInt] {ticker} error (skipping): {e:#}"),
        }
        // Brief pause between ticker calls
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }
}

async fn fetch_finra_ticker(
    ticker: &str,
    since_date: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<u64> {
    // FINRA consolidated short sale volume — covers exchange-listed equities.
    // The otcMarket/regShoDaily endpoint only covers OTC securities.
    let url = format!(
        "https://api.finra.org/data/group/consolidatedShortSaleVolume\
         /name/consolidatedShortSaleVolumeDailyData\
         ?limit=500\
         &compareFilters=securitiesInformationProcessorSymbolIdentifier=={ticker}\
         &dateRangeFilters=tradeReportDate>={since_date}"
    );

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "application/json")
        .send()
        .await
        .context("FINRA API request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("FINRA API returned HTTP {status}: {body}");
    }

    let records: Vec<FinraShortRecord> = resp.json().await
        .context("Failed to parse FINRA API response")?;

    // Aggregate multiple reporting-facility rows into one row per (ticker, date)
    let mut by_date: HashMap<NaiveDate, (i64, i64)> = HashMap::new();

    for rec in &records {
        // Guard: only include records that match this ticker
        match rec.symbol.as_deref() {
            Some(s) if s.eq_ignore_ascii_case(ticker) => {}
            Some(_) => continue,
            None => continue,
        }

        let date_str = match rec.trade_date.as_deref() {
            Some(d) => d,
            None => continue,
        };
        let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };

        let sv = parse_quantity(&rec.short_volume);
        let tv = parse_quantity(&rec.total_volume);

        let entry = by_date.entry(date).or_insert((0, 0));
        entry.0 += sv;
        entry.1 += tv;
    }

    let mut inserted = 0u64;
    for (date, (short_vol, total_vol)) in by_date {
        let short_pct: Option<rust_decimal::Decimal> = if total_vol > 0 {
            let pct = (short_vol as f64 / total_vol as f64) * 100.0;
            format!("{pct:.4}").parse().ok()
        } else {
            None
        };

        let result = sqlx::query(
            "INSERT INTO short_interest \
             (ticker, settlement_date, short_volume, total_volume, short_pct) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (ticker, settlement_date) DO NOTHING",
        )
        .bind(ticker)
        .bind(date)
        .bind(short_vol)
        .bind(total_vol)
        .bind(short_pct)
        .execute(pool)
        .await
        .context("DB insert failed")?;

        inserted += result.rows_affected();
    }

    Ok(inserted)
}

fn parse_quantity(val: &Option<serde_json::Value>) -> i64 {
    match val {
        Some(v) => v.as_i64()
            .or_else(|| v.as_f64().map(|f| f as i64))
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            .unwrap_or(0),
        None => 0,
    }
}
