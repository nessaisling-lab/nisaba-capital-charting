//! Polymarket prediction markets fetcher.
//!
//! Fetches top active prediction markets from Polymarket's Gamma API.
//! No API key needed. Free, public endpoint.
//!
//! Three API bases (from FinceptTerminal):
//!   - Gamma (gamma-api.polymarket.com) — market discovery + metadata
//!   - CLOB (clob.polymarket.com) — order book + pricing
//!   - Data (data-api.polymarket.com) — analytics + history
//!
//! We use only Gamma for discovery: top markets by volume, filtered
//! to financially relevant categories (Economics, Politics, Crypto).
//!
//! Ported from: reference/fincept_src/services/polymarket/PolymarketService.cpp

use anyhow::{Context, Result};
use std::sync::Arc;

const GAMMA_BASE: &str = "https://gamma-api.polymarket.com";

// Financially relevant tags to query
const MARKET_TAGS: &[&str] = &[
    "economics",
    "fed",
    "inflation",
    "recession",
    "crypto",
    "elections",
];

// ---------------------------------------------------------------------------
// Serde types — Gamma API market response
// ---------------------------------------------------------------------------
// Gamma returns numeric fields as JSON strings ("1567632.01"), so we
// deserialize as String and parse manually, matching the C++ num_or_str().

#[derive(Debug, serde::Deserialize)]
struct GammaMarket {
    #[serde(default)]
    id: serde_json::Value, // can be string or number
    #[serde(default)]
    question: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    volume: serde_json::Value,
    #[serde(default)]
    liquidity: serde_json::Value,
    #[serde(default)]
    active: bool,
    #[serde(default)]
    closed: bool,
    #[serde(rename = "endDate", default)]
    end_date: Option<String>,
    #[serde(default)]
    #[allow(dead_code)] // deserialized for completeness; we use outcome_prices
    outcomes: serde_json::Value,      // can be array or JSON string of array
    #[serde(rename = "outcomePrices", default)]
    outcome_prices: serde_json::Value, // same
    #[serde(default)]
    tags: Vec<serde_json::Value>,
}

/// Parse a serde_json::Value that may be a number or a string containing a number.
fn num_or_str(v: &serde_json::Value) -> f64 {
    match v {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
        serde_json::Value::String(s) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Parse an outcomes/outcomePrices field that may be an array or a JSON string encoding an array.
fn parse_str_or_array(v: &serde_json::Value) -> Vec<String> {
    match v {
        serde_json::Value::Array(arr) => arr.iter().map(|x| {
            match x {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => String::new(),
            }
        }).collect(),
        serde_json::Value::String(s) => {
            // Try parsing as JSON array
            serde_json::from_str::<Vec<String>>(s).unwrap_or_default()
        }
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn fetch_all_polymarket(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
) {
    // Fetch top markets by volume across financially relevant tags
    let mut total_inserted = 0u64;

    for tag in MARKET_TAGS {
        match fetch_markets_by_tag(tag, &pool, &client).await {
            Ok(n) => {
                if n > 0 { println!("[Polymarket] tag={tag}: {n} markets upserted"); }
                total_inserted += n;
            }
            Err(e) => {
                eprintln!("[Polymarket] tag={tag} error: {e:#}");
            }
        }
    }

    // Also fetch top markets by volume regardless of tag
    match fetch_top_markets(&pool, &client).await {
        Ok(n) => {
            total_inserted += n;
            println!("[Polymarket] top volume: {n} markets upserted");
        }
        Err(e) => eprintln!("[Polymarket] top volume error: {e:#}"),
    }

    println!("[Polymarket] Done. {total_inserted} total upserts.");
    crate::log_fetch(&pool, "polymarket", None, "polymarket_markets", "ok", None).await;
}

// ---------------------------------------------------------------------------
// Private: fetch markets by tag
// ---------------------------------------------------------------------------

async fn fetch_markets_by_tag(
    tag: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
) -> Result<u64> {
    let url = format!(
        "{GAMMA_BASE}/markets?tag={tag}&closed=false&limit=10&order=volume&ascending=false"
    );
    fetch_and_upsert(&url, Some(tag), pool, client).await
}

async fn fetch_top_markets(
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
) -> Result<u64> {
    let url = format!(
        "{GAMMA_BASE}/markets?closed=false&limit=20&order=volume&ascending=false"
    );
    fetch_and_upsert(&url, None, pool, client).await
}

async fn fetch_and_upsert(
    url: &str,
    fallback_category: Option<&str>,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
) -> Result<u64> {
    let resp = client.get(url).send().await
        .context("Polymarket Gamma request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        anyhow::bail!("Gamma API returned HTTP {status}");
    }

    let markets: Vec<GammaMarket> = resp.json().await
        .context("Failed to parse Gamma response")?;

    let mut upserted = 0u64;

    for m in &markets {
        if m.question.is_empty() { continue; }

        let market_id = match &m.id {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => continue,
        };

        let volume = num_or_str(&m.volume);
        let liquidity = num_or_str(&m.liquidity);

        // Parse outcome prices — typically ["Yes", "No"] with prices ["0.65", "0.35"]
        let prices = parse_str_or_array(&m.outcome_prices);
        let outcome_yes: Option<f64> = prices.first().and_then(|p| p.parse().ok());
        let outcome_no: Option<f64> = prices.get(1).and_then(|p| p.parse().ok());

        // Category: prefer market.category, fall back to first tag, then query tag
        let category = m.category.clone()
            .or_else(|| m.tags.first().and_then(|t| {
                t.as_object().and_then(|o| o.get("label").and_then(|l| l.as_str().map(String::from)))
                    .or_else(|| t.as_str().map(String::from))
            }))
            .or_else(|| fallback_category.map(|s| {
                // Capitalize the tag for display: "economics" -> "Economics"
                let mut c = s.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            }));

        let result = sqlx::query(
            "INSERT INTO polymarket_markets \
                (market_id, question, category, outcome_yes, outcome_no, volume, liquidity, active, end_date, slug, fetched_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW()) \
             ON CONFLICT (market_id) DO UPDATE SET \
                outcome_yes = EXCLUDED.outcome_yes, \
                outcome_no = EXCLUDED.outcome_no, \
                volume = EXCLUDED.volume, \
                liquidity = EXCLUDED.liquidity, \
                active = EXCLUDED.active, \
                category = COALESCE(EXCLUDED.category, polymarket_markets.category), \
                fetched_at = NOW()",
        )
        .bind(&market_id)
        .bind(&m.question)
        .bind(&category)
        .bind(outcome_yes.map(|v| rust_decimal::Decimal::try_from(v).ok()).flatten())
        .bind(outcome_no.map(|v| rust_decimal::Decimal::try_from(v).ok()).flatten())
        .bind(rust_decimal::Decimal::try_from(volume).ok())
        .bind(rust_decimal::Decimal::try_from(liquidity).ok())
        .bind(!m.closed && m.active)
        .bind(&m.end_date)
        .bind(&m.slug)
        .execute(pool)
        .await
        .context("DB upsert failed")?;

        upserted += result.rows_affected();
    }

    Ok(upserted)
}
