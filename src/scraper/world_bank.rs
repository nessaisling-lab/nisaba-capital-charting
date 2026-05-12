//! Wave 7.1 — World Bank Open Data API.
//!
//! Free REST endpoint, no key required. Fetches headline economic
//! indicators for major economies. Stored in `provider_observations`
//! keyed by `provider = 'world_bank'`.
//!
//! Indicators tracked (~10 per country, annual):
//!   - NY.GDP.MKTP.CD       — GDP, current US$
//!   - NY.GDP.MKTP.KD.ZG    — GDP growth %
//!   - FP.CPI.TOTL.ZG       — Inflation, consumer prices YoY %
//!   - SL.UEM.TOTL.ZS       — Unemployment %
//!   - GC.DOD.TOTL.GD.ZS    — Govt debt / GDP %
//!   - BX.KLT.DINV.WD.GD.ZS — FDI net inflows / GDP %
//!   - NE.EXP.GNFS.ZS       — Exports / GDP %
//!   - NE.IMP.GNFS.ZS       — Imports / GDP %
//!   - SP.POP.TOTL          — Population total
//!   - NV.IND.TOTL.ZS       — Industry / GDP %
//!
//! Countries: G7 + BRICS + Eurozone aggregate (12 regions × 10 = 120 series).

use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::Arc;

const API_BASE: &str = "https://api.worldbank.org/v2";

const INDICATORS: &[(&str, &str, &str)] = &[
    ("NY.GDP.MKTP.CD",       "GDP (current US$)",                "USD"),
    ("NY.GDP.MKTP.KD.ZG",    "GDP growth (annual %)",            "%"),
    ("FP.CPI.TOTL.ZG",       "Inflation, consumer prices (YoY %)", "%"),
    ("SL.UEM.TOTL.ZS",       "Unemployment (%)",                  "%"),
    ("GC.DOD.TOTL.GD.ZS",    "Govt debt / GDP (%)",               "%"),
    ("BX.KLT.DINV.WD.GD.ZS", "FDI net inflows / GDP (%)",         "%"),
    ("NE.EXP.GNFS.ZS",       "Exports / GDP (%)",                 "%"),
    ("NE.IMP.GNFS.ZS",       "Imports / GDP (%)",                 "%"),
    ("SP.POP.TOTL",          "Population (total)",                "people"),
    ("NV.IND.TOTL.ZS",       "Industry / GDP (%)",                "%"),
];

const COUNTRIES: &[&str] = &[
    "USA", "CHN", "JPN", "DEU", "GBR", "FRA", "IND", "ITA", "CAN", "BRA",
    "RUS", "EMU", // Eurozone aggregate
];

#[derive(Debug, Deserialize)]
struct WbObservation {
    date: String,
    value: Option<f64>,
}

pub async fn fetch_all(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>) -> Result<()> {
    let mut total_inserted = 0_i64;
    let mut total_errors = 0_u32;

    for &country in COUNTRIES {
        for (series_id, label, unit) in INDICATORS {
            // World Bank API: /country/{code}/indicator/{ind}?format=json&per_page=20
            // Returns the latest 20 annual observations.
            let url = format!(
                "{API_BASE}/country/{country}/indicator/{series_id}?format=json&per_page=20"
            );
            match fetch_series(&client, &url).await {
                Ok(observations) => {
                    let n = store_observations(
                        Arc::clone(&pool),
                        country,
                        series_id,
                        label,
                        unit,
                        &observations,
                    )
                    .await?;
                    total_inserted += n;
                }
                Err(e) => {
                    eprintln!("[world_bank] {country}/{series_id} error: {e:#}");
                    total_errors += 1;
                }
            }
            // Be polite to the World Bank API (no published rate limit but
            // 100ms between calls keeps us well under any throttle)
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    println!(
        "[world_bank] done — {} obs across {} countries × {} indicators ({} errors)",
        total_inserted, COUNTRIES.len(), INDICATORS.len(), total_errors
    );
    Ok(())
}

async fn fetch_series(
    client: &reqwest::Client,
    url: &str,
) -> Result<Vec<WbObservation>> {
    let resp = client
        .get(url)
        .header("User-Agent", "NisabaEngine/0.1")
        .send()
        .await
        .context("WB request")?;
    if !resp.status().is_success() {
        anyhow::bail!("WB HTTP {}", resp.status());
    }
    // World Bank returns [metadata, observations[]] as a 2-element array
    let body: serde_json::Value = resp.json().await.context("WB parse")?;
    let arr = body.as_array().context("WB body not array")?;
    if arr.len() < 2 {
        return Ok(vec![]);
    }
    let observations: Vec<WbObservation> = serde_json::from_value(arr[1].clone())
        .unwrap_or_default();
    Ok(observations)
}

async fn store_observations(
    pool: Arc<sqlx::PgPool>,
    country: &str,
    series_id: &str,
    label: &str,
    unit: &str,
    observations: &[WbObservation],
) -> Result<i64> {
    let mut inserted = 0_i64;
    for obs in observations {
        let Some(value) = obs.value else { continue };
        // World Bank dates are years ("2023"); convert to YYYY-12-31
        let date = match obs.date.parse::<i32>() {
            Ok(year) => match chrono::NaiveDate::from_ymd_opt(year, 12, 31) {
                Some(d) => d,
                None => continue,
            },
            Err(_) => continue,
        };
        let result = sqlx::query(
            "INSERT INTO provider_observations
                (provider, series_id, observation_date, value, label, region, unit, fetched_at)
             VALUES ('world_bank', $1, $2, $3, $4, $5, $6, NOW())
             ON CONFLICT (provider, series_id, region, observation_date)
             DO UPDATE SET value = EXCLUDED.value, fetched_at = NOW()",
        )
        .bind(series_id)
        .bind(date)
        .bind(value)
        .bind(label)
        .bind(country)
        .bind(unit)
        .execute(pool.as_ref())
        .await;
        if result.is_ok() {
            inserted += 1;
        }
    }
    Ok(inserted)
}
