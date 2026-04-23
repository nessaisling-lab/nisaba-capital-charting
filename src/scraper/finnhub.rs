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

#[derive(Debug, Deserialize)]
struct FinnhubProfile {
    #[serde(rename = "finnhubIndustry", default)]
    finnhub_industry: Option<String>,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn fetch_all_finnhub(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    finnhub_key: Arc<String>,
    limiter: Arc<governor::DefaultDirectRateLimiter>,
    extra_tickers: &[String],
) {
    // Combine watchlist + astro-priority tickers (deduplicated)
    let mut all_tickers: Vec<String> = crate::watchlist().iter().map(|s| s.to_string()).collect();
    for t in extra_tickers {
        if !all_tickers.iter().any(|existing| existing == t) {
            all_tickers.push(t.clone());
        }
    }

    for ticker in &all_tickers {
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

    for ticker in &all_tickers {
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

    // Sector enrichment: fetch industry classification for tickers missing sector data.
    // Uses Finnhub /stock/profile2 which returns finnhubIndustry (free, 60 req/min).
    // Capped at 50 per run to avoid hogging the rate limiter.
    enrich_sectors(&pool, &client, &finnhub_key, &limiter).await;
}

async fn enrich_sectors(
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    finnhub_key: &str,
    limiter: &governor::DefaultDirectRateLimiter,
) {
    // Find tickers that have astro scores (scored universe) but no sector data
    let missing: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT a.ticker
         FROM astro_scores a
         JOIN company_metadata cm ON cm.ticker = a.ticker
         WHERE cm.sector IS NULL
         ORDER BY a.ticker
         LIMIT 50",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if missing.is_empty() {
        println!("[Finnhub] All scored tickers have sector data.");
        return;
    }

    println!("[Finnhub] Enriching sector data for {} tickers...", missing.len());
    let mut enriched = 0u32;

    for ticker in &missing {
        limiter.until_ready().await;

        let url = format!(
            "https://finnhub.io/api/v1/stock/profile2?symbol={ticker}&token={finnhub_key}"
        );

        let resp = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[Finnhub] sector {ticker}: request failed: {e}");
                continue;
            }
        };

        if !resp.status().is_success() {
            continue;
        }

        let profile: FinnhubProfile = match resp.json().await {
            Ok(p) => p,
            Err(_) => continue,
        };

        let Some(industry) = profile.finnhub_industry.filter(|s| !s.is_empty()) else {
            continue;
        };

        // Map Finnhub industry to a broad sector classification
        let sector = finnhub_industry_to_sector(&industry);

        let _ = sqlx::query(
            "UPDATE company_metadata SET sector = $1, industry = $2 WHERE ticker = $3",
        )
        .bind(&sector)
        .bind(&industry)
        .bind(ticker)
        .execute(pool)
        .await;

        enriched += 1;
    }

    println!("[Finnhub] Sector enrichment: {enriched}/{} tickers updated.", missing.len());
}

/// Map Finnhub's ~70 industry categories to ~11 GICS-like sectors.
fn finnhub_industry_to_sector(industry: &str) -> String {
    match industry {
        // Technology
        "Technology" | "Semiconductors" | "Software" | "Internet Content & Information"
        | "Information Technology Services" | "Electronic Components"
        | "Computer Hardware" | "Scientific & Technical Instruments"
        | "Communication Equipment" | "Consumer Electronics" => "Technology",

        // Healthcare
        "Biotechnology" | "Drug Manufacturers" | "Medical Devices"
        | "Health Information Services" | "Medical Instruments & Supplies"
        | "Diagnostics & Research" | "Healthcare Plans" | "Medical Care Facilities"
        | "Pharmaceutical Retailers" | "Medical Distribution" => "Healthcare",

        // Financial Services
        "Banks" | "Insurance" | "Capital Markets" | "Financial Data & Stock Exchanges"
        | "Credit Services" | "Asset Management" | "Insurance Brokers"
        | "Financial Conglomerates" | "Mortgage Finance" | "Shell Companies" => "Financial Services",

        // Consumer Cyclical
        "Auto Manufacturers" | "Restaurants" | "Apparel Retail" | "Home Improvement Retail"
        | "Internet Retail" | "Specialty Retail" | "Leisure" | "Lodging"
        | "Residential Construction" | "Auto Parts" | "Luxury Goods"
        | "Gambling" | "Travel Services" | "Apparel Manufacturing" => "Consumer Cyclical",

        // Consumer Defensive
        "Beverages" | "Household & Personal Products" | "Packaged Foods"
        | "Tobacco" | "Discount Stores" | "Grocery Stores"
        | "Farm Products" | "Food Distribution" | "Education & Training Services" => "Consumer Defensive",

        // Industrials
        "Aerospace & Defense" | "Industrial Distribution" | "Railroads"
        | "Trucking" | "Waste Management" | "Consulting Services"
        | "Engineering & Construction" | "Building Products & Equipment"
        | "Airlines" | "Marine Shipping" | "Rental & Leasing Services"
        | "Staffing & Employment Services" | "Integrated Freight & Logistics"
        | "Conglomerates" | "Security & Protection Services" | "Specialty Business Services" => "Industrials",

        // Energy
        "Oil & Gas" | "Oil & Gas E&P" | "Oil & Gas Midstream"
        | "Oil & Gas Integrated" | "Oil & Gas Refining & Marketing"
        | "Oil & Gas Equipment & Services" | "Uranium" => "Energy",

        // Utilities
        "Utilities" | "Utilities - Regulated Electric" | "Utilities - Regulated Gas"
        | "Utilities - Diversified" | "Utilities - Renewable"
        | "Utilities - Independent Power Producers" => "Utilities",

        // Real Estate
        "REIT" | "Real Estate" | "Real Estate Services"
        | "Real Estate - Development" | "Real Estate - Diversified" => "Real Estate",

        // Communication Services
        "Telecom Services" | "Entertainment" | "Publishing"
        | "Broadcasting" | "Advertising Agencies" | "Electronic Gaming & Multimedia" => "Communication Services",

        // Basic Materials
        "Gold" | "Silver" | "Copper" | "Steel" | "Chemicals"
        | "Specialty Chemicals" | "Building Materials" | "Lumber & Wood Production"
        | "Paper & Paper Products" | "Aluminum" | "Other Precious Metals & Mining"
        | "Other Industrial Metals & Mining" | "Agricultural Inputs" | "Coking Coal" => "Basic Materials",

        // Fallback: use the raw industry as the sector
        _ => return industry.to_string(),
    }
    .to_string()
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

    let watchlist_set: std::collections::HashSet<&str> = crate::watchlist().iter().copied().collect();
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
