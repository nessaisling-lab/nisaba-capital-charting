mod astrology;
mod company_enrich;
mod dbnomics;
mod edgar;
mod gdelt;
mod edgar_enrich;
mod enrich_common;
mod fmp_enrich;
mod finnhub;
mod fundamentals;
mod holdings;
mod lagrange;
mod macro_data;
mod options;
mod paper_engine;
mod polymarket;
mod analyst_targets;
mod prices;
mod retry;
mod rss_news;
mod sources;
mod rss_tone;
mod sentiment;
mod short_interest;
mod ticker_seed;
mod tiingo;
mod wikidata_enrich;
mod wikipedia;
mod world_bank;
mod coingecko;
mod treasury_direct;
mod imf;
mod ecb;
mod bls;
mod eia;
mod cftc_cot;
mod ofr;
mod sec_recent;

use anyhow::{Context, Result};
use governor::{Quota, RateLimiter};
use sqlx::postgres::PgPoolOptions;
use std::num::NonZeroU32;
use std::sync::{Arc, OnceLock};
use tokio_cron_scheduler::{Job, JobScheduler};

// ---------------------------------------------------------------------------
// Watchlist + reference data — DB-backed with compile-time defaults.
//
// At startup, `init_config()` loads from `scraper_watchlist` and
// `scraper_institutions` tables. If the tables are empty or the query fails,
// the DEFAULT_* consts are used as fallback. Accessor functions (`watchlist()`,
// `cik_map()`, etc.) return `&'static [&'static str]` for zero-cost
// compatibility with all existing call sites.
//
// The `Box::leak` pattern is intentional: these strings live for the entire
// process lifetime, so leaking them is equivalent to a static allocation.
// ---------------------------------------------------------------------------

const DEFAULT_WATCHLIST: &[&str] = &[
    "AAPL", "MSFT", "GOOGL", "AMZN", "NVDA",
    "META", "TSLA", "JPM", "V", "UNH",
];

const DEFAULT_INSTITUTION_MAP: &[(&str, &str)] = &[
    ("0000102909", "Vanguard Group Inc."),
    ("0001364742", "BlackRock Inc."),
    ("0000093751", "State Street Corporation"),
    ("0000315066", "Fidelity Management & Research"),
];

const DEFAULT_CUSIP_MAP: &[(&str, &str)] = &[
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

const DEFAULT_CIK_MAP: &[(&str, &str)] = &[
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

static WATCHLIST_STORE: OnceLock<&'static [&'static str]> = OnceLock::new();
static CIK_MAP_STORE: OnceLock<&'static [(&'static str, &'static str)]> = OnceLock::new();
static CUSIP_MAP_STORE: OnceLock<&'static [(&'static str, &'static str)]> = OnceLock::new();
static INSTITUTION_MAP_STORE: OnceLock<&'static [(&'static str, &'static str)]> = OnceLock::new();

pub(crate) fn watchlist() -> &'static [&'static str] {
    WATCHLIST_STORE.get().copied().unwrap_or(DEFAULT_WATCHLIST)
}
pub(crate) fn cik_map() -> &'static [(&'static str, &'static str)] {
    CIK_MAP_STORE.get().copied().unwrap_or(DEFAULT_CIK_MAP)
}
pub(crate) fn cusip_map() -> &'static [(&'static str, &'static str)] {
    CUSIP_MAP_STORE.get().copied().unwrap_or(DEFAULT_CUSIP_MAP)
}
pub(crate) fn institution_map() -> &'static [(&'static str, &'static str)] {
    INSTITUTION_MAP_STORE.get().copied().unwrap_or(DEFAULT_INSTITUTION_MAP)
}

/// Leak a `String` to get a `&'static str`. Safe for process-lifetime config.
fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// Load scraper config from the `scraper_watchlist` and `scraper_institutions`
/// tables. Falls back to DEFAULT_* constants on error or empty results.
async fn init_config(pool: &sqlx::PgPool) {
    // Load watchlist + CIK + CUSIP from one table
    match sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT ticker, cik, cusip FROM scraper_watchlist WHERE active = true ORDER BY ticker",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) if !rows.is_empty() => {
            let tickers: Vec<&'static str> = rows.iter().map(|(t, _, _)| leak_str(t.clone())).collect();
            let cik_pairs: Vec<(&'static str, &'static str)> = rows.iter()
                .filter_map(|(t, c, _)| Some((leak_str(t.clone()), leak_str(c.as_ref()?.clone()))))
                .collect();
            let cusip_pairs: Vec<(&'static str, &'static str)> = rows.iter()
                .filter_map(|(t, _, c)| Some((leak_str(c.as_ref()?.clone()), leak_str(t.clone()))))
                .collect();
            println!("[Config] Loaded {} tickers from scraper_watchlist", tickers.len());
            let _ = WATCHLIST_STORE.set(Box::leak(tickers.into_boxed_slice()));
            let _ = CIK_MAP_STORE.set(Box::leak(cik_pairs.into_boxed_slice()));
            let _ = CUSIP_MAP_STORE.set(Box::leak(cusip_pairs.into_boxed_slice()));
        }
        Ok(_) => println!("[Config] scraper_watchlist empty, using defaults"),
        Err(e) => eprintln!("[Config] Failed to load watchlist ({e}), using defaults"),
    }

    // Load institution CIKs
    match sqlx::query_as::<_, (String, String)>(
        "SELECT cik, name FROM scraper_institutions WHERE active = true ORDER BY name",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) if !rows.is_empty() => {
            let pairs: Vec<(&'static str, &'static str)> =
                rows.into_iter().map(|(c, n)| (leak_str(c), leak_str(n))).collect();
            let _ = INSTITUTION_MAP_STORE.set(Box::leak(pairs.into_boxed_slice()));
        }
        _ => {} // Use defaults silently
    }
}

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

    // Load DB-backed config (watchlist, CIK map, etc.)
    init_config(&pool).await;

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

    let fmp_key: Option<Arc<String>> = match std::env::var("FMP_API_KEY") {
        Ok(k) if !k.is_empty() => { println!("FMP key found."); Some(Arc::new(k)) }
        _ => { eprintln!("Warning: FMP_API_KEY not set — FMP ticker seed and IPO enrichment will be skipped."); None }
    };

    let tiingo_key: Option<Arc<String>> = match std::env::var("TIINGO_API_KEY") {
        Ok(k) if !k.is_empty() => { println!("Tiingo key found."); Some(Arc::new(k)) }
        _ => { eprintln!("Warning: TIINGO_API_KEY not set — bulk price history will be skipped."); None }
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
    let user_agent  = Arc::new(user_agent);

    // ---- Check for single-ticker mode (--ticker AAPL) -----------------------
    let args: Vec<String> = std::env::args().collect();
    let single_ticker = args.iter()
        .position(|a| a == "--ticker")
        .and_then(|i| args.get(i + 1))
        .cloned();

    if let Some(ticker) = single_ticker {
        println!("=== Single-ticker mode: {ticker} ===");
        fetch_single_ticker(
            Arc::clone(&pool),
            Arc::clone(&http_client),
            Arc::clone(&api_key),
            Arc::clone(&av_limiter),
            finnhub_key.clone(),
            Arc::clone(&fh_limiter),
            fmp_key.clone(),
            &ticker,
        ).await?;
        println!("=== Done: {ticker} ===");
        return Ok(());
    }

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
        fmp_key.clone(),
        tiingo_key.clone(),
        Arc::clone(&user_agent),
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
    let fmp_key2     = fmp_key.clone();
    let tiingo_key2  = tiingo_key.clone();
    let user_agent2  = Arc::clone(&user_agent);

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
        let fmp_key     = fmp_key2.clone();
        let tiingo_key  = tiingo_key2.clone();
        let user_agent  = Arc::clone(&user_agent2);
        Box::pin(async move {
            run_all_fetches(pool, client, key, av_lim, fh_key, fh_lim,
                            fred_key, finra_key, polygon_key, fmp_key, tiingo_key, user_agent).await;
        })
    })?).await?;

    sched.start().await?;
    println!("Scheduler running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    println!("Shutting down.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Single-ticker fetch — triggered by dashboard "Fetch" button via CLI
// ---------------------------------------------------------------------------

async fn fetch_single_ticker(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
    av_limiter: Arc<governor::DefaultDirectRateLimiter>,
    finnhub_key: Option<Arc<String>>,
    fh_limiter: Arc<governor::DefaultDirectRateLimiter>,
    fmp_key: Option<Arc<String>>,
    ticker: &str,
) -> anyhow::Result<()> {
    // 1. Astrology: scope to this ticker only (v11.3 — was universe-wide)
    println!("[{ticker}] Phase 1: Astrology (single-ticker)...");
    if let Err(e) = astrology::seed_natal_chart_one(Arc::clone(&pool), ticker).await {
        eprintln!("[{ticker}] Natal seed error: {e:#}");
    }
    // Daily transits are date-keyed (not ticker-keyed) — runs once per day,
    // skips if already computed. Keep the universe-wide call.
    astrology::compute_daily_transits(Arc::clone(&pool)).await;
    if let Err(e) = astrology::compute_astro_score_one(Arc::clone(&pool), ticker).await {
        eprintln!("[{ticker}] Astro score error: {e:#}");
    }

    // 2. Price data (Alpha Vantage)
    println!("[{ticker}] Phase 2: Price data...");
    av_limiter.until_ready().await;
    match prices::fetch_and_store(ticker, &pool, &client, &api_key).await {
        Ok(n) => println!("[{ticker}] Prices: {n} new rows"),
        Err(e) => eprintln!("[{ticker}] Price error: {e:#}"),
    }

    // 3. Finnhub news + recommendations
    if let Some(ref fh_key) = finnhub_key {
        println!("[{ticker}] Phase 3: Finnhub...");
        fh_limiter.until_ready().await;
        match finnhub::fetch_finnhub_news(ticker, &pool, &client, fh_key).await {
            Ok(n) => println!("[{ticker}] Finnhub news: {n} articles"),
            Err(e) => eprintln!("[{ticker}] Finnhub news error: {e:#}"),
        }
        fh_limiter.until_ready().await;
        match finnhub::fetch_finnhub_recommendations(ticker, &pool, &client, fh_key).await {
            Ok(n) => println!("[{ticker}] Finnhub ratings: {n} new"),
            Err(e) => eprintln!("[{ticker}] Finnhub ratings error: {e:#}"),
        }
    }

    // 4. Sentiment (Alpha Vantage)
    println!("[{ticker}] Phase 4: Sentiment...");
    let one_ticker = vec![ticker.to_string()];
    sentiment::fetch_av_sentiment_all(
        Arc::clone(&pool), Arc::clone(&client), Arc::clone(&api_key), &one_ticker,
    ).await;

    // 4b. Analyst price targets (Wave 6.A3) — single-ticker via Finnhub
    if let Some(ref fh_key) = finnhub_key {
        println!("[{ticker}] Phase 4b: Analyst targets...");
        if let Err(e) = analyst_targets::fetch_one_and_store(
            Arc::clone(&pool), Arc::clone(&client), Arc::clone(fh_key), ticker,
        ).await {
            eprintln!("[{ticker}] Analyst targets error: {e:#}");
        }
    }

    // 5. Fundamentals — Wave 6.A2 cascade: FMP → Finnhub → AV OVERVIEW
    if let Some(ref fmp) = fmp_key {
        println!("[{ticker}] Phase 5: Fundamentals (cascade)...");
        let fh_ref = finnhub_key.as_deref().map(|s| s.as_str());
        let av_ref = Some(api_key.as_str());
        match fundamentals::fetch_and_store_with_fallback(
            ticker, &pool, &client, fmp, fh_ref, av_ref,
        ).await {
            Ok(true)  => println!("[{ticker}] Fundamentals: stored"),
            Ok(false) => println!("[{ticker}] Fundamentals: already up to date"),
            Err(e)    => eprintln!("[{ticker}] Fundamentals error: {e:#}"),
        }
    }

    // 5b. RSS tone sentiment (supplements AV, free)
    println!("[{ticker}] Phase 5b: RSS tone sentiment...");
    rss_tone::compute_rss_tone(Arc::clone(&pool)).await;

    // 6. Lagrange composite score
    println!("[{ticker}] Phase 6: Lagrange score...");
    lagrange::compute_all_scores(Arc::clone(&pool)).await;

    // v11.8.G — 7. Wikipedia summary (per-ticker fetch). Per user feedback:
    // "Every time we fetch, we need to be able to fetch Wikipedia summaries
    // for here. This has to be included with the fetch button."
    println!("[{ticker}] Phase 7: Wikipedia summary...");
    if let Err(e) = wikipedia::fetch_one(Arc::clone(&pool), Arc::clone(&client), ticker).await {
        eprintln!("[{ticker}] Wikipedia fetch error: {e:#}");
    }

    log_fetch(&pool, "dashboard", Some(ticker), "single_ticker_fetch", "ok", None).await;
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
    fmp_key: Option<Arc<String>>,
    tiingo_key: Option<Arc<String>>,
    user_agent: Arc<String>,
) {
    // =========================================================================
    // PHASE 1: ASTROLOGY (runs FIRST — zero API calls, local computation)
    // =========================================================================
    // Astrology is THE product differentiator. It runs before any financial
    // data fetching because it requires zero network calls (Swiss Ephemeris is
    // compiled into the binary). After Phase 1, we know which tickers the
    // stars favor and which are misaligned, so Phase 2 can prioritize them.
    // =========================================================================

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  PHASE 1: ASTROLOGY (local computation, no API calls)  ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("1.1 Seeding natal charts (any missing companies)...");
    astrology::seed_natal_charts(Arc::clone(&pool)).await;

    println!("1.2 Computing daily planetary transits (Swiss Ephemeris)...");
    astrology::compute_daily_transits(Arc::clone(&pool)).await;

    println!("1.3 Computing astrological scores + horoscope readings...");
    astrology::compute_astro_scores(Arc::clone(&pool)).await;

    println!("1.4 Computing astro rankings (Top 5 / Bottom 5)...");
    let ranking = astrology::compute_astro_ranking(Arc::clone(&pool)).await;

    let priority = ranking.priority_tickers();
    if !priority.is_empty() {
        println!("\n  ★ Astro-priority tickers ({} total):", priority.len());
        for (t, s, theme) in &ranking.top_favorable {
            println!("    ▲ {t:>6}  {s:5.1}  {theme}");
        }
        for (t, s, theme) in &ranking.bottom_misaligned {
            println!("    ▼ {t:>6}  {s:5.1}  {theme}");
        }
    }

    // =========================================================================
    // PHASE 1.5: PAPER ENGINE PRIORITY TICKERS
    // =========================================================================
    // The paper engine needs fresh prices for open positions (stop-loss eval)
    // and buy candidates (sizing). Merge these into the priority list so ALL
    // Phase 2 steps (prices, sentiment, Finnhub, etc.) cover them.
    // =========================================================================

    println!("\n1.5 Collecting paper engine priority tickers...");
    let paper_tickers = paper_engine::collect_priority_tickers(pool.as_ref()).await;
    let priority = if paper_tickers.is_empty() {
        println!("  No paper engine tickers to prioritize.");
        priority
    } else {
        println!("  Paper engine tickers: {} (positions + candidates)", paper_tickers.len());
        let mut combined = priority;
        for t in paper_tickers {
            if !combined.contains(&t) {
                combined.push(t);
            }
        }
        println!("  Combined priority list: {} tickers", combined.len());
        combined
    };

    // =========================================================================
    // PHASE 2: TARGETED FINANCIAL VERIFICATION (guided by astro rankings)
    // =========================================================================
    // The astro-prioritized tickers get financial data first. With limited API
    // budgets (AV: 5/min, FMP: 120/day), this ensures the most astrologically
    // interesting tickers are verified before budget is exhausted. Paper engine
    // tickers are now merged into the priority list (Phase 1.5).
    // =========================================================================

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  PHASE 2: FINANCIAL DATA (astro + paper prioritized)   ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    if !priority.is_empty() {
        println!("2.0 Fetching price data for priority tickers ({} total)...", priority.len());
        prices::fetch_priority_prices(
            Arc::clone(&pool), Arc::clone(&client), Arc::clone(&api_key),
            Arc::clone(&av_limiter), &priority,
        ).await;
    }

    println!("2.1 Fetching watchlist price data (Alpha Vantage)...");
    prices::fetch_all_tickers(
        Arc::clone(&pool), Arc::clone(&client), Arc::clone(&api_key), Arc::clone(&av_limiter),
    ).await;

    println!("2.2 Fetching Alpha Vantage sentiment (budget-aware, astro-priority first)...");
    sentiment::fetch_av_sentiment_all(Arc::clone(&pool), Arc::clone(&client), Arc::clone(&api_key), &priority).await;

    if let Some(ref fh_key) = finnhub_key {
        println!("2.3 Fetching Finnhub data (news, earnings, ratings)...");
        finnhub::fetch_all_finnhub(
            Arc::clone(&pool), Arc::clone(&client), Arc::clone(fh_key), Arc::clone(&fh_limiter),
            &priority,
        ).await;

        // Wave 6.A3 — analyst price targets (separate Finnhub endpoint).
        println!("2.3b Fetching analyst price targets (Finnhub)...");
        analyst_targets::fetch_analyst_targets(
            Arc::clone(&pool), Arc::clone(&client), Arc::clone(fh_key),
        ).await;
    }

    if let Some(ref fred_k) = fred_key {
        println!("2.4 Fetching FRED macroeconomic data...");
        macro_data::fetch_all_macro(Arc::clone(&pool), Arc::clone(&client), Arc::clone(fred_k)).await;
    }

    if let Some(ref finra_k) = finra_key {
        println!("2.5 Fetching short interest (FINRA API)...");
        short_interest::fetch_all_short_interest(Arc::clone(&pool), Arc::clone(&client), Arc::clone(finra_k), &priority).await;
    }

    if let Some(ref polygon_k) = polygon_key {
        println!("2.6 Fetching options flow (Polygon.io)...");
        options::fetch_all_options_flow(Arc::clone(&pool), Arc::clone(&client), Arc::clone(polygon_k)).await;
    }

    if let Some(ref fmp_k) = fmp_key {
        println!("2.7 Fetching fundamental metrics (FMP → Finnhub → AV fallback cascade)...");
        fundamentals::fetch_fundamentals(
            Arc::clone(&pool),
            Arc::clone(&client),
            Arc::clone(fmp_k),
            finnhub_key.as_ref().map(Arc::clone),
            Some(Arc::clone(&api_key)),
        ).await;
    }

    // =========================================================================
    // PHASE 3: BULK DATA (ticker universe + price history + enrichment)
    // =========================================================================
    // These are the heavy, high-volume fetches: ticker universe seeding,
    // bulk price history (Tiingo 490/day), EDGAR filings, and IPO enrichment
    // from multiple sources. Runs after targeted fetches so priority tickers
    // already have fresh data.
    // =========================================================================

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  PHASE 3: BULK DATA (universe + history + enrichment)  ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Ticker universe seeding
    if let Some(ref polygon_k) = polygon_key {
        println!("3.1 Seeding ticker universe (Polygon)...");
        ticker_seed::run(Arc::clone(&pool), Arc::clone(&client), Arc::clone(polygon_k)).await;
    }
    if let Some(ref fmp_k) = fmp_key {
        println!("3.2 Seeding ticker universe (FMP)...");
        fmp_enrich::seed_ticker_universe(Arc::clone(&pool), Arc::clone(&client), Arc::clone(fmp_k)).await;
    }

    // Bulk price history
    if let Some(ref tiingo_k) = tiingo_key {
        println!("3.3 Fetching bulk price history (Tiingo, up to 490/day)...");
        tiingo::fetch_all_prices_tiingo(Arc::clone(&pool), Arc::clone(&client), Arc::clone(tiingo_k)).await;
    }

    // EDGAR filings (dynamic CIK lookup, priority tickers first)
    println!("3.4 Fetching SEC CIK map + EDGAR filings (priority + watchlist)...");
    let cik_map = match edgar_enrich::fetch_cik_map(&client, &user_agent).await {
        Ok(m) => {
            println!("     CIK map loaded ({} companies)", m.len());
            m
        }
        Err(e) => {
            eprintln!("     CIK map fetch failed ({e:#}), falling back to DB/default CIK map");
            std::collections::HashMap::new()
        }
    };
    edgar::fetch_all_edgar(Arc::clone(&pool), Arc::clone(&client), &cik_map, &priority).await;

    println!("3.5 Fetching 13F institutional holdings...");
    holdings::fetch_all_13f(Arc::clone(&pool), Arc::clone(&client)).await;

    // International economics (free, no API key)
    println!("3.6 Fetching DBnomics international macro data...");
    dbnomics::fetch_all_dbnomics(Arc::clone(&pool), Arc::clone(&client)).await;

    println!("3.7 Fetching RSS news feeds (25 sources)...");
    rss_news::fetch_all_rss(Arc::clone(&pool), Arc::clone(&client)).await;

    println!("3.7b Computing RSS tone sentiment...");
    rss_tone::compute_rss_tone(Arc::clone(&pool)).await;

    println!("3.8 Fetching Polymarket prediction markets...");
    polymarket::fetch_all_polymarket(Arc::clone(&pool), Arc::clone(&client)).await;

    println!("3.9a Fetching GDELT geopolitical events...");
    gdelt::fetch_gdelt_events(Arc::clone(&pool), Arc::clone(&client)).await;

    // IPO date enrichment pipeline (4 sources)
    println!("3.9 Enriching missing IPO dates (AV OVERVIEW)...");
    company_enrich::enrich_missing_ipo_dates(
        Arc::clone(&pool), Arc::clone(&client), Arc::clone(&api_key),
    ).await;

    if let Some(ref fmp_k) = fmp_key {
        println!("3.10 Enriching missing IPO dates (FMP profile)...");
        fmp_enrich::enrich_ipo_dates(Arc::clone(&pool), Arc::clone(&client), Arc::clone(fmp_k)).await;
    }

    println!("3.11 Enriching founding dates (Wikidata SPARQL)...");
    wikidata_enrich::enrich_founding_dates(
        Arc::clone(&pool), Arc::clone(&client), Arc::clone(&user_agent),
    ).await;

    println!("3.12 Enriching first-filing dates (SEC EDGAR)...");
    edgar_enrich::enrich_first_filing_dates_with_cik(
        Arc::clone(&pool), Arc::clone(&client), Arc::clone(&user_agent),
        if cik_map.is_empty() { None } else { Some(&cik_map) },
    ).await;

    println!("3.13 Enriching Wikipedia summaries (REST API)...");
    if let Err(e) = wikipedia::enrich_all(Arc::clone(&pool), Arc::clone(&client)).await {
        eprintln!("[wikipedia] enrich_all error: {e}");
    }

    // ── Wave 7 — native Rust providers ─────────────────────────────────
    println!("3.14 World Bank — economic indicators (free, no key)...");
    if let Err(e) = world_bank::fetch_all(Arc::clone(&pool), Arc::clone(&client)).await {
        eprintln!("[world_bank] error: {e:#}");
    }
    println!("3.15 CoinGecko — crypto market data (free tier)...");
    if let Err(e) = coingecko::fetch_all(Arc::clone(&pool), Arc::clone(&client)).await {
        eprintln!("[coingecko] error: {e:#}");
    }
    println!("3.16 Treasury Direct — daily yield curve...");
    if let Err(e) = treasury_direct::fetch_all(Arc::clone(&pool), Arc::clone(&client)).await {
        eprintln!("[treasury_direct] error: {e:#}");
    }
    println!("3.17 IMF — World Economic Outlook...");
    if let Err(e) = imf::fetch_all(Arc::clone(&pool), Arc::clone(&client)).await {
        eprintln!("[imf] error: {e:#}");
    }
    println!("3.18 ECB — Eurozone rates + FX...");
    if let Err(e) = ecb::fetch_all(Arc::clone(&pool), Arc::clone(&client)).await {
        eprintln!("[ecb] error: {e:#}");
    }
    println!("3.19 BLS — US labor + CPI detail...");
    if let Err(e) = bls::fetch_all(Arc::clone(&pool), Arc::clone(&client)).await {
        eprintln!("[bls] error: {e:#}");
    }
    println!("3.20 EIA — energy spot prices...");
    if let Err(e) = eia::fetch_all(Arc::clone(&pool), Arc::clone(&client)).await {
        eprintln!("[eia] error: {e:#}");
    }
    println!("3.21 CFTC COT — large-trader positioning...");
    if let Err(e) = cftc_cot::fetch_all(Arc::clone(&pool), Arc::clone(&client)).await {
        eprintln!("[cftc_cot] error: {e:#}");
    }
    println!("3.22 OFR — Financial Stress Index...");
    if let Err(e) = ofr::fetch_all(Arc::clone(&pool), Arc::clone(&client)).await {
        eprintln!("[ofr] error: {e:#}");
    }
    println!("3.23 SEC EDGAR — recent-filings firehose...");
    if let Err(e) = sec_recent::fetch_all(Arc::clone(&pool), Arc::clone(&client)).await {
        eprintln!("[sec_recent] error: {e:#}");
    }

    // =========================================================================
    // PHASE 4: COMPOSITE SCORING (astro-informed Lagrange Score)
    // =========================================================================

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  PHASE 4: LAGRANGE SCORING (astro-informed composite)  ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("4.1 Computing Lagrange scores...");
    lagrange::compute_all_scores(Arc::clone(&pool)).await;

    // =========================================================================
    // PHASE 5: PAPER TRADING SIMULATION (The Paper Trail)
    // =========================================================================
    // Runs after all scores are computed. Evaluates positions against Lagrange
    // thresholds, executes simulated BUY/SELL trades, updates paper account.
    // Idempotent: skips if already simulated for today's trading date.
    // =========================================================================

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  PHASE 5: PAPER TRADING (The Paper Trail simulation)   ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    paper_engine::run_simulation(pool).await;

    println!("\n Pipeline complete.\n");
}
