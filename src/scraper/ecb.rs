//! Wave 7.5 — European Central Bank Statistical Data Warehouse.
//!
//! ECB SDMX REST API at data-api.ecb.europa.eu. Free, no key.
//! Fetches Eurozone macro/financial series:
//!   - FM.M.U2.EUR.RT.MM.EURIBOR3MD_.HSTA   — Euribor 3-month
//!   - FM.D.U2.EUR.4F.KR.MRR_FR.LEV         — ECB Main Refinancing Rate
//!   - EXR.M.USD.EUR.SP00.A                  — EUR/USD exchange rate
//!   - EXR.M.GBP.EUR.SP00.A                  — EUR/GBP exchange rate
//!   - EXR.M.CHF.EUR.SP00.A                  — EUR/CHF exchange rate
//!   - EXR.M.JPY.EUR.SP00.A                  — EUR/JPY exchange rate

use anyhow::Result;
use std::sync::Arc;

const API_BASE: &str = "https://data-api.ecb.europa.eu/service/data";

const SERIES: &[(&str, &str, &str, &str, &str)] = &[
    // (flow, key, our_series_id, label, unit)
    ("FM",  "M.U2.EUR.RT.MM.EURIBOR3MD_.HSTA", "euribor_3m",
        "Euribor 3-month rate", "%"),
    ("FM",  "D.U2.EUR.4F.KR.MRR_FR.LEV",       "ecb_mrr",
        "ECB Main Refinancing Rate", "%"),
    ("EXR", "D.USD.EUR.SP00.A",                "eur_usd",
        "EUR/USD exchange rate", "EUR"),
    ("EXR", "D.GBP.EUR.SP00.A",                "eur_gbp",
        "EUR/GBP exchange rate", "EUR"),
    ("EXR", "D.CHF.EUR.SP00.A",                "eur_chf",
        "EUR/CHF exchange rate", "EUR"),
    ("EXR", "D.JPY.EUR.SP00.A",                "eur_jpy",
        "EUR/JPY exchange rate", "EUR"),
];

pub async fn fetch_all(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>) -> Result<()> {
    let mut total = 0_i64;
    for (flow, key, series_id, label, unit) in SERIES {
        // ECB SDMX: /service/data/{flow}/{key}?format=jsondata&detail=dataonly&lastNObservations=60
        let url = format!(
            "{API_BASE}/{flow}/{key}?format=jsondata&detail=dataonly&lastNObservations=60"
        );
        let resp = match client.get(&url)
            .header("User-Agent", "NisabaEngine/0.1")
            .header("Accept", "application/json")
            .send().await
        {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => { eprintln!("[ecb] {series_id} HTTP {}", r.status()); continue; }
            Err(e) => { eprintln!("[ecb] {series_id} request error: {e:#}"); continue; }
        };
        let body: serde_json::Value = match resp.json().await {
            Ok(b) => b,
            Err(e) => { eprintln!("[ecb] {series_id} parse error: {e:#}"); continue; }
        };
        // SDMX-JSON structure: dataSets[0].series.{seriesKey}.observations.{idx} = [value]
        // dimensions.observation[0].values[idx].{id} = "2024-08"
        let dim_values = body.pointer("/structure/dimensions/observation/0/values")
            .and_then(|v| v.as_array());
        let observations = body.pointer("/dataSets/0/series/0:0:0:0:0:0:0/observations")
            .and_then(|v| v.as_object())
            .or_else(|| body.pointer("/dataSets/0/series").and_then(|s| s.as_object())
                        .and_then(|m| m.values().next())
                        .and_then(|v| v.get("observations"))
                        .and_then(|o| o.as_object()));
        let (Some(dims), Some(obs)) = (dim_values, observations) else {
            eprintln!("[ecb] {series_id} unexpected JSON shape");
            continue;
        };
        for (idx_str, val_arr) in obs {
            let Ok(idx) = idx_str.parse::<usize>() else { continue };
            let Some(dim) = dims.get(idx) else { continue };
            let Some(date_str) = dim.get("id").and_then(|s| s.as_str()) else { continue };
            let Some(value) = val_arr.as_array().and_then(|a| a.first()).and_then(|v| v.as_f64()) else { continue };
            // Date format from ECB: "YYYY-MM" (monthly) or "YYYY-MM-DD" (daily)
            let date = if date_str.len() == 7 {
                let parts: Vec<&str> = date_str.split('-').collect();
                if parts.len() != 2 { continue }
                let Ok(y) = parts[0].parse::<i32>() else { continue };
                let Ok(m) = parts[1].parse::<u32>() else { continue };
                match chrono::NaiveDate::from_ymd_opt(y, m, 1) { Some(d) => d, None => continue }
            } else {
                match chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") { Ok(d) => d, Err(_) => continue }
            };
            let r = sqlx::query(
                "INSERT INTO provider_observations
                    (provider, series_id, observation_date, value, label, region, unit, fetched_at)
                 VALUES ('ecb', $1, $2, $3, $4, 'EUR', $5, NOW())
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
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    }
    println!("[ecb] done — {} observations stored", total);
    let _ = (pool, client);
    Ok(())
}
