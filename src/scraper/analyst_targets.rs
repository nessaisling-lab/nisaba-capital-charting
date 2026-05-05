//! Analyst price targets via Finnhub /stock/price-target (Wave 6.A3).
//!
//! Per-ticker fetch returns low/median/high targets aggregated across
//! analysts plus the analyst count. Stored once per ticker (latest
//! fetch wins) — Finnhub updates these maybe weekly, no need to keep
//! a daily history.
//!
//! Free tier: 60 calls/min shared with other Finnhub modules. Budget
//! 30 tickers per run leaves headroom for news + recommendations.

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::Deserialize;
use std::sync::Arc;

const MAX_PER_RUN: usize = 30;

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct FinnhubPriceTarget {
    target_high:   Option<f64>,
    target_low:    Option<f64>,
    target_median: Option<f64>,
    #[serde(rename = "numberOfAnalysts")]
    number_of_analysts: Option<i32>,
}

pub async fn fetch_analyst_targets(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
) {
    let stale: Vec<String> = sqlx::query_scalar(
        "SELECT cm.ticker FROM company_metadata cm \
         WHERE NOT EXISTS ( \
             SELECT 1 FROM analyst_targets at \
             WHERE at.ticker = cm.ticker \
               AND at.last_updated > NOW() - INTERVAL '7 days' \
         ) \
         ORDER BY cm.ticker \
         LIMIT $1",
    )
    .bind(MAX_PER_RUN as i32)
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    if stale.is_empty() {
        println!("[Analyst Targets] All tickers fresh (within 7 days). Skipping.");
        return;
    }

    println!("[Analyst Targets] Fetching {} stale ticker(s)...", stale.len());
    let mut consecutive_403s = 0;
    for ticker in &stale {
        match fetch_one(ticker, &client, &api_key).await {
            Ok(Some(t)) => {
                let _ = upsert(&pool, ticker, &t).await;
                crate::log_fetch(&pool, "finnhub", Some(ticker), "price-target", "ok", None).await;
                consecutive_403s = 0;
            }
            Ok(None) => {
                println!("[Analyst Targets] {ticker}: no analyst coverage");
                consecutive_403s = 0;
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("403") {
                    consecutive_403s += 1;
                    // Bail after 3 consecutive 403s — endpoint requires paid tier
                    if consecutive_403s >= 3 {
                        eprintln!("[Analyst Targets] 3+ consecutive 403s — Finnhub /price-target is paid-tier only. Skipping remaining {} ticker(s).",
                            stale.len() - (stale.iter().position(|t| t == ticker).unwrap_or(0) + 1));
                        break;
                    }
                }
                eprintln!("[Analyst Targets] {ticker}: {e}");
                crate::log_fetch(&pool, "finnhub", Some(ticker), "price-target", "error",
                    Some(&msg)).await;
            }
        }
        // 1.1 sec/call respects 60/min Finnhub limit with headroom.
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    }
    println!("[Analyst Targets] Done.");
}

async fn fetch_one(
    ticker: &str,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<Option<FinnhubPriceTarget>> {
    let url = format!(
        "https://finnhub.io/api/v1/stock/price-target?symbol={ticker}&token={api_key}"
    );
    let resp = client.get(&url).send().await
        .context("Finnhub price-target request failed")?;
    if !resp.status().is_success() {
        anyhow::bail!("Finnhub price-target HTTP {}", resp.status());
    }

    let body: FinnhubPriceTarget = resp.json().await
        .context("Finnhub price-target JSON parse failed")?;

    // No analyst coverage = all-None response
    if body.target_high.is_none() && body.target_median.is_none() && body.target_low.is_none() {
        return Ok(None);
    }
    Ok(Some(body))
}

async fn upsert(
    pool: &sqlx::PgPool,
    ticker: &str,
    t: &FinnhubPriceTarget,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO analyst_targets \
         (ticker, fetch_date, low_target, median_target, high_target, n_analysts, last_updated) \
         VALUES ($1, CURRENT_DATE, $2, $3, $4, $5, NOW()) \
         ON CONFLICT (ticker) DO UPDATE SET \
             fetch_date    = EXCLUDED.fetch_date, \
             low_target    = EXCLUDED.low_target, \
             median_target = EXCLUDED.median_target, \
             high_target   = EXCLUDED.high_target, \
             n_analysts    = EXCLUDED.n_analysts, \
             last_updated  = NOW()",
    )
    .bind(ticker)
    .bind(t.target_low.and_then(Decimal::from_f64))
    .bind(t.target_median.and_then(Decimal::from_f64))
    .bind(t.target_high.and_then(Decimal::from_f64))
    .bind(t.number_of_analysts)
    .execute(pool)
    .await
    .context("Failed to upsert analyst_targets")?;
    Ok(())
}

/// Single-ticker variant for FetchThisTicker flow.
pub async fn fetch_one_and_store(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
    ticker: &str,
) -> Result<()> {
    if let Some(t) = fetch_one(ticker, &client, &api_key).await? {
        upsert(&pool, ticker, &t).await?;
        println!("[{ticker}] Analyst targets stored");
    }
    Ok(())
}
