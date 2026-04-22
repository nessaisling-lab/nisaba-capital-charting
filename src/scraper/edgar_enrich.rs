//! SEC EDGAR first-filing enrichment.
//!
//! Uses two free, unlimited EDGAR endpoints:
//!
//! 1. `https://www.sec.gov/files/company_tickers.json`
//!    One call, returns every SEC-registered company with its CIK number.
//!    Builds a ticker → CIK lookup table.
//!
//! 2. `https://data.sec.gov/submissions/CIK{padded}.json`
//!    Per-company filing history.  We scan for the earliest 10-K or S-1
//!    and use that date as a proxy for the IPO date.
//!
//! Form types considered: 10-K, 10-K405, 10-KSB, S-1, SB-2, 20-F, F-1
//! (S-1 = US IPO registration, 20-F / F-1 = foreign private issuer equivalents)
//!
//! Rate: EDGAR asks for ≤10 req/sec with a descriptive User-Agent.
//! We run 50 tickers per day at 200ms/request ≈ 5 req/sec.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

const EDGAR_TICKERS_URL: &str = "https://www.sec.gov/files/company_tickers.json";
const EDGAR_SUBMISSIONS_BASE: &str = "https://data.sec.gov/submissions";
const MAX_PER_RUN: usize = 50;

// Form types that indicate the company became public
const IPO_FORMS: &[&str] = &["10-K", "10-K405", "10-KSB", "S-1", "SB-2", "20-F", "F-1"];

// ---------------------------------------------------------------------------
// Serde types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct EdgarCompanyEntry {
    cik_str: u64,
    ticker:  String,
}

// company_tickers.json is keyed by sequential string integers: {"0": {...}, "1": {...}}
type CompanyTickersJson = HashMap<String, EdgarCompanyEntry>;

#[derive(Debug, Deserialize)]
struct EdgarSubmissions {
    filings: EdgarFilings,
}

#[derive(Debug, Deserialize)]
struct EdgarFilings {
    recent: EdgarRecentFilings,
    files:  Vec<EdgarFilingFile>,
}

#[derive(Debug, Deserialize)]
struct EdgarRecentFilings {
    form:        Vec<String>,
    #[serde(rename = "filingDate")]
    filing_date: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EdgarFilingFile {
    name:         String,
    #[serde(rename = "filingFrom")]
    filing_from:  Option<String>,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn enrich_first_filing_dates(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    user_agent: Arc<String>,
) {
    // Find tickers that still need a date, watchlist-first
    let order = crate::enrich_common::watchlist_priority_sql();
    let to_enrich: Vec<String> = sqlx::query_scalar(&format!(
        "SELECT ticker FROM company_metadata \
         WHERE ipo_date IS NULL \
         ORDER BY {order} \
         LIMIT {MAX_PER_RUN}"
    ))
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    if to_enrich.is_empty() {
        println!("[EDGAR-Enrich] All tickers have IPO dates. Nothing to enrich.");
        return;
    }

    println!(
        "[EDGAR-Enrich] Enriching {} ticker(s) via SEC first-filing dates...",
        to_enrich.len()
    );

    // Build CIK map from EDGAR (one call)
    let cik_map = match fetch_cik_map(&client, &user_agent).await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[EDGAR-Enrich] Failed to fetch CIK map: {e:#}");
            return;
        }
    };

    println!("[EDGAR-Enrich] CIK map loaded ({} companies).", cik_map.len());

    let mut enriched  = 0usize;
    let mut not_found = 0usize;
    // CIK → date cache: lets share-class variants (ADAMG, ADAMH…) reuse the
    // date already fetched for the primary ticker without an extra API call.
    let mut cik_date_cache = std::collections::HashMap::<u64, NaiveDate>::new();

    for ticker in &to_enrich {
        let cik = match cik_map.get(ticker.as_str()) {
            Some(c) => *c,
            None => { not_found += 1; continue; }
        };

        // Reuse cached date for share-class variants sharing the same CIK
        if let Some(&cached_date) = cik_date_cache.get(&cik) {
            let _ = sqlx::query(
                "UPDATE company_metadata SET ipo_date = $1 WHERE ticker = $2",
            )
            .bind(cached_date)
            .bind(ticker)
            .execute(pool.as_ref())
            .await;
            crate::enrich_common::seed_one_natal_chart(pool.as_ref(), ticker, cached_date).await;
            enriched += 1;
            continue;
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        match find_earliest_filing(cik, &client, &user_agent).await {
            Ok(Some(date)) => {
                let result = sqlx::query(
                    "UPDATE company_metadata SET ipo_date = $1 WHERE ticker = $2",
                )
                .bind(date)
                .bind(ticker)
                .execute(pool.as_ref())
                .await;

                if result.is_ok() {
                    cik_date_cache.insert(cik, date);
                    crate::enrich_common::seed_one_natal_chart(pool.as_ref(), ticker, date).await;
                    println!("[EDGAR-Enrich] {ticker}: first filing {date} — natal chart seeded");
                    enriched += 1;
                }
            }
            Ok(None)  => { not_found += 1; }
            Err(e)    => eprintln!("[EDGAR-Enrich] {ticker} (CIK {cik}): {e}"),
        }
    }

    println!(
        "[EDGAR-Enrich] Done — {enriched} dated, {not_found} not found in EDGAR."
    );
}

// ---------------------------------------------------------------------------
// Fetch ticker → CIK map
// ---------------------------------------------------------------------------

async fn fetch_cik_map(
    client: &reqwest::Client,
    user_agent: &str,
) -> Result<HashMap<String, u64>> {
    let resp = client
        .get(EDGAR_TICKERS_URL)
        .header("User-Agent", user_agent)
        .send()
        .await
        .context("EDGAR company_tickers.json request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("EDGAR tickers HTTP {}", resp.status());
    }

    let raw: CompanyTickersJson = resp.json().await
        .context("Failed to parse EDGAR company_tickers.json")?;

    let map: HashMap<String, u64> = raw
        .into_values()
        .map(|e| (e.ticker.to_uppercase(), e.cik_str))
        .collect();

    Ok(map)
}

// ---------------------------------------------------------------------------
// Find the earliest IPO-relevant filing for a given CIK
// ---------------------------------------------------------------------------

async fn find_earliest_filing(
    cik: u64,
    client: &reqwest::Client,
    user_agent: &str,
) -> Result<Option<NaiveDate>> {
    let padded = format!("{cik:010}");
    let url    = format!("{EDGAR_SUBMISSIONS_BASE}/CIK{padded}.json");

    let resp = client
        .get(&url)
        .header("User-Agent", user_agent)
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("EDGAR submissions HTTP {}", resp.status());
    }

    let body: EdgarSubmissions = resp.json().await
        .context("Failed to parse EDGAR submissions")?;

    // Scan recent filings for the earliest IPO-type form
    let recent_min: Option<NaiveDate> = body.filings.recent.form
        .iter()
        .zip(body.filings.recent.filing_date.iter())
        .filter(|(form, _)| IPO_FORMS.contains(&form.as_str()))
        .filter_map(|(_, date)| date.parse::<NaiveDate>().ok())
        .min();

    // If no older filing archive exists, return what we found in recent
    if body.filings.files.is_empty() {
        return Ok(recent_min);
    }

    // Find the oldest archived batch and fetch it (one more call)
    let oldest_file = body.filings.files.iter()
        .min_by_key(|f| f.filing_from.as_deref().unwrap_or("9999-12-31"));

    let archive_min = match oldest_file {
        None => None,
        Some(f) => {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            let archive_url = format!("{EDGAR_SUBMISSIONS_BASE}/{}", f.name);
            match fetch_archive_min(&archive_url, client, user_agent).await {
                Ok(d)  => d,
                Err(e) => {
                    eprintln!("[EDGAR-Enrich] Archive fetch failed ({archive_url}): {e}");
                    None
                }
            }
        }
    };

    // Return the earlier of the two
    Ok([recent_min, archive_min].into_iter().flatten().min())
}

async fn fetch_archive_min(
    url: &str,
    client: &reqwest::Client,
    user_agent: &str,
) -> Result<Option<NaiveDate>> {
    let resp = client
        .get(url)
        .header("User-Agent", user_agent)
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Archive HTTP {}", resp.status());
    }

    let body: EdgarRecentFilings = resp.json().await?;

    let min = body.form
        .iter()
        .zip(body.filing_date.iter())
        .filter(|(form, _)| IPO_FORMS.contains(&form.as_str()))
        .filter_map(|(_, date)| date.parse::<NaiveDate>().ok())
        .min();

    Ok(min)
}

// ---------------------------------------------------------------------------
// Natal chart seeder (same pattern as other enrich modules)
// ---------------------------------------------------------------------------

