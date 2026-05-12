//! Wave 7.10 — SEC EDGAR recent-filings firehose (buffer slot).
//!
//! Public endpoint at sec.gov, no key. Pulls the latest cross-company
//! 8-K + 10-K + 10-Q filings (EDGAR's "full-text search" feed) so the
//! Research tab has a market-wide breaking-news surface alongside the
//! per-ticker filings already covered by `edgar.rs`.
//!
//! Stored in `provider_observations` as a count metric (filings per
//! day, by form type) so cross-source dashboards can see filing
//! activity over time.

use anyhow::{Context, Result};
use std::sync::Arc;

const SEC_LATEST_URL: &str =
    "https://www.sec.gov/cgi-bin/browse-edgar?action=getcurrent&type=&company=&dateb=&owner=include&count=40&action=getcurrent";

pub async fn fetch_all(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>) -> Result<()> {
    // SEC requires a descriptive User-Agent
    let resp = match client.get(SEC_LATEST_URL)
        .header("User-Agent", "NisabaEngine PursuitNYC/0.1 contact@pursuit.org")
        .send().await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            eprintln!("[sec_recent] HTTP {}", r.status());
            return Ok(());
        }
        Err(e) => {
            eprintln!("[sec_recent] request error: {e:#}");
            return Ok(());
        }
    };
    let body = resp.text().await.context("SEC latest body")?;
    // Page is HTML with <pre> blocks; we count occurrences per form type
    // for a coarse "filing pulse" signal. Fine-grained parsing is handled
    // by the existing edgar.rs per-ticker module.
    let today = chrono::Local::now().date_naive();
    let mut counts = std::collections::HashMap::<&str, i64>::new();
    for form in &["8-K", "10-K", "10-Q", "S-1", "13D", "13G", "4"] {
        let count = body.matches(&format!("/{}/", form)).count() as i64
            + body.matches(&format!(">{}<", form)).count() as i64;
        if count > 0 {
            counts.insert(form, count);
        }
    }
    let mut total = 0_i64;
    for (form, count) in counts {
        let r = sqlx::query(
            "INSERT INTO provider_observations
                (provider, series_id, observation_date, value, label, region, unit, fetched_at)
             VALUES ('sec_recent', $1, $2, $3, $4, 'USA', 'count', NOW())
             ON CONFLICT (provider, series_id, region, observation_date)
             DO UPDATE SET value = EXCLUDED.value, fetched_at = NOW()",
        )
        .bind(format!("filings_{}", form.replace('-', "_").to_lowercase()))
        .bind(today)
        .bind(count as f64)
        .bind(format!("SEC {} filings (last-40 firehose)", form))
        .execute(pool.as_ref())
        .await;
        if r.is_ok() { total += 1; }
    }
    println!("[sec_recent] done — {} form-type counts stored", total);
    Ok(())
}
