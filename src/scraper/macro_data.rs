//! FRED (St. Louis Fed) macroeconomic data fetcher.
//!
//! Fetches key series daily: Fed Funds Rate, CPI, Unemployment, 10Y Treasury,
//! 2Y Treasury, GDP growth (quarterly), VIX (via FRED proxy).
//!
//! FRED API: free, no rate limit for reasonable use. Key required (free registration).
//! Endpoint: https://api.stlouisfed.org/fred/series/observations

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// FRED series we track — (series_id, human_label)
// ---------------------------------------------------------------------------

const FRED_SERIES: &[(&str, &str)] = &[
    ("FEDFUNDS",  "Fed Funds Rate"),
    ("CPIAUCSL",  "CPI (All Urban)"),
    ("UNRATE",    "Unemployment Rate"),
    ("GS10",      "10Y Treasury Yield"),
    ("GS2",       "2Y Treasury Yield"),
    ("T10Y2Y",    "10Y-2Y Yield Spread"),
    ("GDPC1",     "Real GDP (Quarterly)"),
    ("VIXCLS",    "CBOE VIX"),
    ("M2SL",      "M2 Money Supply"),
    ("DCOILWTICO","WTI Crude Oil"),
];

// ---------------------------------------------------------------------------
// Serde types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FredResponse {
    observations: Vec<FredObservation>,
}

#[derive(Debug, Deserialize)]
struct FredObservation {
    date: String,
    value: String,  // FRED uses "." for missing values
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn fetch_all_macro(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    fred_api_key: Arc<String>,
) {
    for (series_id, series_name) in FRED_SERIES {
        match fetch_fred_series(series_id, series_name, &pool, &client, &fred_api_key).await {
            Ok(n) => println!("[FRED] {series_name}: {n} new observations"),
            Err(e) => eprintln!("[FRED] {series_name} error (skipping): {e:#}"),
        }
    }
}

async fn fetch_fred_series(
    series_id: &str,
    series_name: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<u64> {
    // Fetch the last 2 years of data (enough to show trends)
    let obs_start = (Utc::now() - chrono::Duration::days(730))
        .format("%Y-%m-%d")
        .to_string();

    let url = format!(
        "https://api.stlouisfed.org/fred/series/observations\
         ?series_id={series_id}\
         &api_key={api_key}\
         &file_type=json\
         &observation_start={obs_start}\
         &sort_order=asc"
    );

    let resp = client.get(&url).send().await
        .context("FRED request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        anyhow::bail!("FRED returned HTTP {status}: {body_text}");
    }

    let body: FredResponse = resp.json().await
        .context("Failed to parse FRED response")?;

    let mut inserted = 0u64;
    for obs in &body.observations {
        // FRED uses "." as a sentinel for missing data
        if obs.value == "." { continue; }

        let obs_date: NaiveDate = NaiveDate::parse_from_str(&obs.date, "%Y-%m-%d")
            .context(format!("Invalid FRED date: {}", obs.date))?;

        let value: rust_decimal::Decimal = obs.value.parse()
            .context(format!("Invalid FRED value: {}", obs.value))?;

        let result = sqlx::query(
            "INSERT INTO macro_indicators (series_id, series_name, obs_date, value) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (series_id, obs_date) DO NOTHING",
        )
        .bind(series_id)
        .bind(series_name)
        .bind(obs_date)
        .bind(value)
        .execute(pool)
        .await
        .context("DB insert failed")?;

        inserted += result.rows_affected();
    }

    Ok(inserted)
}
