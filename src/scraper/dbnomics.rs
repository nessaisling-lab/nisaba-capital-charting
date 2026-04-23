//! DBnomics international economics data fetcher.
//!
//! Fetches key macro series from DBnomics (api.db.nomics.world), a free
//! aggregator of 70+ statistical providers (ECB, BIS, IMF, Eurostat, OECD).
//! No API key required. Rate limit: ~3 requests/second.
//!
//! Data is stored in the existing `macro_indicators` table with a "DBNOMICS:"
//! prefix on series_id, so the dashboard macro strip picks them up automatically.
//!
//! Ported from: reference/fincept_src/services/dbnomics/DBnomicsService.cpp

use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Pre-selected international macro series
// ---------------------------------------------------------------------------
// Each entry: (provider, dataset, series_code, human_label)
// The series_id stored in DB = "DBNOMICS:{provider}/{dataset}/{series_code}"

const DBNOMICS_SERIES: &[(&str, &str, &str, &str)] = &[
    // ECB Euribor 3-month rate — EU short-term benchmark
    ("ECB", "FM", "M.U2.EUR.RT.MM.EURIBOR3MD_.HSTA", "Euribor 3M"),
    // BIS central bank policy rate — PBoC (China)
    ("BIS", "WS_CBPOL", "M.CN", "PBoC Policy Rate"),
    // IMF World Economic Outlook — US real GDP growth forecast
    ("IMF", "WEO:2024-10", "USA.NGDP_RPCH", "US GDP Growth (IMF)"),
    // Eurostat HICP — Eurozone headline inflation (annual rate)
    ("Eurostat", "prc_hicp_manr", "M.RCH_A.CP00.EA", "Eurozone CPI YoY"),
    // OECD Composite Leading Indicator — US
    ("OECD", "MEI_CLI", "LOLITOAA.USA.M", "OECD CLI (US)"),
    // BIS total credit to private non-financial sector — US (% of GDP)
    ("BIS", "WS_TC", "Q.US.P.A.M.770.A", "US Total Credit/GDP"),
];

// ---------------------------------------------------------------------------
// Serde types — DBnomics observation response
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
struct DbnResponse {
    series: Option<DbnSeriesWrapper>,
}

#[derive(Debug, serde::Deserialize)]
struct DbnSeriesWrapper {
    docs: Vec<DbnDoc>,
}

#[derive(Debug, serde::Deserialize)]
struct DbnDoc {
    #[serde(default)]
    series_name: Option<String>,
    #[serde(default)]
    period: Vec<String>,
    #[serde(default)]
    value: Vec<serde_json::Value>, // can be number, null, or "NA"
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn fetch_all_dbnomics(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
) {
    for &(provider, dataset, series_code, label) in DBNOMICS_SERIES {
        match fetch_dbn_series(provider, dataset, series_code, label, &pool, &client).await {
            Ok(n) => {
                println!("[DBnomics] {label}: {n} new observations");
                crate::log_fetch(
                    &pool, "dbnomics", None,
                    &format!("{provider}/{dataset}/{series_code}"), "ok", None,
                ).await;
            }
            Err(e) => {
                eprintln!("[DBnomics] {label} error (skipping): {e:#}");
                crate::log_fetch(
                    &pool, "dbnomics", None,
                    &format!("{provider}/{dataset}/{series_code}"), "error",
                    Some(&e.to_string()),
                ).await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Private: fetch one series and upsert observations
// ---------------------------------------------------------------------------

async fn fetch_dbn_series(
    provider: &str,
    dataset: &str,
    series_code: &str,
    label: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
) -> Result<u64> {
    // DBnomics v22 observations endpoint (official domain: api.db.nomics.world)
    let url = format!(
        "https://api.db.nomics.world/v22/series/{provider}/{dataset}/{series_code}\
         ?observations=1&format=json"
    );

    let resp = client.get(&url).send().await
        .context("DBnomics request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        anyhow::bail!("DBnomics returned HTTP {status}: {body_text}");
    }

    let body: DbnResponse = resp.json().await
        .context("Failed to parse DBnomics response")?;

    let docs = body.series
        .and_then(|s| s.docs.into_iter().next())
        .context("No series data returned from DBnomics")?;

    let series_id = format!("DBNOMICS:{provider}/{dataset}/{series_code}");
    let series_name = docs.series_name
        .unwrap_or_else(|| label.to_string());

    let mut inserted = 0u64;

    for (period_str, val) in docs.period.iter().zip(docs.value.iter()) {
        // Parse the value — skip nulls and "NA"
        let numeric_value = match val {
            serde_json::Value::Number(n) => {
                n.as_f64().map(|f| rust_decimal::Decimal::try_from(f).ok()).flatten()
            }
            serde_json::Value::String(s) if s == "NA" || s == "nan" => None,
            serde_json::Value::Null => None,
            serde_json::Value::String(s) => s.parse::<rust_decimal::Decimal>().ok(),
            _ => None,
        };

        let Some(value) = numeric_value else { continue };

        // DBnomics periods can be "2024-01", "2024-Q1", or "2024-01-15"
        let obs_date = parse_dbn_period(period_str)?;

        let result = sqlx::query(
            "INSERT INTO macro_indicators (series_id, series_name, obs_date, value) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (series_id, obs_date) DO NOTHING",
        )
        .bind(&series_id)
        .bind(&series_name)
        .bind(obs_date)
        .bind(value)
        .execute(pool)
        .await
        .context("DB insert failed")?;

        inserted += result.rows_affected();
    }

    Ok(inserted)
}

/// Parse DBnomics period strings into NaiveDate.
/// Handles: "2024-01-15" (daily), "2024-01" (monthly -> 1st), "2024-Q1" (quarterly -> 1st of quarter).
fn parse_dbn_period(period: &str) -> Result<NaiveDate> {
    // Try full date first: "2024-01-15"
    if let Ok(d) = NaiveDate::parse_from_str(period, "%Y-%m-%d") {
        return Ok(d);
    }
    // Monthly: "2024-01"
    if let Ok(d) = NaiveDate::parse_from_str(&format!("{period}-01"), "%Y-%m-%d") {
        return Ok(d);
    }
    // Quarterly: "2024-Q1" -> January 1st, etc.
    if period.len() == 7 && period.contains("-Q") {
        let year: i32 = period[..4].parse().context("Invalid year in quarter")?;
        let q: u32 = period[6..].parse().context("Invalid quarter number")?;
        let month = match q {
            1 => 1,
            2 => 4,
            3 => 7,
            4 => 10,
            _ => anyhow::bail!("Invalid quarter: Q{q}"),
        };
        return NaiveDate::from_ymd_opt(year, month, 1)
            .context(format!("Invalid quarterly date: {period}"));
    }
    // Annual: "2024"
    if period.len() == 4 {
        let year: i32 = period.parse().context("Invalid year")?;
        return NaiveDate::from_ymd_opt(year, 1, 1)
            .context(format!("Invalid annual date: {period}"));
    }
    anyhow::bail!("Unrecognized DBnomics period format: {period}")
}
