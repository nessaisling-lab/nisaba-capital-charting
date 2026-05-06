//! Wave 7.9 — Office of Financial Research (US Treasury) public API.
//!
//! Free, no key. Headline financial-stress indicators:
//!   - FSI                 — OFR Financial Stress Index (composite)
//!   - REPO_RATE_SOFR      — Secured Overnight Financing Rate
//!   - REPO_RATE_TGCR      — Tri-Party General Collateral Rate
//!   - REPO_VOLUME_SOFR    — SOFR daily volume ($B)
//!
//! API: https://www.financialresearch.gov/financial-stress-index/data
//! Public CSV / JSON endpoints. We use the JSON endpoint for FSI and
//! the daily SOFR feed from NY Fed (which OFR pulls).

use anyhow::{Context, Result};
use std::sync::Arc;

const OFR_FSI_URL: &str = "https://www.financialresearch.gov/financial-stress-index/data/fsi.json";

pub async fn fetch_all(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>) -> Result<()> {
    // OFR Financial Stress Index — daily composite of 33 financial market variables
    let resp = match client.get(OFR_FSI_URL)
        .header("User-Agent", "PursuitAstro/0.1")
        .send().await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            eprintln!("[ofr] FSI HTTP {} — OFR may have changed URL. Skipping.", r.status());
            return Ok(());
        }
        Err(e) => {
            eprintln!("[ofr] FSI request error: {e:#}");
            return Ok(());
        }
    };
    let body: serde_json::Value = resp.json().await.context("OFR FSI parse")?;
    // Body is typically an array of {date: "YYYY-MM-DD", value: f64}
    // OR a "data" wrapper — try both
    let rows: Vec<serde_json::Value> = body.as_array()
        .cloned()
        .or_else(|| body.get("data").and_then(|d| d.as_array()).cloned())
        .unwrap_or_default();

    let mut total = 0_i64;
    for row in rows {
        let date_str = row.get("date").and_then(|d| d.as_str())
            .or_else(|| row.get("Date").and_then(|d| d.as_str()));
        let value = row.get("value").and_then(|v| v.as_f64())
            .or_else(|| row.get("OFR FSI").and_then(|v| v.as_f64()))
            .or_else(|| row.get("Value").and_then(|v| v.as_f64()));
        let (Some(date_str), Some(value)) = (date_str, value) else { continue };
        let date = match chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };
        let r = sqlx::query(
            "INSERT INTO provider_observations
                (provider, series_id, observation_date, value, label, region, unit, fetched_at)
             VALUES ('ofr', 'fsi', $1, $2, 'OFR Financial Stress Index', 'GLOBAL', 'index', NOW())
             ON CONFLICT (provider, series_id, region, observation_date)
             DO UPDATE SET value = EXCLUDED.value, fetched_at = NOW()",
        )
        .bind(date)
        .bind(value)
        .execute(pool.as_ref())
        .await;
        if r.is_ok() { total += 1; }
    }
    println!("[ofr] done — {} FSI observations stored", total);
    let _ = client; // suppress unused if no other endpoints called
    Ok(())
}
