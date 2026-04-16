mod astrology;
mod edgar;
mod finnhub;
mod holdings;
mod macro_data;
mod options;
mod prices;
mod sentiment;
mod short_interest;

use anyhow::{Context, Result};
use governor::{Quota, RateLimiter};
use sqlx::postgres::PgPoolOptions;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

// ---------------------------------------------------------------------------
// Watchlist + reference data — shared across all scraper modules via crate::
// ---------------------------------------------------------------------------

pub(crate) const WATCHLIST: &[&str] = &[
    "AAPL", "MSFT", "GOOGL", "AMZN", "NVDA",
    "META", "TSLA", "JPM", "V", "UNH",
];

pub(crate) const INSTITUTION_MAP: &[(&str, &str)] = &[
    ("0000102909", "Vanguard Group Inc."),
    ("0001364742", "BlackRock Inc."),
    ("0000093751", "State Street Corporation"),
    ("0000315066", "Fidelity Management & Research"),
    ("0001113169", "T. Rowe Price Associates"),
];

pub(crate) const CUSIP_MAP: &[(&str, &str)] = &[
    ("037833100", "AAPL"),
    ("594918104", "MSFT"),
    ("02079K305", "GOOGL"),
    ("023135106", "AMZN"),
    ("67066G104", "NVDA"),
    ("30303M102", "META"),
    ("88160R101", "TSLA"),
    ("46625H100", "JPM"),
    ("92826C839", "V"),
    ("91324P102", "UNH"),
];

pub(crate) const CIK_MAP: &[(&str, &str)] = &[
    ("AAPL",  "0000320193"),
    ("MSFT",  "0000789019"),
    ("GOOGL", "0001652044"),
    ("AMZN",  "0001018724"),
    ("NVDA",  "0001045810"),
    ("META",  "0001326801"),
    ("TSLA",  "0001318605"),
    ("JPM",   "0000019617"),
    ("V",     "0001403161"),
    ("UNH",   "0000731766"),
];

// ---------------------------------------------------------------------------
// Shared fetch audit log — fire-and-forget, never crashes the scraper
// ---------------------------------------------------------------------------

pub(crate) async fn log_fetch(
    pool: &sqlx::PgPool,
    source: &str,
    ticker: Option<&str>,
    fetch_type: &str,
    status: &str,
    error_message: Option<&str>,
) {
    let _ = sqlx::query(
        "INSERT INTO fetch_log (source, ticker, fetch_type, status, error_message) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(source)
    .bind(ticker)
    .bind(fetch_type)
    .bind(status)
    .bind(error_message)
    .execute(pool)
    .await;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
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

    let user_agent = std::env::var("EDGAR_USER_AGENT")
        .unwrap_or_else(|_| "FinancialDashboard/1.0 student@pursuit.org".to_string());

    let finnhub_key: Option<Arc<String>> = match std::env::var("FINNHUB_API_KEY") {
        Ok(k) if !k.is_empty() => {
            println!("Finnhub API key found.");
            Some(Arc::new(k))
        }
        _ => {
            eprintln!("Warning: FINNHUB_API_KEY not set — Finnhub fetches will be skipped.");
            None
        }
    };

    let fred_key: Option<Arc<String>> = match std::env::var("FRED_API_KEY") {
        Ok(k) if !k.is_empty() => { println!("FRED API key found."); Some(Arc::new(k)) }
        _ => { eprintln!("Warning: FRED_API_KEY not set — macro data will be skipped."); None }
    };

    let finra_key: Option<Arc<String>> = match std::env::var("FINRA_API_KEY") {
        Ok(k) if !k.is_empty() => { println!("FINRA API key found."); Some(Arc::new(k)) }
        _ => { eprintln!("Warning: FINRA_API_KEY not set — short interest will be skipped."); None }
    };

    let polygon_key: Option<Arc<String>> = match std::env::var("POLYGON_API_KEY") {
        Ok(k) if !k.is_empty() => { println!("Polygon.io key found."); Some(Arc::new(k)) }
        _ => { eprintln!("Warning: POLYGON_API_KEY not set — options flow will be skipped."); None }
    };

    let http_client = reqwest::Client::builder()
        .user_agent(&user_agent)
        .build()
        .context("Failed to build HTTP client")?;

    // Rate limiters
    let av_quota      = Quota::per_minute(NonZeroU32::new(5).unwrap());
    let av_limiter    = Arc::new(RateLimiter::direct(av_quota));
    let fh_quota      = Quota::per_minute(NonZeroU32::new(60).unwrap());
    let fh_limiter    = Arc::new(RateLimiter::direct(fh_quota));

    let pool        = Arc::new(pool);
    let http_client = Arc::new(http_client);
    let api_key     = Arc::new(api_key);

    // ---- Startup run -------------------------------------------------------
    run_all_fetches(
        Arc::clone(&pool),
        Arc::clone(&http_client),
        Arc::clone(&api_key),
        Arc::clone(&av_limiter),
        finnhub_key.clone(),
        Arc::clone(&fh_limiter),
        fred_key.clone(),
        finra_key.clone(),
        polygon_key.clone(),
    ).await;

    // ---- Daily scheduler at 06:00 UTC -------------------------------------
    println!("Starting scheduler. Daily fetch at 06:00 UTC.");
    let sched = JobScheduler::new().await?;

    let pool2        = Arc::clone(&pool);
    let client2      = Arc::clone(&http_client);
    let key2         = Arc::clone(&api_key);
    let av_lim2      = Arc::clone(&av_limiter);
    let fh_key2      = finnhub_key.clone();
    let fh_lim2      = Arc::clone(&fh_limiter);
    let fred_key2    = fred_key.clone();
    let finra_key2   = finra_key.clone();
    let polygon_key2 = polygon_key.clone();

    sched.add(Job::new_async("0 0 6 * * *", move |_, _| {
        let pool        = Arc::clone(&pool2);
        let client      = Arc::clone(&client2);
        let key         = Arc::clone(&key2);
        let av_lim      = Arc::clone(&av_lim2);
        let fh_key      = fh_key2.clone();
        let fh_lim      = Arc::clone(&fh_lim2);
        let fred_key    = fred_key2.clone();
        let finra_key   = finra_key2.clone();
        let polygon_key = polygon_key2.clone();
        Box::pin(async move {
            run_all_fetches(pool, client, key, av_lim, fh_key, fh_lim,
                            fred_key, finra_key, polygon_key).await;
        })
    })?).await?;

    sched.start().await?;
    println!("Scheduler running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    println!("Shutting down.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Full fetch pipeline — called at startup and by the daily cron job
// ---------------------------------------------------------------------------

async fn run_all_fetches(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
    av_limiter: Arc<governor::DefaultDirectRateLimiter>,
    finnhub_key: Option<Arc<String>>,
    fh_limiter: Arc<governor::DefaultDirectRateLimiter>,
    fred_key: Option<Arc<String>>,
    finra_key: Option<Arc<String>>,
    polygon_key: Option<Arc<String>>,
) {
    println!("Running price fetch...");
    prices::fetch_all_tickers(
        Arc::clone(&pool), Arc::clone(&client), Arc::clone(&api_key), Arc::clone(&av_limiter),
    ).await;

    println!("Fetching EDGAR insider trades and 8-K filings...");
    edgar::fetch_all_edgar(Arc::clone(&pool), Arc::clone(&client)).await;

    println!("Fetching 13F institutional holdings...");
    holdings::fetch_all_13f(Arc::clone(&pool), Arc::clone(&client)).await;

    if let Some(fh_key) = finnhub_key {
        println!("Fetching Finnhub data (news, earnings, analyst ratings)...");
        finnhub::fetch_all_finnhub(
            Arc::clone(&pool), Arc::clone(&client), fh_key, fh_limiter,
        ).await;
    }

    println!("Fetching Alpha Vantage sentiment (budget-aware)...");
    sentiment::fetch_av_sentiment_all(Arc::clone(&pool), Arc::clone(&client), Arc::clone(&api_key)).await;

    if let Some(fred_k) = fred_key {
        println!("Fetching FRED macroeconomic data...");
        macro_data::fetch_all_macro(Arc::clone(&pool), Arc::clone(&client), fred_k).await;
    }

    if let Some(finra_k) = finra_key {
        println!("Fetching short interest (FINRA API)...");
        short_interest::fetch_all_short_interest(Arc::clone(&pool), Arc::clone(&client), finra_k).await;
    }

    if let Some(polygon_k) = polygon_key {
        println!("Fetching options flow (Polygon.io)...");
        options::fetch_all_options_flow(Arc::clone(&pool), Arc::clone(&client), polygon_k).await;
    }

    println!("Seeding natal charts (any missing companies)...");
    astrology::seed_natal_charts(Arc::clone(&pool)).await;
    println!("Computing daily planetary transits...");
    astrology::compute_daily_transits(Arc::clone(&pool)).await;
    println!("Computing astrological scores...");
    astrology::compute_astro_scores(pool).await;
}
