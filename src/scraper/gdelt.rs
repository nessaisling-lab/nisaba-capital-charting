//! GDELT 2.0 geopolitical event scraper.
//!
//! Fetches recent geopolitical articles from the GDELT DOC 2.0 API.
//! No API key required. Queries for finance-relevant themes:
//! trade wars, sanctions, central bank actions, political instability.
//!
//! API: https://api.gdeltproject.org/api/v2/doc/doc
//! Free, no auth, returns JSON with article metadata + tone scores.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::Arc;

/// GDELT queries targeting finance-relevant geopolitical themes.
/// v11.4 (Wave 6 follow-up) — GDELT requires explicit parens around OR clauses.
const GDELT_QUERIES: &[(&str, &str)] = &[
    ("(sanctions OR \"trade war\" OR tariff)",                       "trade"),
    ("(\"central bank\" OR \"interest rate\" OR \"monetary policy\")", "monetary"),
    ("(\"geopolitical risk\" OR \"military conflict\" OR war)",       "conflict"),
    ("(\"election instability\" OR \"regime change\" OR coup)",       "political"),
    ("(\"oil supply\" OR OPEC OR \"energy crisis\")",                  "energy"),
];

const GDELT_API: &str = "https://api.gdeltproject.org/api/v2/doc/doc";

/// Maximum articles per query
const MAX_RECORDS: usize = 25;

/// Retry config for transient failures (429, 503, timeouts)
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_SECS: u64 = 5;

// ---------------------------------------------------------------------------
// GDELT JSON response structures
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct GdeltResponse {
    articles: Option<Vec<GdeltArticle>>,
}

#[derive(Debug, Deserialize)]
struct GdeltArticle {
    url: Option<String>,
    title: Option<String>,
    #[serde(rename = "sourcecountry")]
    source_country: Option<String>,
    tone: Option<f64>,
    #[serde(rename = "seendate")]
    seen_date: Option<String>,  // "20260424T120000Z"
    domain: Option<String>,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn fetch_gdelt_events(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
) {
    let mut total_inserted = 0u64;
    let mut errors = 0u32;

    for &(query, category) in GDELT_QUERIES {
        match fetch_one_query(&pool, &client, query, category).await {
            Ok(n) => {
                if n > 0 {
                    println!("[GDELT] {category}: {n} new articles");
                }
                total_inserted += n;
            }
            Err(e) => {
                eprintln!("[GDELT] {category} error (skipping): {e:#}");
                errors += 1;
            }
        }
    }

    if total_inserted > 0 {
        println!("[GDELT] Done. {total_inserted} new geopolitical articles stored.");
    } else {
        println!("[GDELT] No new articles inserted.");
    }
    let status = if errors == 0 { "ok" } else if total_inserted > 0 { "partial" } else { "error" };
    crate::log_fetch(&pool, "gdelt", None, "gdelt_events", status, None).await;
}

// ---------------------------------------------------------------------------
// Private: fetch one query category
// ---------------------------------------------------------------------------

async fn fetch_one_query(
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    query: &str,
    category: &str,
) -> Result<u64> {
    let encoded_query = query.replace(' ', "+");
    let url = format!(
        "{}?query={}&mode=ArtList&maxrecords={}&format=json&timespan=1d&sourcelang=eng",
        GDELT_API, encoded_query, MAX_RECORDS
    );

    // Retry loop: handles 429, 503, and transient timeouts
    let body = {
        let mut last_err = None;
        let mut result_body = None;
        for attempt in 0..MAX_RETRIES {
            match client.get(&url)
                .timeout(std::time::Duration::from_secs(15))
                .send().await
            {
                Ok(resp) => {
                    let status = resp.status();
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS
                        || status == reqwest::StatusCode::SERVICE_UNAVAILABLE
                    {
                        let wait = RETRY_BASE_SECS * (1 << attempt);
                        eprintln!("[GDELT] {category}: HTTP {status}, retrying in {wait}s (attempt {}/{})",
                            attempt + 1, MAX_RETRIES);
                        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                        last_err = Some(anyhow::anyhow!("GDELT HTTP {status}"));
                        continue;
                    }
                    if !status.is_success() {
                        anyhow::bail!("GDELT HTTP {status}");
                    }
                    result_body = Some(resp.text().await.context("Failed to read GDELT response")?);
                    break;
                }
                Err(e) if e.is_timeout() && attempt + 1 < MAX_RETRIES => {
                    let wait = RETRY_BASE_SECS * (1 << attempt);
                    eprintln!("[GDELT] {category}: timeout, retrying in {wait}s (attempt {}/{})",
                        attempt + 1, MAX_RETRIES);
                    tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                    last_err = Some(e.into());
                    continue;
                }
                Err(e) => {
                    anyhow::bail!("GDELT request failed: {e}");
                }
            }
        }
        match result_body {
            Some(b) => b,
            None => return Err(last_err.unwrap_or_else(|| anyhow::anyhow!("GDELT: max retries exceeded"))),
        }
    };

    // Guard: empty or non-JSON response
    let body_trimmed = body.trim();
    if body_trimmed.is_empty() || !body_trimmed.starts_with('{') {
        if !body_trimmed.is_empty() {
            eprintln!("[GDELT] {category}: non-JSON response ({}B): {}",
                body.len(), &body[..body.len().min(120)]);
        }
        return Ok(0);
    }

    let parsed: GdeltResponse = match serde_json::from_str(body_trimmed) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[GDELT] {category}: JSON parse error: {e} — body preview: {}",
                &body[..body.len().min(200)]);
            return Ok(0);
        }
    };

    let articles = match parsed.articles {
        Some(a) => a,
        None => return Ok(0),
    };

    let mut inserted = 0u64;

    for article in &articles {
        let url = match &article.url {
            Some(u) if !u.is_empty() => u,
            _ => continue,
        };
        let title = match &article.title {
            Some(t) if !t.is_empty() => t,
            _ => continue,
        };

        let tone = article.tone.map(|t| t as f32);
        let domain = article.domain.as_deref();
        let source_country = article.source_country.as_deref();

        // Parse GDELT date format: "20260424T120000Z" -> DateTime
        let published_at = article.seen_date.as_ref()
            .and_then(|d| chrono::NaiveDateTime::parse_from_str(d, "%Y%m%dT%H%M%SZ").ok())
            .map(|dt| dt.and_utc())
            .unwrap_or_else(chrono::Utc::now);

        let result = sqlx::query(
            "INSERT INTO gdelt_events (url, title, source_country, tone, domain, published_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (url) DO NOTHING",
        )
        .bind(url)
        .bind(title)
        .bind(source_country)
        .bind(tone)
        .bind(domain)
        .bind(published_at)
        .execute(pool)
        .await
        .context("GDELT DB insert failed")?;

        inserted += result.rows_affected();
    }

    Ok(inserted)
}
