//! Stooq CSV daily prices (Wave 6.A1).
//!
//! Endpoint: `https://stooq.com/q/d/l/?s={ticker}.us&i=d`
//! Returns CSV with header `Date,Open,High,Low,Close,Volume`. No API key.
//!
//! Stooq is reliable for end-of-day data, especially good for EU/Asia
//! tickers that Alpha Vantage's free tier doesn't cover well.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::SourcedPriceRow;

pub async fn fetch_prices(ticker: &str, client: &reqwest::Client) -> Result<Vec<SourcedPriceRow>> {
    let url = format!("https://stooq.com/q/d/l/?s={}.us&i=d", ticker.to_lowercase());
    let resp = client.get(&url).send().await
        .context("Stooq HTTP request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("Stooq HTTP {}", resp.status());
    }

    let csv = resp.text().await
        .context("Stooq response body read failed")?;

    // Stooq returns "No data" or empty body when ticker not found
    if csv.trim().is_empty() || csv.starts_with("No data") {
        anyhow::bail!("Stooq has no data for {ticker}");
    }

    let mut lines = csv.lines();
    let header = lines.next().unwrap_or("");
    if !header.starts_with("Date,Open,High,Low,Close,Volume") {
        anyhow::bail!("Stooq response shape unexpected — header was: {header}");
    }

    let mut rows = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 5 { continue; }

        let date: NaiveDate = match NaiveDate::parse_from_str(parts[0], "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };
        let open:  Decimal = parts[1].parse().unwrap_or_default();
        let high:  Decimal = parts[2].parse().unwrap_or_default();
        let low:   Decimal = parts[3].parse().unwrap_or_default();
        let close: Decimal = parts[4].parse().unwrap_or_default();
        let volume: Option<i64> = parts.get(5).and_then(|v| v.parse().ok());

        rows.push(SourcedPriceRow { date, open, high, low, close, volume });
    }

    Ok(rows)
}
