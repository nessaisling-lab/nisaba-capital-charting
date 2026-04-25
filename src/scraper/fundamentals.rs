//! Fundamental financial metrics scraper via FMP.
//!
//! Fetches `/v3/key-metrics-ttm/{ticker}` and `/v3/ratios-ttm/{ticker}` from
//! Financial Modeling Prep.  Stores one row per ticker per day in
//! `fundamental_metrics`.
//!
//! Budget: shares the FMP free tier (250 req/day) with `fmp_enrich`.  Each
//! ticker costs 2 calls (key-metrics + ratios), so we fetch up to 60 tickers
//! per run.  Watchlist tickers are prioritized.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::Arc;

/// Max tickers per daily run (2 calls each = 120 FMP calls).
const MAX_TICKERS_PER_RUN: usize = 60;

// ---------------------------------------------------------------------------
// FMP response shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // `earnings_yield_ttm` deserialized from FMP API, used in future agent context
struct FmpKeyMetrics {
    market_cap_ttm:             Option<f64>,
    pe_ratio_ttm:               Option<f64>,
    pb_ratio_ttm:               Option<f64>,
    price_to_sales_ratio_ttm:   Option<f64>,
    enterprise_value_over_ebitda_ttm: Option<f64>,
    peg_ratio_ttm:              Option<f64>,
    price_to_free_cash_flows_ttm: Option<f64>,
    roe_ttm:                    Option<f64>,
    roa_ttm:                    Option<f64>,
    debt_to_equity_ttm:         Option<f64>,
    current_ratio_ttm:          Option<f64>,
    free_cash_flow_per_share_ttm: Option<f64>,
    operating_cash_flow_per_share_ttm: Option<f64>,
    revenue_per_share_ttm:      Option<f64>,
    net_income_per_share_ttm:   Option<f64>,
    dividend_yield_ttm:         Option<f64>,
    earnings_yield_ttm:         Option<f64>,
    shares_outstanding:         Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct FmpRatios {
    net_profit_margin_ttm:     Option<f64>,
    operating_profit_margin_ttm: Option<f64>,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Fetch fundamental metrics for watchlist + astro-prioritized tickers.
pub async fn fetch_fundamentals(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    fmp_key: Arc<String>,
) {
    // Check FMP budget remaining today
    let fmp_calls_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fetch_log \
         WHERE source = 'fmp' AND fetched_at::date = CURRENT_DATE",
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0);

    // Reserve 120 calls for fundamentals (60 tickers x 2 calls each).
    // If enrichment already used most of the budget, reduce accordingly.
    let remaining = (250_i64 - fmp_calls_today).max(0);
    let budget = (remaining / 2).min(MAX_TICKERS_PER_RUN as i64) as usize;

    if budget == 0 {
        println!("[Fundamentals] FMP daily budget exhausted ({fmp_calls_today} calls). Skipping.");
        return;
    }

    // Already fetched today?  Skip those tickers.
    let order = crate::enrich_common::watchlist_priority_sql();
    let tickers: Vec<String> = sqlx::query_scalar(&format!(
        "SELECT cm.ticker FROM company_metadata cm
         WHERE NOT EXISTS (
             SELECT 1 FROM fundamental_metrics fm
             WHERE fm.ticker = cm.ticker AND fm.fetch_date = CURRENT_DATE
         )
         ORDER BY {order}
         LIMIT {budget}"
    ))
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    if tickers.is_empty() {
        println!("[Fundamentals] All tickers already fetched today. Skipping.");
        return;
    }

    println!(
        "[Fundamentals] Fetching metrics for {} ticker(s) (budget: {budget})...",
        tickers.len()
    );

    let mut ok_count = 0usize;
    let mut err_count = 0usize;

    for ticker in &tickers {
        match fetch_and_store(ticker, &pool, &client, &fmp_key).await {
            Ok(true) => ok_count += 1,
            Ok(false) => {} // no data from FMP
            Err(ref e) if e.to_string().contains("403") => {
                eprintln!(
                    "[Fundamentals] HTTP 403 — FMP paid plan required. \
                     Stopping ({} remaining).",
                    tickers.len().saturating_sub(ok_count + err_count + 1)
                );
                break;
            }
            Err(e) => {
                eprintln!("[Fundamentals] {ticker}: {e}");
                err_count += 1;
            }
        }

        // ~4 calls/sec to stay within rate limits
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    println!("[Fundamentals] Done — {ok_count} fetched, {err_count} errors.");
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

pub(crate) async fn fetch_and_store(
    ticker: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<bool> {
    // Fetch key-metrics TTM
    let km_url = format!(
        "https://financialmodelingprep.com/api/v3/key-metrics-ttm/{ticker}?apikey={api_key}"
    );
    let km_resp = client.get(&km_url).send().await
        .context("FMP key-metrics request failed")?;
    if !km_resp.status().is_success() {
        anyhow::bail!("FMP key-metrics HTTP {}", km_resp.status());
    }
    let km_list: Vec<FmpKeyMetrics> = km_resp.json().await.unwrap_or_default();
    let km = km_list.into_iter().next().unwrap_or_default();

    crate::log_fetch(pool, "fmp", Some(ticker), "key-metrics-ttm", "ok", None).await;

    // Fetch ratios TTM
    let rat_url = format!(
        "https://financialmodelingprep.com/api/v3/ratios-ttm/{ticker}?apikey={api_key}"
    );
    let rat_resp = client.get(&rat_url).send().await
        .context("FMP ratios request failed")?;
    if !rat_resp.status().is_success() {
        anyhow::bail!("FMP ratios HTTP {}", rat_resp.status());
    }
    let rat_list: Vec<FmpRatios> = rat_resp.json().await.unwrap_or_default();
    let rat = rat_list.into_iter().next().unwrap_or_default();

    crate::log_fetch(pool, "fmp", Some(ticker), "ratios-ttm", "ok", None).await;

    // Derive absolute values from per-share metrics
    let shares = km.shares_outstanding.map(|s| s as i64);
    let fcf = km.free_cash_flow_per_share_ttm.and_then(|ps| {
        km.shares_outstanding.map(|s| (ps * s) as i64)
    });
    let operating_cf = km.operating_cash_flow_per_share_ttm.and_then(|ps| {
        km.shares_outstanding.map(|s| (ps * s) as i64)
    });
    let revenue = km.revenue_per_share_ttm.and_then(|ps| {
        km.shares_outstanding.map(|s| (ps * s) as i64)
    });
    let net_income = km.net_income_per_share_ttm.and_then(|ps| {
        km.shares_outstanding.map(|s| (ps * s) as i64)
    });
    let eps = km.net_income_per_share_ttm;

    // Check if we got any useful data
    if km.pe_ratio_ttm.is_none()
        && km.pb_ratio_ttm.is_none()
        && km.market_cap_ttm.is_none()
    {
        return Ok(false);
    }

    sqlx::query(
        "INSERT INTO fundamental_metrics (
            ticker, fetch_date,
            market_cap, pe_ratio, pb_ratio, ps_ratio, ev_ebitda, peg_ratio, price_to_fcf,
            roe, roa, net_margin, operating_margin,
            debt_equity, current_ratio,
            fcf, operating_cf, revenue, net_income, eps,
            dividend_yield, shares_outstanding
        ) VALUES (
            $1, CURRENT_DATE,
            $2, $3, $4, $5, $6, $7, $8,
            $9, $10, $11, $12,
            $13, $14,
            $15, $16, $17, $18, $19,
            $20, $21
        ) ON CONFLICT (ticker, fetch_date) DO UPDATE SET
            market_cap = EXCLUDED.market_cap,
            pe_ratio = EXCLUDED.pe_ratio,
            pb_ratio = EXCLUDED.pb_ratio,
            ps_ratio = EXCLUDED.ps_ratio,
            ev_ebitda = EXCLUDED.ev_ebitda,
            peg_ratio = EXCLUDED.peg_ratio,
            price_to_fcf = EXCLUDED.price_to_fcf,
            roe = EXCLUDED.roe,
            roa = EXCLUDED.roa,
            net_margin = EXCLUDED.net_margin,
            operating_margin = EXCLUDED.operating_margin,
            debt_equity = EXCLUDED.debt_equity,
            current_ratio = EXCLUDED.current_ratio,
            fcf = EXCLUDED.fcf,
            operating_cf = EXCLUDED.operating_cf,
            revenue = EXCLUDED.revenue,
            net_income = EXCLUDED.net_income,
            eps = EXCLUDED.eps,
            dividend_yield = EXCLUDED.dividend_yield,
            shares_outstanding = EXCLUDED.shares_outstanding",
    )
    .bind(ticker)
    .bind(km.market_cap_ttm.map(|v| v as i64))
    .bind(km.pe_ratio_ttm)
    .bind(km.pb_ratio_ttm)
    .bind(km.price_to_sales_ratio_ttm)
    .bind(km.enterprise_value_over_ebitda_ttm)
    .bind(km.peg_ratio_ttm)
    .bind(km.price_to_free_cash_flows_ttm)
    .bind(km.roe_ttm)
    .bind(km.roa_ttm)
    .bind(rat.net_profit_margin_ttm)
    .bind(rat.operating_profit_margin_ttm)
    .bind(km.debt_to_equity_ttm)
    .bind(km.current_ratio_ttm)
    .bind(fcf)
    .bind(operating_cf)
    .bind(revenue)
    .bind(net_income)
    .bind(eps)
    .bind(km.dividend_yield_ttm)
    .bind(shares)
    .execute(pool)
    .await
    .context("Failed to insert fundamental_metrics")?;

    Ok(true)
}
