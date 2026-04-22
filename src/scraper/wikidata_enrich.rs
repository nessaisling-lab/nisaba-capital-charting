//! Wikidata SPARQL enrichment — founding/inception dates for US-listed companies.
//!
//! One HTTP call fetches up to 10,000 companies that have both a ticker symbol
//! (P249) and an inception date (P571) in Wikidata.  We then match those tickers
//! against company_metadata rows where ipo_date IS NULL and fill in the gaps.
//!
//! For old companies (JPMorgan 1799, Berkshire 1955, etc.) the Wikidata inception
//! date is often more meaningful astrologically than a modern listing date.
//!
//! Runs once per day max (logged in fetch_log).  No API key.  No rate limit beyond
//! Wikidata's courtesy preference for a descriptive User-Agent.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

const SPARQL_ENDPOINT: &str = "https://query.wikidata.org/sparql";

// Fetch companies with a ticker symbol AND an inception date.
//
// Tickers in Wikidata live in two places:
//   1. Direct property:  wdt:P249 on the company item (rarely populated)
//   2. P414 qualifier:   p:P414 → pq:P249 on the "traded on" statement (most data)
//
// We UNION both paths and filter to US exchanges (NYSE / NASDAQ / NYSE American)
// to keep the result set small and avoid the 60-second query timeout.
//
// Wikidata exchange QIDs:
//   wd:Q13677   — New York Stock Exchange (NYSE)
//   wd:Q117473  — NASDAQ
//   wd:Q1137652 — NYSE American (AMEX)
const SPARQL_QUERY: &str = r#"
SELECT DISTINCT ?ticker ?inceptionDate WHERE {
  {
    ?company wdt:P249 ?ticker .
  } UNION {
    ?company p:P414 ?exchangeStatement .
    ?exchangeStatement ps:P414 ?exchange ;
                       pq:P249 ?ticker .
    VALUES ?exchange { wd:Q13677 wd:Q117473 wd:Q1137652 }
  }
  ?company wdt:P571 ?inceptionDate .
  FILTER(STRLEN(?ticker) <= 10)
}
LIMIT 10000
"#;

// ---------------------------------------------------------------------------
// Serde types for SPARQL JSON response
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SparqlResponse {
    results: SparqlResults,
}

#[derive(Debug, Deserialize)]
struct SparqlResults {
    bindings: Vec<SparqlBinding>,
}

#[derive(Debug, Deserialize)]
struct SparqlBinding {
    ticker:        SparqlValue,
    #[serde(rename = "inceptionDate")]
    inception_date: SparqlValue,
}

#[derive(Debug, Deserialize)]
struct SparqlValue {
    value: String,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn enrich_founding_dates(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    user_agent: Arc<String>,
) {
    // Run once per day to avoid hammering Wikidata
    let ran_today: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM fetch_log \
         WHERE source = 'wikidata' AND fetched_at::date = CURRENT_DATE)",
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(false);

    if ran_today {
        println!("[Wikidata] Already ran today — skipping.");
        return;
    }

    println!("[Wikidata] Fetching company founding dates via SPARQL...");

    match fetch_and_enrich(&pool, &client, &user_agent).await {
        Ok((updated, seeded)) => {
            println!(
                "[Wikidata] Done — {updated} IPO dates filled, {seeded} natal charts seeded."
            );
            crate::log_fetch(pool.as_ref(), "wikidata", None, "sparql", "ok", None).await;
        }
        Err(e) => eprintln!("[Wikidata] SPARQL request failed: {e:#}"),
    }
}

// ---------------------------------------------------------------------------
// Internal
// ---------------------------------------------------------------------------

async fn fetch_and_enrich(
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    user_agent: &str,
) -> Result<(usize, usize)> {
    // Execute SPARQL query via POST form (reqwest handles URL encoding)
    let resp = client
        .post(SPARQL_ENDPOINT)
        .header("User-Agent", user_agent)
        .header("Accept", "application/sparql-results+json")
        .form(&[("query", SPARQL_QUERY), ("format", "json")])
        .send()
        .await
        .context("Wikidata SPARQL request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("Wikidata returned HTTP {}", resp.status());
    }

    let body: SparqlResponse = resp.json().await
        .context("Failed to parse Wikidata SPARQL response")?;

    println!(
        "[Wikidata] Received {} bindings from SPARQL.",
        body.results.bindings.len()
    );

    // Build ticker → date map (take first occurrence per ticker)
    let mut wikidata_dates: HashMap<String, NaiveDate> = HashMap::new();
    for binding in &body.results.bindings {
        let ticker = binding.ticker.value.trim().to_uppercase();
        if ticker.is_empty() { continue; }

        // Wikidata dates: "1977-04-01T00:00:00Z" or "+1977-04-01T00:00:00Z"
        let raw = binding.inception_date.value.trim_start_matches('+');
        let date_part = &raw[..raw.len().min(10)]; // take "YYYY-MM-DD"
        if let Ok(date) = date_part.parse::<NaiveDate>() {
            wikidata_dates.entry(ticker).or_insert(date);
        }
    }

    println!(
        "[Wikidata] {} unique tickers with valid inception dates.",
        wikidata_dates.len()
    );

    // Fetch tickers in our DB that still have null ipo_date
    let null_tickers: Vec<String> = sqlx::query_scalar(
        "SELECT ticker FROM company_metadata WHERE ipo_date IS NULL ORDER BY ticker",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut updated = 0usize;
    let mut seeded  = 0usize;

    for ticker in &null_tickers {
        if let Some(&date) = wikidata_dates.get(ticker.as_str()) {
            let result = sqlx::query(
                "UPDATE company_metadata SET ipo_date = $1 WHERE ticker = $2",
            )
            .bind(date)
            .bind(ticker)
            .execute(pool)
            .await;

            if result.is_ok() {
                updated += 1;
                crate::enrich_common::seed_one_natal_chart(pool, ticker, date).await;
                seeded += 1;
                println!("[Wikidata] {ticker}: inception {date} — natal chart seeded");
            }
        }
    }

    Ok((updated, seeded))
}

