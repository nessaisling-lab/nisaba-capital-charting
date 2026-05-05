//! Wikipedia summary scraper — v11.5.E1.
//!
//! Fills `wiki_summary` table from Wikipedia REST `/page/summary/{title}`.
//! Resolution strategy: company_name from `company_metadata` (preferred) or
//! `tickers` table, URL-encode, hit endpoint. Wikipedia auto-redirects to
//! canonical page so a fuzzy title still lands. 404 stores an empty
//! extract row so future runs respect the 30-day TTL gate instead of
//! hammering the API.
//!
//! No API key. Wikipedia's only rate-limit is per-IP courtesy
//! (~200 req/sec). One req per stale ticker is well under that.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::Arc;

const SUMMARY_URL: &str = "https://en.wikipedia.org/api/rest_v1/page/summary/";
const TTL_DAYS: i64 = 30;

/// Minimal percent-encoder for Wikipedia page titles. Spaces become `_`
/// (Wikipedia convention), then RFC 3986 reserved characters are %-encoded.
/// Not a general URL encoder — only safe for path segments.
fn encode_title(title: &str) -> String {
    let mut out = String::with_capacity(title.len() * 2);
    for ch in title.chars() {
        match ch {
            ' ' => out.push('_'),
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => out.push(ch),
            _ => {
                let mut buf = [0u8; 4];
                for byte in ch.encode_utf8(&mut buf).bytes() {
                    out.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    out
}

#[derive(Debug, Deserialize)]
struct WikiSummaryResponse {
    title: String,
    extract: Option<String>,
    thumbnail: Option<WikiThumb>,
    content_urls: Option<WikiContentUrls>,
}

#[derive(Debug, Deserialize)]
struct WikiThumb {
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WikiContentUrls {
    desktop: Option<WikiPageUrls>,
}

#[derive(Debug, Deserialize)]
struct WikiPageUrls {
    page: Option<String>,
}

/// Fetch + cache one ticker's Wikipedia summary. Returns Ok even when the
/// page is a 404 (caches an empty row to honor TTL).
pub async fn fetch_one(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    ticker: &str,
) -> Result<()> {
    // Skip if a fresh row already exists.
    let fresh: Option<(i32,)> = sqlx::query_as(
        "SELECT 1 FROM wiki_summary
         WHERE ticker = $1 AND fetched_at > NOW() - ($2 || ' days')::interval
         LIMIT 1",
    )
    .bind(ticker)
    .bind(TTL_DAYS.to_string())
    .fetch_optional(pool.as_ref())
    .await
    .context("wiki_summary freshness check")?;
    if fresh.is_some() {
        return Ok(());
    }

    let company_name: Option<String> = sqlx::query_scalar(
        "SELECT COALESCE(cm.company_name, t.ticker)
         FROM tickers t
         LEFT JOIN company_metadata cm ON cm.ticker = t.ticker
         WHERE t.ticker = $1",
    )
    .bind(ticker)
    .fetch_optional(pool.as_ref())
    .await
    .ok()
    .flatten();

    let title = company_name.unwrap_or_else(|| ticker.to_string());
    let encoded = encode_title(&title);
    let url = format!("{SUMMARY_URL}{encoded}");

    let resp = client
        .get(&url)
        .header("User-Agent", "PursuitAstro/0.1 (educational; pursuit.org)")
        .send()
        .await;

    let (title, extract, thumb_url, page_url) = match resp {
        Ok(r) if r.status().is_success() => {
            let body: WikiSummaryResponse = r
                .json()
                .await
                .context("wiki summary parse")?;
            let thumb = body.thumbnail.and_then(|t| t.source);
            let page = body.content_urls.and_then(|c| c.desktop).and_then(|d| d.page);
            (body.title, body.extract, thumb, page)
        }
        _ => (title, None, None, None),
    };

    sqlx::query(
        "INSERT INTO wiki_summary (ticker, title, extract, thumbnail_url, wikipedia_url, fetched_at)
         VALUES ($1, $2, $3, $4, $5, NOW())
         ON CONFLICT (ticker) DO UPDATE
            SET title = EXCLUDED.title,
                extract = EXCLUDED.extract,
                thumbnail_url = EXCLUDED.thumbnail_url,
                wikipedia_url = EXCLUDED.wikipedia_url,
                fetched_at = NOW()",
    )
    .bind(ticker)
    .bind(&title)
    .bind(&extract)
    .bind(&thumb_url)
    .bind(&page_url)
    .execute(pool.as_ref())
    .await
    .context("wiki_summary upsert")?;

    Ok(())
}

/// Universe-wide fetch — iterates active tickers, respects TTL gate.
pub async fn enrich_all(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
) -> Result<()> {
    let stale: Vec<String> = sqlx::query_scalar(
        "SELECT t.ticker
         FROM tickers t
         LEFT JOIN wiki_summary w ON w.ticker = t.ticker
         WHERE t.active = true
           AND (w.fetched_at IS NULL OR w.fetched_at < NOW() - ($1 || ' days')::interval)
         ORDER BY w.fetched_at NULLS FIRST
         LIMIT 50",
    )
    .bind(TTL_DAYS.to_string())
    .fetch_all(pool.as_ref())
    .await
    .context("wiki stale list")?;

    let total = stale.len();
    if total == 0 {
        println!("[wikipedia] all tickers fresh (TTL {TTL_DAYS}d), skipping");
        return Ok(());
    }
    println!("[wikipedia] enriching {total} tickers");
    let mut ok = 0;
    let mut err = 0;
    for ticker in stale {
        match fetch_one(Arc::clone(&pool), Arc::clone(&client), &ticker).await {
            Ok(_) => ok += 1,
            Err(_) => err += 1,
        }
        // Courtesy spacing — Wikipedia tolerates a flood but be polite.
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    }
    println!("[wikipedia] done: {ok} ok, {err} err");
    Ok(())
}
