//! Multi-source data adapters (Wave 6.A1).
//!
//! When the primary source (Alpha Vantage) fails or rate-limits, the
//! cascade tries Yahoo Finance then Stooq before giving up. Each call
//! returns rows tagged with a source name for provenance tracking in
//! the `price_data.data_source` column.

pub mod yahoo;
pub mod stooq;

use anyhow::Result;
use chrono::NaiveDate;
use rust_decimal::Decimal;

/// One day of OHLCV from any source. Decimal precision matches the existing
/// `price_data` schema. `volume` may be None for sources that omit it.
#[derive(Debug, Clone)]
pub struct SourcedPriceRow {
    pub date:   NaiveDate,
    pub open:   Decimal,
    pub high:   Decimal,
    pub low:    Decimal,
    pub close:  Decimal,
    pub volume: Option<i64>,
}

/// Try fallback sources in order. Returns rows + the source name that
/// succeeded. Each source has its own log line on failure so operators
/// can see which one is flaky.
pub async fn fetch_fallback_chain(
    ticker: &str,
    client: &reqwest::Client,
) -> Result<(Vec<SourcedPriceRow>, &'static str)> {
    // Source 1: Yahoo Finance (no key, JSON API, generous limits)
    match yahoo::fetch_prices(ticker, client).await {
        Ok(rows) if !rows.is_empty() => return Ok((rows, "yahoo")),
        Ok(_)  => eprintln!("[fallback] yahoo returned empty rows for {ticker}"),
        Err(e) => eprintln!("[fallback] yahoo failed for {ticker}: {e}"),
    }

    // Source 2: Stooq (no key, CSV, EU/Asia coverage)
    match stooq::fetch_prices(ticker, client).await {
        Ok(rows) if !rows.is_empty() => return Ok((rows, "stooq")),
        Ok(_)  => eprintln!("[fallback] stooq returned empty rows for {ticker}"),
        Err(e) => eprintln!("[fallback] stooq failed for {ticker}: {e}"),
    }

    anyhow::bail!("All fallback sources exhausted for {ticker}")
}
