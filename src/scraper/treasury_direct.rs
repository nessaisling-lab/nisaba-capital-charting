//! Wave 7.3 — US Treasury Direct daily yield curve.
//!
//! Public XML feed at home.treasury.gov, no key. Fetches the 12 standard
//! constant-maturity Treasury rates (1mo, 2mo, 3mo, 4mo, 6mo, 1yr, 2yr,
//! 3yr, 5yr, 7yr, 10yr, 20yr, 30yr) for the latest 30 trading days.
//!
//! Already covered by FRED for some maturities, but this is the
//! authoritative US Treasury source — zero rate-limit, more reliable
//! than FRED's API for daily yield-curve scrapes.

use anyhow::{Context, Result};
use std::sync::Arc;

const FEED_URL: &str = "https://home.treasury.gov/resource-center/data-chart-center/interest-rates/daily-treasury-rates.csv/all/";

const MATURITIES: &[(&str, &str)] = &[
    ("1 Mo",  "treasury_1m"),  ("2 Mo",  "treasury_2m"),
    ("3 Mo",  "treasury_3m"),  ("4 Mo",  "treasury_4m"),
    ("6 Mo",  "treasury_6m"),  ("1 Yr",  "treasury_1y"),
    ("2 Yr",  "treasury_2y"),  ("3 Yr",  "treasury_3y"),
    ("5 Yr",  "treasury_5y"),  ("7 Yr",  "treasury_7y"),
    ("10 Yr", "treasury_10y"), ("20 Yr", "treasury_20y"),
    ("30 Yr", "treasury_30y"),
];

pub async fn fetch_all(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>) -> Result<()> {
    // Year-scoped URL: /all/?type=daily_treasury_yield_curve&field_tdr_date_value={year}
    let year = chrono::Datelike::year(&chrono::Local::now().date_naive());
    let url = format!(
        "{FEED_URL}?type=daily_treasury_yield_curve&field_tdr_date_value={year}"
    );
    let resp = client.get(&url)
        .header("User-Agent", "NisabaEngine/0.1")
        .send().await
        .context("Treasury Direct CSV request")?;
    if !resp.status().is_success() {
        anyhow::bail!("Treasury Direct HTTP {}", resp.status());
    }
    let csv_text = resp.text().await.context("Treasury Direct CSV body")?;
    let mut rdr = csv::Reader::from_reader(csv_text.as_bytes());
    let headers = rdr.headers().context("Treasury Direct CSV headers")?.clone();

    // Map header column indices to (label, series_id)
    let mut col_map: Vec<(usize, &str, &str)> = Vec::new();
    for (i, h) in headers.iter().enumerate() {
        for (label, sid) in MATURITIES {
            if h.eq_ignore_ascii_case(label) {
                col_map.push((i, label, sid));
            }
        }
    }
    let date_col = headers.iter().position(|h| h.eq_ignore_ascii_case("Date"))
        .context("Treasury Direct CSV: Date column missing")?;

    let mut total = 0_i64;
    for (row_idx, record) in rdr.records().enumerate() {
        if row_idx > 30 { break; } // last 30 trading days
        let record = match record {
            Ok(r) => r,
            Err(_) => continue,
        };
        let date_str = match record.get(date_col) {
            Some(s) => s,
            None => continue,
        };
        // Format: "MM/DD/YYYY"
        let date = match chrono::NaiveDate::parse_from_str(date_str, "%m/%d/%Y") {
            Ok(d) => d,
            Err(_) => continue,
        };
        for (col, label, series_id) in &col_map {
            let cell = match record.get(*col) {
                Some(s) => s,
                None => continue,
            };
            let value: f64 = match cell.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            sqlx::query(
                "INSERT INTO provider_observations
                    (provider, series_id, observation_date, value, label, region, unit, fetched_at)
                 VALUES ('treasury_direct', $1, $2, $3, $4, 'USA', '%', NOW())
                 ON CONFLICT (provider, series_id, region, observation_date)
                 DO UPDATE SET value = EXCLUDED.value, fetched_at = NOW()",
            )
            .bind(series_id)
            .bind(date)
            .bind(value)
            .bind(format!("US Treasury {} yield", label))
            .execute(pool.as_ref())
            .await
            .context("Treasury Direct store")?;
            total += 1;
        }
    }
    println!("[treasury_direct] done — {} yield-curve observations stored", total);
    Ok(())
}
