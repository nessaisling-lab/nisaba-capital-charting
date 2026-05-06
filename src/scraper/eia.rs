//! Wave 7.7 — Energy Information Administration v2 API.
//!
//! Free key required (env var `EIA_API_KEY`). Skips gracefully if absent.
//! Headline energy series:
//!   - PET.RWTC.W — WTI Crude Spot Price ($/bbl, weekly)
//!   - PET.RBRTE.W — Brent Crude Spot Price ($/bbl, weekly)
//!   - NG.RNGWHHD.W — Henry Hub Natural Gas Spot ($/MMBtu, weekly)
//!   - PET.EMM_EPMR_PTE_NUS_DPG.W — US Regular Gasoline retail ($/gal)
//!   - ELEC.GEN.ALL-US-99.M — US total electricity generation (thousand MWh)

use anyhow::Result;
use std::sync::Arc;

const API_BASE: &str = "https://api.eia.gov/v2/seriesid";

const SERIES: &[(&str, &str, &str)] = &[
    ("PET.RWTC.W",                       "WTI Crude Spot",          "USD/bbl"),
    ("PET.RBRTE.W",                      "Brent Crude Spot",        "USD/bbl"),
    ("NG.RNGWHHD.W",                     "Henry Hub Natural Gas",   "USD/MMBtu"),
    ("PET.EMM_EPMR_PTE_NUS_DPG.W",       "US Regular Gasoline",     "USD/gal"),
];

pub async fn fetch_all(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>) -> Result<()> {
    let key = match std::env::var("EIA_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("[eia] EIA_API_KEY not set — skipping EIA fetch");
            return Ok(());
        }
    };
    let mut total = 0_i64;
    for (series_id, label, unit) in SERIES {
        let url = format!("{API_BASE}/{series_id}?api_key={key}&length=104");
        let resp = match client.get(&url)
            .header("User-Agent", "PursuitAstro/0.1")
            .send().await
        {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => { eprintln!("[eia] {series_id} HTTP {}", r.status()); continue; }
            Err(e) => { eprintln!("[eia] {series_id} request error: {e:#}"); continue; }
        };
        let body: serde_json::Value = match resp.json().await {
            Ok(b) => b,
            Err(e) => { eprintln!("[eia] {series_id} parse error: {e:#}"); continue; }
        };
        // body.response.data = [ {period: "YYYY-MM-DD", value: f64}, ... ]
        let Some(data) = body.pointer("/response/data").and_then(|d| d.as_array()) else {
            eprintln!("[eia] {series_id} no data array"); continue;
        };
        for row in data {
            let Some(period) = row.get("period").and_then(|p| p.as_str()) else { continue };
            // EIA can return value as f64 OR string-encoded float
            let value = row.get("value").and_then(|v| {
                v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            });
            let Some(value) = value else { continue };
            let date = match period.len() {
                10 => chrono::NaiveDate::parse_from_str(period, "%Y-%m-%d").ok(),
                7  => {
                    let parts: Vec<&str> = period.split('-').collect();
                    if parts.len() != 2 { None }
                    else {
                        let y: i32 = match parts[0].parse() { Ok(v) => v, Err(_) => continue };
                        let m: u32 = match parts[1].parse() { Ok(v) => v, Err(_) => continue };
                        chrono::NaiveDate::from_ymd_opt(y, m, 1)
                    }
                }
                _ => None,
            };
            let Some(date) = date else { continue };
            let r = sqlx::query(
                "INSERT INTO provider_observations
                    (provider, series_id, observation_date, value, label, region, unit, fetched_at)
                 VALUES ('eia', $1, $2, $3, $4, 'USA', $5, NOW())
                 ON CONFLICT (provider, series_id, region, observation_date)
                 DO UPDATE SET value = EXCLUDED.value, fetched_at = NOW()",
            )
            .bind(series_id)
            .bind(date)
            .bind(value)
            .bind(label)
            .bind(unit)
            .execute(pool.as_ref())
            .await;
            if r.is_ok() { total += 1; }
        }
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }
    println!("[eia] done — {} observations stored", total);
    Ok(())
}
