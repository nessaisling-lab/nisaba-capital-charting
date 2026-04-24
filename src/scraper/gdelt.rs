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
const GDELT_QUERIES: &[(&str, &str)] = &[
    ("sanctions OR trade war OR tariff",     "trade"),
    ("central bank OR interest rate OR monetary policy", "monetary"),
    ("geopolitical risk OR military conflict OR war",   "conflict"),
    ("election instability OR regime change OR coup",    "political"),
    ("oil supply OR OPEC OR energy crisis",              "energy"),
];

const GDELT_API: &str = "https://api.gdeltproject.org/api/v2/doc/doc";

/// Maximum articles per query
const MAX_RECORDS: usize = 25;

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
            }
        }
    }

    if total_inserted > 0 {
        println!("[GDELT] Done. {total_inserted} new geopolitical articles stored.");
    }
    crate::log_fetch(&pool, "gdelt", None, "gdelt_events", "ok", None).await;
}

// ---------------------------------------------------------------------------
// Private: fetch one query category
// ---------------------------------------------------------------------------

async fn fetch_one_query(
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    query: &str,
    _category: &str,
) -> Result<u64> {
    // Build URL with query params manually (reqwest::query needs serde feature)
    let encoded_query = query.replace(' ', "+");
    let url = format!(
        "{}?query={}&mode=ArtList&maxrecords={}&format=json&timespan=1d&sourcelang=eng",
        GDELT_API, encoded_query, MAX_RECORDS
    );
    let resp = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send().await
        .context("GDELT HTTP request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("GDELT HTTP {}", resp.status());
    }

    let body = resp.text().await.context("Failed to read GDELT response")?;
    let parsed: GdeltResponse = serde_json::from_str(&body)
        .context("Failed to parse GDELT JSON")?;

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
