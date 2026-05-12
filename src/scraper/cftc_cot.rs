//! Wave 7.8 — CFTC Commitments of Traders (COT) reports.
//!
//! Public dataset on data.nasdaq.com / CFTC weekly. Tracks net long/short
//! positioning of large-trader categories on key futures markets.
//! Stored as `provider = 'cftc_cot'` with custom `series_id` per
//! market+category.
//!
//! Markets tracked (financial focus):
//!   - SP500_EMINI    — S&P 500 e-mini
//!   - NASDAQ_EMINI   — Nasdaq 100 e-mini
//!   - DXY            — US Dollar Index
//!   - GOLD           — Gold (COMEX)
//!   - WTI            — WTI crude (NYMEX)
//!
//! Public CSV at cftc.gov/dea/newcot/c_year.txt (legacy format) or via
//! the disaggregated reports. We use Quandl's free CFTC mirror since
//! it's more developer-friendly. NB: no API key required for these.

use anyhow::{Context, Result};
use std::sync::Arc;

// CFTC publishes weekly TXT files; URL format is consistent.
// Latest "current year" file: https://www.cftc.gov/files/dea/cotarchives/2026/futures/c_year.txt
// Easier: use the public commitments-of-traders CSV mirror at quandl/nasdaq.
// For a pure-Rust no-key path, we hit the cftc.gov public report.
const CFTC_TFF_URL: &str = "https://www.cftc.gov/dea/newcot/f_year.txt";

// Map CFTC market name → our series_id
const MARKETS: &[(&str, &str)] = &[
    ("E-MINI S&P 500", "sp500_emini"),
    ("NASDAQ-100 E-MINI", "nasdaq_emini"),
    ("U.S. DOLLAR INDEX", "dxy"),
    ("GOLD - COMMODITY EXCHANGE INC.", "gold_comex"),
    ("WTI FINANCIAL CRUDE OIL", "wti_financial"),
];

pub async fn fetch_all(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>) -> Result<()> {
    let resp = match client.get(CFTC_TFF_URL)
        .header("User-Agent", "NisabaEngine/0.1")
        .send().await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            eprintln!("[cftc_cot] HTTP {} — CFTC may have moved file. Skipping.", r.status());
            return Ok(());
        }
        Err(e) => {
            eprintln!("[cftc_cot] request error: {e:#}");
            return Ok(());
        }
    };
    let csv_text = resp.text().await.context("CFTC CSV body")?;
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(csv_text.as_bytes());
    // CFTC TFF schema is fixed: column 0 = Market_and_Exchange_Names,
    // 2 = Report_Date_as_YYYY-MM-DD, plus dozens of position columns.
    // We extract Asset_Mgr_Net (cols 35-36 in disaggregated) — but
    // for legacy TFF, the simpler field is Noncomm_Long - Noncomm_Short.
    let headers = rdr.headers().context("CFTC headers")?.clone();
    let market_col = headers.iter().position(|h| h.contains("Market_and_Exchange_Names"));
    let date_col = headers.iter().position(|h| h.contains("Report_Date_as_YYYY-MM-DD"));
    let nc_long_col = headers.iter().position(|h| h.contains("NonComm_Positions_Long_All"));
    let nc_short_col = headers.iter().position(|h| h.contains("NonComm_Positions_Short_All"));

    let (Some(mc), Some(dc), Some(nlc), Some(nsc)) = (market_col, date_col, nc_long_col, nc_short_col) else {
        eprintln!("[cftc_cot] expected columns not found in CSV — schema may have changed");
        return Ok(());
    };

    let mut total = 0_i64;
    for record in rdr.records() {
        let record = match record { Ok(r) => r, Err(_) => continue };
        let market = record.get(mc).unwrap_or("");
        let Some((_, series_id)) = MARKETS.iter().find(|(name, _)| market.starts_with(name)) else {
            continue;
        };
        let date_str = record.get(dc).unwrap_or("");
        let date = match chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };
        let long: f64 = record.get(nlc).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let short: f64 = record.get(nsc).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let net = long - short;
        let r = sqlx::query(
            "INSERT INTO provider_observations
                (provider, series_id, observation_date, value, label, region, unit, fetched_at)
             VALUES ('cftc_cot', $1, $2, $3, $4, 'USA', 'contracts', NOW())
             ON CONFLICT (provider, series_id, region, observation_date)
             DO UPDATE SET value = EXCLUDED.value, fetched_at = NOW()",
        )
        .bind(format!("{}_net_noncomm", series_id))
        .bind(date)
        .bind(net)
        .bind(format!("{} non-commercial net positioning", market))
        .execute(pool.as_ref())
        .await;
        if r.is_ok() { total += 1; }
    }
    println!("[cftc_cot] done — {} positioning observations stored", total);
    Ok(())
}
