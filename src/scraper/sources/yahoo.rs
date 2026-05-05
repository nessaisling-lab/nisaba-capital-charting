//! Yahoo Finance unofficial v8 chart API (Wave 6.A1).
//!
//! Endpoint: `https://query1.finance.yahoo.com/v8/finance/chart/{TICKER}?range=3mo&interval=1d`
//! Returns JSON with timestamp + indicators arrays. No API key required.
//!
//! Caveat: Yahoo has changed response shape historically (notably 2017).
//! Treat parse failures as transient errors with explicit logging so
//! operators see when format drift occurs.

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Deserialize;

use super::SourcedPriceRow;

#[derive(Debug, Deserialize)]
struct YahooResponse {
    chart: YahooChart,
}

#[derive(Debug, Deserialize)]
struct YahooChart {
    result: Option<Vec<YahooResult>>,
    error:  Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct YahooResult {
    timestamp:  Vec<i64>,
    indicators: YahooIndicators,
}

#[derive(Debug, Deserialize)]
struct YahooIndicators {
    quote: Vec<YahooQuote>,
}

#[derive(Debug, Deserialize)]
struct YahooQuote {
    open:   Vec<Option<f64>>,
    high:   Vec<Option<f64>>,
    low:    Vec<Option<f64>>,
    close:  Vec<Option<f64>>,
    volume: Vec<Option<i64>>,
}

pub async fn fetch_prices(ticker: &str, client: &reqwest::Client) -> Result<Vec<SourcedPriceRow>> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{ticker}?range=3mo&interval=1d"
    );

    // Yahoo blocks default reqwest User-Agent — set a browser-like one.
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (compatible; PursuitDashboard/11.4)")
        .header("Accept", "application/json")
        .send()
        .await
        .context("Yahoo HTTP request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("Yahoo HTTP {}", resp.status());
    }

    let body: YahooResponse = resp.json().await
        .context("Yahoo JSON parse failed — response shape may have changed")?;

    if let Some(err) = body.chart.error {
        anyhow::bail!("Yahoo API error: {err}");
    }

    let result = body.chart.result
        .and_then(|mut v| v.pop())
        .ok_or_else(|| anyhow!("Yahoo response had no result for {ticker}"))?;

    let quote = result.indicators.quote.into_iter().next()
        .ok_or_else(|| anyhow!("Yahoo response missing quote array"))?;

    let n = result.timestamp.len();
    let mut rows = Vec::with_capacity(n);
    for i in 0..n {
        // Skip rows with any missing OHLC field
        let (open, high, low, close, volume) = match (
            quote.open.get(i).copied().flatten(),
            quote.high.get(i).copied().flatten(),
            quote.low.get(i).copied().flatten(),
            quote.close.get(i).copied().flatten(),
            quote.volume.get(i).copied().flatten(),
        ) {
            (Some(o), Some(h), Some(l), Some(c), v) => (o, h, l, c, v),
            _ => continue,
        };

        let ts = result.timestamp[i];
        let date: NaiveDate = DateTime::<Utc>::from_timestamp(ts, 0)
            .ok_or_else(|| anyhow!("Invalid timestamp {ts}"))?
            .date_naive();

        let to_dec = |f: f64| Decimal::from_f64(f).unwrap_or_default();
        rows.push(SourcedPriceRow {
            date,
            open:  to_dec(open),
            high:  to_dec(high),
            low:   to_dec(low),
            close: to_dec(close),
            volume,
        });
    }

    Ok(rows)
}
