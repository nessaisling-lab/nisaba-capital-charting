//! RSS/Atom news feed aggregator.
//!
//! Fetches articles from 25+ free RSS/Atom feeds across wire services,
//! financial media, central banks, and analysis blogs. No API key needed.
//! Parallel fetch with 5s timeout per feed.
//!
//! Ported from: reference/fincept_src/services/news/NewsService.cpp
//! (1327 lines of C++ reduced to ~200 lines of Rust by leveraging feed-rs)

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Feed registry — (url, source_name, category)
// ---------------------------------------------------------------------------
// Curated from FinceptTerminal's 80+ feeds, trimmed to 25 high-value sources.
// Categories: wire, markets, central_bank, analysis, tech, energy, crypto

const RSS_FEEDS: &[(&str, &str, &str)] = &[
    // Tier 1: Wire services & regulators
    ("https://feeds.reuters.com/reuters/businessNews",     "Reuters",       "wire"),
    ("https://feeds.reuters.com/reuters/financialsNews",   "Reuters",       "markets"),
    ("https://www.sec.gov/news/pressreleases.rss",         "SEC",           "central_bank"),
    ("https://www.federalreserve.gov/feeds/press_all.xml", "Federal Reserve","central_bank"),
    ("https://www.ecb.europa.eu/rss/press.html",           "ECB",           "central_bank"),

    // Tier 2: Major financial media
    ("https://feeds.bloomberg.com/markets/news.rss",       "Bloomberg",     "markets"),
    ("https://feeds.a.dj.com/rss/RSSMarketsMain.xml",     "WSJ",           "markets"),
    ("https://feeds.marketwatch.com/marketwatch/topstories/","MarketWatch", "markets"),
    ("https://search.cnbc.com/rs/search/combinedcms/view.xml?partnerId=wrss01&id=100003114", "CNBC", "markets"),
    ("https://seekingalpha.com/market_currents.xml",       "Seeking Alpha", "analysis"),

    // Tier 2: World news (macro context)
    ("http://feeds.bbci.co.uk/news/business/rss.xml",      "BBC",          "wire"),
    ("https://rss.nytimes.com/services/xml/rss/World.xml",  "NYT",         "wire"),
    ("https://www.aljazeera.com/xml/rss/all.xml",           "Al Jazeera",  "wire"),

    // Energy & commodities
    ("https://oilprice.com/rss/main",                       "OilPrice",    "energy"),

    // Tech
    ("https://techcrunch.com/feed/",                        "TechCrunch",   "tech"),
    ("https://www.finextra.com/rss/headlines.aspx",         "Finextra",     "tech"),

    // Asia-Pacific
    ("https://www.scmp.com/rss/91/feed",                    "SCMP",         "wire"),
    ("https://asia.nikkei.com/rss/feed/nar",                "Nikkei Asia",  "markets"),

    // Economics / analysis
    ("https://www.economist.com/finance-and-economics/rss.xml", "Economist","analysis"),
    ("https://feeds.feedburner.com/CalculatedRisk",         "Calculated Risk","analysis"),
    ("https://wolfstreet.com/feed/",                        "Wolf Street",  "analysis"),

    // Crypto
    ("https://www.coindesk.com/arc/outboundfeeds/rss/",     "CoinDesk",    "crypto"),
    ("https://cointelegraph.com/rss",                       "CoinTelegraph","crypto"),

    // Geopolitics
    ("https://foreignpolicy.com/feed/",                     "Foreign Policy","wire"),
    ("https://www.defensenews.com/rss/",                    "Defense News", "wire"),
];

/// Maximum summary length (chars) — matches C++ kSummaryMaxChars
const SUMMARY_MAX_CHARS: usize = 300;

/// Per-feed HTTP timeout
const FEED_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn fetch_all_rss(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
) {
    // Fetch all feeds in parallel via tokio::spawn (no futures crate needed)
    let futs: Vec<tokio::task::JoinHandle<()>> = RSS_FEEDS.iter().map(|&(url, source, category)| {
        let pool = Arc::clone(&pool);
        let client = Arc::clone(&client);
        tokio::spawn(async move {
            match fetch_one_feed(url, source, category, &pool, &client).await {
                Ok(n) => {
                    if n > 0 { println!("[RSS] {source}: {n} new articles"); }
                }
                Err(e) => {
                    eprintln!("[RSS] {source} error (skipping): {e:#}");
                }
            }
        })
    }).collect();
    for handle in futs {
        let _ = handle.await;
    }

    // Log aggregate result
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rss_articles")
        .fetch_one(pool.as_ref()).await.unwrap_or(0);
    println!("[RSS] Done. {total} total articles in database.");
    crate::log_fetch(&pool, "rss", None, "rss_articles", "ok", None).await;
}

// ---------------------------------------------------------------------------
// Private: fetch and parse one feed
// ---------------------------------------------------------------------------

async fn fetch_one_feed(
    url: &str,
    source: &str,
    category: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
) -> Result<u64> {
    let resp = client.get(url)
        .timeout(FEED_TIMEOUT)
        .send().await
        .context("HTTP request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("HTTP {}", resp.status());
    }

    let bytes = resp.bytes().await.context("Failed to read response body")?;
    let feed = feed_rs::parser::parse(&bytes[..])
        .context("Failed to parse feed")?;

    let mut inserted = 0u64;

    for entry in &feed.entries {
        let headline = entry.title.as_ref()
            .map(|t| t.content.clone())
            .unwrap_or_default();

        if headline.is_empty() { continue; }

        let link = entry.links.first()
            .map(|l| l.href.clone())
            .or_else(|| entry.id.clone().into())
            .unwrap_or_default();

        if link.is_empty() { continue; }

        let summary = entry.summary.as_ref()
            .map(|t| strip_html_and_truncate(&t.content, SUMMARY_MAX_CHARS))
            .or_else(|| entry.content.as_ref()
                .and_then(|c| c.body.as_ref())
                .map(|body| strip_html_and_truncate(body, SUMMARY_MAX_CHARS)));

        let published_at: DateTime<Utc> = entry.published
            .or(entry.updated)
            .unwrap_or_else(Utc::now);

        let result = sqlx::query(
            "INSERT INTO rss_articles (feed_source, category, headline, summary, link, published_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (link) DO NOTHING",
        )
        .bind(source)
        .bind(category)
        .bind(&headline)
        .bind(&summary)
        .bind(&link)
        .bind(published_at)
        .execute(pool)
        .await
        .context("DB insert failed")?;

        inserted += result.rows_affected();
    }

    Ok(inserted)
}

/// Strip HTML tags and truncate to max_chars. Matches C++ NewsService behavior.
fn strip_html_and_truncate(html: &str, max_chars: usize) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Collapse whitespace
    let cleaned: String = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.len() > max_chars {
        format!("{}...", &cleaned[..max_chars])
    } else {
        cleaned
    }
}
