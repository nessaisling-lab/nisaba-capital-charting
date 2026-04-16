use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Serde types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub(crate) struct FinnhubNewsItem {
    pub headline: String,
    pub summary: String,
    pub source: String,
    pub url: String,
    pub datetime: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FinnhubEarningsResponse {
    #[serde(rename = "earningsCalendar")]
    pub earnings_calendar: Vec<FinnhubEarningsItem>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FinnhubEarningsItem {
    pub symbol: String,
    pub date: Option<String>,
    #[serde(rename = "epsEstimate")]
    pub eps_estimate: Option<f64>,
    #[serde(rename = "epsActual")]
    pub eps_actual: Option<f64>,
    #[serde(rename = "revenueEstimate")]
    pub revenue_estimate: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FinnhubRecommendation {
    pub period: String,
    #[serde(rename = "strongBuy", default)]
    pub strong_buy: i32,
    #[serde(default)]
    pub buy: i32,
    #[serde(default)]
    pub hold: i32,
    #[serde(default)]
    pub sell: i32,
    #[serde(rename = "strongSell", default)]
    pub strong_sell: i32,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn fetch_all_finnhub(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    finnhub_key: Arc<String>,
    limiter: Arc<governor::DefaultDirectRateLimiter>,
) {
    for ticker in crate::WATCHLIST {
        limiter.until_ready().await;
        match fetch_finnhub_news(ticker, &pool, &client, &finnhub_key).await {
            Ok(n) => {
                println!("[{ticker}] Finnhub news: {n} new articles");
                crate::log_fetch(&pool, "finnhub", Some(ticker), "news", "ok", None).await;
            }
            Err(e) => {
                eprintln!("[{ticker}] Finnhub news error (skipping): {e:#}");
                crate::log_fetch(&pool, "finnhub", Some(ticker), "news", "error", Some(&e.to_string())).await;
            }
        }
    }

    limiter.until_ready().await;
    match fetch_finnhub_earnings(&pool, &client, &finnhub_key).await {
        Ok(n) => {
            println!("[earnings] Finnhub earnings calendar: {n} new dates");
            crate::log_fetch(&pool, "finnhub", None, "earnings", "ok", None).await;
        }
        Err(e) => {
            eprintln!("[earnings] Finnhub earnings error (skipping): {e:#}");
            crate::log_fetch(&pool, "finnhub", None, "earnings", "error", Some(&e.to_string())).await;
        }
    }

    for ticker in crate::WATCHLIST {
        limiter.until_ready().await;
        match fetch_finnhub_recommendations(ticker, &pool, &client, &finnhub_key).await {
            Ok(n) => {
                println!("[{ticker}] Finnhub recommendations: {n} new rows");
                crate::log_fetch(&pool, "finnhub", Some(ticker), "recommendations", "ok", None).await;
            }
            Err(e) => {
                eprintln!("[{ticker}] Finnhub recommendations error (skipping): {e:#}");
                crate::log_fetch(&pool, "finnhub", Some(ticker), "recommendations", "error", Some(&e.to_string())).await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Individual fetch functions
// ---------------------------------------------------------------------------

async fn fetch_finnhub_news(
    ticker: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    finnhub_key: &str,
) -> Result<u64> {
    let to   = Utc::now();
    let from = to - chrono::Duration::days(7);
    let url  = format!(
        "https://finnhub.io/api/v1/company-news\
         ?symbol={ticker}&from={}&to={}&token={finnhub_key}",
        from.format("%Y-%m-%d"),
        to.format("%Y-%m-%d"),
    );

    let resp = client.get(&url).send().await
        .context("Finnhub news request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("Finnhub news returned HTTP {}", resp.status());
    }

    let articles: Vec<FinnhubNewsItem> = resp.json().await
        .context("Failed to parse Finnhub news response")?;

    let mut inserted = 0u64;
    for item in articles {
        if item.url.is_empty() { continue; }

        let published_at = chrono::DateTime::from_timestamp(item.datetime, 0)
            .unwrap_or_else(Utc::now);

        let result = sqlx::query(
            "INSERT INTO news_articles (ticker, headline, summary, source, url, published_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (ticker, url) DO NOTHING",
        )
        .bind(ticker)
        .bind(&item.headline)
        .bind(&item.summary)
        .bind(&item.source)
        .bind(&item.url)
        .bind(published_at)
        .execute(pool)
        .await
        .context("Failed to insert news article")?;

        inserted += result.rows_affected();
    }

    Ok(inserted)
}

async fn fetch_finnhub_earnings(
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    finnhub_key: &str,
) -> Result<u64> {
    let from = Utc::now();
    let to   = from + chrono::Duration::days(90);
    let url  = format!(
        "https://finnhub.io/api/v1/calendar/earnings\
         ?from={}&to={}&token={finnhub_key}",
        from.format("%Y-%m-%d"),
        to.format("%Y-%m-%d"),
    );

    let resp = client.get(&url).send().await
        .context("Finnhub earnings request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("Finnhub earnings returned HTTP {}", resp.status());
    }

    let data: FinnhubEarningsResponse = resp.json().await
        .context("Failed to parse Finnhub earnings response")?;

    let watchlist_set: std::collections::HashSet<&str> = crate::WATCHLIST.iter().copied().collect();
    let mut inserted = 0u64;

    for item in data.earnings_calendar {
        if !watchlist_set.contains(item.symbol.as_str()) { continue; }
        let date_str     = match item.date { Some(d) => d, None => continue };
        let earnings_date = match chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };

        let eps_estimate    = item.eps_estimate
            .and_then(|v| format!("{v:.4}").parse::<rust_decimal::Decimal>().ok());
        let eps_actual      = item.eps_actual
            .and_then(|v| format!("{v:.4}").parse::<rust_decimal::Decimal>().ok());
        let revenue_estimate = item.revenue_estimate.map(|v| v as i64);

        let result = sqlx::query(
            "INSERT INTO earnings_dates \
             (ticker, earnings_date, eps_estimate, eps_actual, revenue_estimate) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (ticker, earnings_date) DO NOTHING",
        )
        .bind(&item.symbol)
        .bind(earnings_date)
        .bind(eps_estimate)
        .bind(eps_actual)
        .bind(revenue_estimate)
        .execute(pool)
        .await
        .context("Failed to insert earnings date")?;

        inserted += result.rows_affected();
    }

    Ok(inserted)
}

async fn fetch_finnhub_recommendations(
    ticker: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    finnhub_key: &str,
) -> Result<u64> {
    let url = format!(
        "https://finnhub.io/api/v1/stock/recommendation\
         ?symbol={ticker}&token={finnhub_key}"
    );

    let resp = client.get(&url).send().await
        .context("Finnhub recommendations request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("Finnhub recommendations returned HTTP {}", resp.status());
    }

    let recs: Vec<FinnhubRecommendation> = resp.json().await
        .context("Failed to parse Finnhub recommendations response")?;

    let rec = match recs.into_iter().next() {
        Some(r) => r,
        None => return Ok(0),
    };

    let result = sqlx::query(
        "INSERT INTO analyst_ratings \
         (ticker, period, strong_buy, buy, hold, sell, strong_sell) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (ticker, period) DO NOTHING",
    )
    .bind(ticker)
    .bind(&rec.period)
    .bind(rec.strong_buy)
    .bind(rec.buy)
    .bind(rec.hold)
    .bind(rec.sell)
    .bind(rec.strong_sell)
    .execute(pool)
    .await
    .context("Failed to insert analyst rating")?;

    Ok(result.rows_affected())
}
