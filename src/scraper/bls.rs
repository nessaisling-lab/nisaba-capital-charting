//! Wave 7.6 — Bureau of Labor Statistics public data API v2.
//!
//! Free without key (rate-limited 25 req/day per IP). With BLS_API_KEY
//! env var set, gets 500 req/day. Tracks headline employment / inflation
//! detail series:
//!   - LNS14000000 — Unemployment rate, seasonally adjusted
//!   - CES0000000001 — Total nonfarm employment (in thousands)
//!   - CUUR0000SA0 — CPI All Urban Consumers
//!   - CUUR0000SAF1 — CPI Food
//!   - CUUR0000SAH — CPI Housing
//!   - CUUR0000SETB01 — CPI Gasoline (motor fuel)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const API_BASE: &str = "https://api.bls.gov/publicAPI/v2/timeseries/data/";

const SERIES: &[(&str, &str, &str)] = &[
    ("LNS14000000",        "Unemployment rate, SA",          "%"),
    ("CES0000000001",      "Total nonfarm employment",       "thousands"),
    ("CUUR0000SA0",        "CPI All Urban Consumers",        "index"),
    ("CUUR0000SAF1",       "CPI Food",                       "index"),
    ("CUUR0000SAH",        "CPI Housing",                    "index"),
    ("CUUR0000SETB01",     "CPI Gasoline (motor fuel)",      "index"),
];

#[derive(Serialize)]
struct BlsRequest<'a> {
    seriesid: Vec<&'a str>,
    startyear: String,
    endyear: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    registrationkey: Option<String>,
}

#[derive(Deserialize)]
struct BlsResponse {
    #[serde(rename = "Results")]
    results: BlsResults,
}

#[derive(Deserialize)]
struct BlsResults {
    series: Vec<BlsSeries>,
}

#[derive(Deserialize)]
struct BlsSeries {
    #[serde(rename = "seriesID")]
    series_id: String,
    data: Vec<BlsDatum>,
}

#[derive(Deserialize)]
struct BlsDatum {
    year: String,
    period: String, // M01..M12 monthly, Q01..Q04 quarterly, A01 annual
    value: String,
}

pub async fn fetch_all(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>) -> Result<()> {
    let key = std::env::var("BLS_API_KEY").ok();
    let now = chrono::Local::now().date_naive();
    let end_year = chrono::Datelike::year(&now);
    let start_year = end_year - 5;
    let body = BlsRequest {
        seriesid: SERIES.iter().map(|(id, _, _)| *id).collect(),
        startyear: start_year.to_string(),
        endyear: end_year.to_string(),
        registrationkey: key.clone(),
    };
    let resp = client
        .post(API_BASE)
        .json(&body)
        .send().await
        .context("BLS request")?;
    if !resp.status().is_success() {
        anyhow::bail!("BLS HTTP {}", resp.status());
    }
    let parsed: BlsResponse = resp.json().await.context("BLS parse")?;
    let mut total = 0_i64;
    for series in parsed.results.series {
        let meta = SERIES.iter().find(|(id, _, _)| *id == series.series_id);
        let (label, unit) = match meta {
            Some((_, l, u)) => (*l, *u),
            None => (series.series_id.as_str(), "value"),
        };
        for datum in series.data {
            let Ok(year) = datum.year.parse::<i32>() else { continue };
            let month = match datum.period.as_str() {
                p if p.starts_with('M') => p.trim_start_matches('M').parse::<u32>().ok(),
                "Q01" => Some(3), "Q02" => Some(6), "Q03" => Some(9), "Q04" => Some(12),
                "A01" => Some(12),
                _ => None,
            };
            let Some(month) = month else { continue };
            let date = match chrono::NaiveDate::from_ymd_opt(year, month, 1) {
                Some(d) => d,
                None => continue,
            };
            let Ok(value) = datum.value.parse::<f64>() else { continue };
            let r = sqlx::query(
                "INSERT INTO provider_observations
                    (provider, series_id, observation_date, value, label, region, unit, fetched_at)
                 VALUES ('bls', $1, $2, $3, $4, 'USA', $5, NOW())
                 ON CONFLICT (provider, series_id, region, observation_date)
                 DO UPDATE SET value = EXCLUDED.value, fetched_at = NOW()",
            )
            .bind(&series.series_id)
            .bind(date)
            .bind(value)
            .bind(label)
            .bind(unit)
            .execute(pool.as_ref())
            .await;
            if r.is_ok() { total += 1; }
        }
    }
    println!("[bls] done — {} observations stored ({} series, {})",
        total, SERIES.len(),
        if key.is_some() { "with key" } else { "no key (25 req/day limit)" });
    Ok(())
}
