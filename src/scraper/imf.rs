//! Wave 7.4 — IMF World Economic Outlook (WEO) via SDMX/JSON.
//!
//! Free public API at dataservices.imf.org. Fetches headline IMF
//! indicators for major economies. Series IDs reuse World Bank-style
//! codes for cross-source consistency where possible.
//!
//! Indicators:
//!   - NGDP_RPCH      — Real GDP growth (annual %)
//!   - PCPIPCH        — Inflation, average consumer prices (%)
//!   - LUR            — Unemployment rate (%)
//!   - GGXWDG_NGDP    — Govt gross debt (% of GDP)
//!   - BCA_NGDPD      — Current account balance (% of GDP)
//!
//! Countries: G7 + BRICS (matches World Bank for cross-validation).

use anyhow::Result;
use std::sync::Arc;

const API_BASE: &str = "https://www.imf.org/external/datamapper/api/v1";

const INDICATORS: &[(&str, &str)] = &[
    ("NGDP_RPCH",      "Real GDP growth (annual %)"),
    ("PCPIPCH",        "Inflation, avg consumer prices (%)"),
    ("LUR",            "Unemployment rate (%)"),
    ("GGXWDG_NGDP",    "Govt gross debt (% of GDP)"),
    ("BCA_NGDPD",      "Current account balance (% of GDP)"),
];

const COUNTRIES: &[&str] = &[
    "USA", "CHN", "JPN", "DEU", "GBR", "FRA", "IND", "ITA", "CAN", "BRA", "RUS",
];

pub async fn fetch_all(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>) -> Result<()> {
    let mut total = 0_i64;
    for (series_id, label) in INDICATORS {
        // IMF Datamapper: /api/v1/{indicator}/{country1}/{country2}/...
        let countries_path = COUNTRIES.join("/");
        let url = format!("{API_BASE}/{series_id}/{countries_path}");
        let resp = match client.get(&url)
            .header("User-Agent", "PursuitAstro/0.1")
            .send().await
        {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => { eprintln!("[imf] {series_id} HTTP {}", r.status()); continue; }
            Err(e) => { eprintln!("[imf] {series_id} request error: {e:#}"); continue; }
        };
        let body: serde_json::Value = match resp.json().await {
            Ok(b) => b,
            Err(e) => { eprintln!("[imf] {series_id} parse error: {e:#}"); continue; }
        };
        // body.values.{indicator}.{country} = { "1980": 1.23, "1981": ... }
        let values = body.get("values")
            .and_then(|v| v.get(series_id));
        let Some(values) = values else { continue };
        let Some(country_map) = values.as_object() else { continue };
        for (country, year_map) in country_map {
            let Some(years) = year_map.as_object() else { continue };
            for (year_str, val) in years {
                let Some(value) = val.as_f64() else { continue };
                let Ok(year) = year_str.parse::<i32>() else { continue };
                let Some(date) = chrono::NaiveDate::from_ymd_opt(year, 12, 31) else { continue };
                let r = sqlx::query(
                    "INSERT INTO provider_observations
                        (provider, series_id, observation_date, value, label, region, unit, fetched_at)
                     VALUES ('imf', $1, $2, $3, $4, $5, '%', NOW())
                     ON CONFLICT (provider, series_id, region, observation_date)
                     DO UPDATE SET value = EXCLUDED.value, fetched_at = NOW()",
                )
                .bind(series_id)
                .bind(date)
                .bind(value)
                .bind(label)
                .bind(country)
                .execute(pool.as_ref())
                .await;
                if r.is_ok() { total += 1; }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    let _ = INDICATORS.len();
    println!("[imf] done — {} observations stored", total);
    let _ = (pool, client);
    Ok(())
}
