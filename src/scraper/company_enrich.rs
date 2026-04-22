//! IPO date enrichment via Alpha Vantage OVERVIEW endpoint.
//!
//! Runs nightly after the ticker seed. Finds company_metadata rows where
//! ipo_date IS NULL (inserted by Polygon without a list_date), calls
//! AV OVERVIEW to get IPODate, updates the row, then triggers natal chart
//! computation for that ticker.
//!
//! Budget: consumes at most 10 AV calls per run, leaving budget for prices.
//! At 10/day, a 10k-ticker universe is fully enriched in ~1000 days — but
//! in practice the most-searched tickers get enriched first and the watchlist
//! is always full within the first run.

use anyhow::Result;
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;

const MAX_ENRICH_PER_RUN: i64 = 10;

#[derive(Debug, Deserialize)]
struct AvOverviewResponse {
    #[serde(rename = "IPODate")]
    ipo_date: Option<String>,
}

/// Fills in missing `ipo_date` values using Alpha Vantage OVERVIEW.
/// Prioritises WATCHLIST tickers first, then alphabetical order.
pub async fn enrich_missing_ipo_dates(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
) {
    // Count how many AV calls already used today
    let calls_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fetch_log \
         WHERE source = 'alpha_vantage' AND fetched_at::date = CURRENT_DATE",
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0);

    // Prices and sentiment have already run at this point in the pipeline.
    // Leave just 1 call as safety margin for any stragglers.
    let budget = (25_i64 - calls_today - 1).max(0).min(MAX_ENRICH_PER_RUN);
    if budget == 0 {
        println!("[Enrich] AV budget exhausted today ({calls_today} calls used). Skipping.");
        return;
    }

    // Fetch tickers that need enrichment — watchlist first, then alpha
    let order = crate::enrich_common::watchlist_priority_sql();
    let to_enrich: Vec<String> = sqlx::query_scalar(&format!(
        "SELECT ticker FROM company_metadata \
         WHERE ipo_date IS NULL \
         ORDER BY {order} \
         LIMIT {budget}"
    ))
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    if to_enrich.is_empty() {
        println!("[Enrich] All tickers have IPO dates. Nothing to enrich.");
        return;
    }

    println!(
        "[Enrich] Enriching {} ticker(s) with AV OVERVIEW (budget: {budget} calls today)...",
        to_enrich.len()
    );

    for ticker in &to_enrich {
        match fetch_overview_ipo_date(ticker, &client, &api_key).await {
            Ok(Some(ipo_date)) => {
                // Update the row and seed the natal chart
                let updated = sqlx::query(
                    "UPDATE company_metadata SET ipo_date = $1 WHERE ticker = $2",
                )
                .bind(ipo_date)
                .bind(ticker)
                .execute(pool.as_ref())
                .await;

                match updated {
                    Ok(_) => {
                        crate::enrich_common::seed_one_natal_chart(pool.as_ref(), ticker, ipo_date).await;
                        println!("[Enrich] {ticker}: IPO date set to {ipo_date} — natal chart seeded");
                    }
                    Err(e) => eprintln!("[Enrich] {ticker}: DB update failed: {e}"),
                }
            }
            Ok(None) => println!("[Enrich] {ticker}: AV returned no IPODate — will retry tomorrow"),
            Err(e)   => eprintln!("[Enrich] {ticker}: AV OVERVIEW error: {e}"),
        }

        crate::log_fetch(
            pool.as_ref(), "alpha_vantage", Some(ticker), "overview", "ok", None
        ).await;

        // 1.2s between calls to respect AV rate limit
        tokio::time::sleep(std::time::Duration::from_millis(1_200)).await;
    }

    println!("[Enrich] Done.");
}

async fn fetch_overview_ipo_date(
    ticker: &str,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<Option<NaiveDate>> {
    let url = format!(
        "https://www.alphavantage.co/query?function=OVERVIEW&symbol={ticker}&apikey={api_key}"
    );

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("HTTP {}", resp.status());
    }

    let body: serde_json::Value = resp.json().await?;

    // AV returns rate limit messages as JSON objects with "Note" or "Information"
    if body.get("Note").is_some() || body.get("Information").is_some() {
        anyhow::bail!("AV rate limit hit");
    }

    // Empty response {} means AV doesn't know this ticker
    if body.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        return Ok(None);
    }

    let overview: AvOverviewResponse = serde_json::from_value(body)?;

    let date = overview
        .ipo_date
        .as_deref()
        .filter(|s| !s.is_empty() && *s != "None" && *s != "null")
        .and_then(|s| s.parse::<NaiveDate>().ok());

    Ok(date)
}

