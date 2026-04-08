use anyhow::{Context, Result};
use chrono::NaiveDate;
use governor::{Quota, RateLimiter};
use pursuit_week4_automation::models::AlphaVantageResponse;
use rust_decimal::Decimal;
use sqlx::postgres::PgPoolOptions;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

// ---------------------------------------------------------------------------
// Data flow:
//
//  tokio-cron-scheduler (daily at 6am UTC)
//       │
//       ▼
//  governor rate limiter (5 req/min for Alpha Vantage)
//       │
//       ▼
//  reqwest::Client → GET alphavantage.co/TIME_SERIES_DAILY
//       │
//       ▼
//  serde_json parse → AlphaVantageResponse
//       │
//       ▼
//  sqlx INSERT INTO price_data
//  ON CONFLICT (ticker, date) DO NOTHING   ← handles re-runs safely
//       │
//       ▼
//  PostgreSQL
// ---------------------------------------------------------------------------

const WATCHLIST: &[&str] = &["AAPL"];

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    let api_key = std::env::var("ALPHA_VANTAGE_API_KEY")
        .context("ALPHA_VANTAGE_API_KEY env var required")?;
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL env var required")?;

    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;

    println!("Connected. Running migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations")?;
    println!("Migrations OK.");

    // HTTP client — EDGAR requires a descriptive User-Agent on every request
    let user_agent = std::env::var("EDGAR_USER_AGENT")
        .unwrap_or_else(|_| "FinancialDashboard/1.0 student@pursuit.org".to_string());

    let http_client = reqwest::Client::builder()
        .user_agent(&user_agent)
        .build()
        .context("Failed to build HTTP client")?;

    // Rate limiter: Alpha Vantage free tier = 5 req/min
    let quota = Quota::per_minute(NonZeroU32::new(5).unwrap());
    let limiter = Arc::new(RateLimiter::direct(quota));

    let pool = Arc::new(pool);
    let http_client = Arc::new(http_client);
    let api_key = Arc::new(api_key);

    // Run once immediately on startup so you can test without waiting 24 hours
    println!("Running immediate fetch...");
    fetch_all_tickers(
        Arc::clone(&pool),
        Arc::clone(&http_client),
        Arc::clone(&api_key),
        Arc::clone(&limiter),
    )
    .await;

    // Schedule daily at 6:00 UTC (school requirement: automated system)
    println!("Starting scheduler. Daily fetch at 06:00 UTC.");
    let sched = JobScheduler::new().await?;

    let pool_job = Arc::clone(&pool);
    let client_job = Arc::clone(&http_client);
    let key_job = Arc::clone(&api_key);
    let limiter_job = Arc::clone(&limiter);

    sched
        .add(Job::new_async("0 0 6 * * *", move |_, _| {
            let pool = Arc::clone(&pool_job);
            let client = Arc::clone(&client_job);
            let key = Arc::clone(&key_job);
            let lim = Arc::clone(&limiter_job);
            Box::pin(async move {
                fetch_all_tickers(pool, client, key, lim).await;
            })
        })?)
        .await?;

    sched.start().await?;

    println!("Scheduler running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    println!("Shutting down.");
    Ok(())
}

async fn fetch_all_tickers(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
    limiter: Arc<governor::DefaultDirectRateLimiter>,
) {
    for ticker in WATCHLIST {
        // Block until we have a rate-limit slot
        limiter.until_ready().await;

        match fetch_and_store(ticker, &pool, &client, &api_key).await {
            Ok(inserted) => println!("[{ticker}] Inserted {inserted} new rows"),
            Err(e) => eprintln!("[{ticker}] Error (skipping): {e:#}"),
        }
    }
}

async fn fetch_and_store(
    ticker: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<u64> {
    let url = format!(
        "https://www.alphavantage.co/query\
         ?function=TIME_SERIES_DAILY\
         &symbol={ticker}\
         &apikey={api_key}\
         &outputsize=compact"
    );

    let response = client
        .get(&url)
        .send()
        .await
        .context("HTTP request to Alpha Vantage failed")?;

    if !response.status().is_success() {
        anyhow::bail!("Alpha Vantage returned HTTP {}", response.status());
    }

    let body: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse Alpha Vantage response")?;

    // Alpha Vantage signals rate limits via JSON fields, not HTTP status codes
    if let Some(note) = body.get("Note") {
        anyhow::bail!("Rate limit message: {note}");
    }
    if let Some(msg) = body.get("Information") {
        anyhow::bail!("Alpha Vantage info: {msg}");
    }

    let parsed: AlphaVantageResponse =
        serde_json::from_value(body).context("Failed to parse time series")?;

    let mut inserted = 0u64;
    for (date_str, entry) in &parsed.time_series {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .context(format!("Invalid date: {date_str}"))?;

        let open: Decimal = entry.open.parse().context("parse open")?;
        let high: Decimal = entry.high.parse().context("parse high")?;
        let low: Decimal = entry.low.parse().context("parse low")?;
        let close: Decimal = entry.close.parse().context("parse close")?;
        let volume: i64 = entry.volume.parse().context("parse volume")?;

        // ON CONFLICT DO NOTHING: safe to re-run, handles duplicate days cleanly
        let result = sqlx::query(
            "INSERT INTO price_data (ticker, date, open, high, low, close, volume) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (ticker, date) DO NOTHING",
        )
        .bind(ticker)
        .bind(date)
        .bind(open)
        .bind(high)
        .bind(low)
        .bind(close)
        .bind(volume)
        .execute(pool)
        .await
        .context("DB insert failed")?;

        inserted += result.rows_affected();
    }

    Ok(inserted)
}
