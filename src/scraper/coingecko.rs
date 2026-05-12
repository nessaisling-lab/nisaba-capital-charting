//! Wave 7.2 — CoinGecko public API.
//!
//! Free tier (~10-50 req/min, no key for basic). Tracks:
//!   - Top 20 cryptocurrencies by market cap
//!   - BTC + ETH dominance %
//!   - Total crypto market cap
//!   - 24h trading volume aggregate
//!
//! Stored in `provider_observations` keyed by `provider = 'coingecko'`.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::Arc;

const API_BASE: &str = "https://api.coingecko.com/api/v3";

#[derive(Debug, Deserialize)]
struct CoinMarket {
    id: String,
    symbol: String,
    name: String,
    current_price: Option<f64>,
    market_cap: Option<f64>,
    total_volume: Option<f64>,
    price_change_percentage_24h: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct GlobalPayload {
    data: GlobalData,
}

#[derive(Debug, Deserialize)]
struct GlobalData {
    total_market_cap: std::collections::HashMap<String, f64>,
    total_volume: std::collections::HashMap<String, f64>,
    market_cap_percentage: std::collections::HashMap<String, f64>,
}

pub async fn fetch_all(pool: Arc<sqlx::PgPool>, client: Arc<reqwest::Client>) -> Result<()> {
    let today = chrono::Local::now().date_naive();
    let mut total = 0_i64;

    // 1. Global stats — total mcap, volume, BTC/ETH dominance
    let global_url = format!("{API_BASE}/global");
    match client.get(&global_url)
        .header("User-Agent", "NisabaEngine/0.1")
        .send().await
        .context("CoinGecko /global")
    {
        Ok(resp) if resp.status().is_success() => {
            let body: GlobalPayload = resp.json().await.context("CoinGecko /global parse")?;
            // Total market cap (USD)
            if let Some(&mcap) = body.data.total_market_cap.get("usd") {
                store_obs(&pool, "total_market_cap_usd",
                    "Crypto market cap (total, USD)", "USD", "GLOBAL",
                    today, mcap).await?;
                total += 1;
            }
            // Total 24h volume (USD)
            if let Some(&vol) = body.data.total_volume.get("usd") {
                store_obs(&pool, "total_volume_24h_usd",
                    "Crypto 24h volume (total, USD)", "USD", "GLOBAL",
                    today, vol).await?;
                total += 1;
            }
            // Dominance percentages
            for (symbol, &pct) in body.data.market_cap_percentage.iter() {
                if matches!(symbol.as_str(), "btc" | "eth") {
                    store_obs(&pool,
                        &format!("dominance_{symbol}"),
                        &format!("{} dominance (%)", symbol.to_uppercase()),
                        "%", "GLOBAL",
                        today, pct).await?;
                    total += 1;
                }
            }
        }
        Ok(resp) => eprintln!("[coingecko] /global HTTP {}", resp.status()),
        Err(e) => eprintln!("[coingecko] /global error: {e:#}"),
    }

    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

    // 2. Top 20 coins by market cap
    let coins_url = format!(
        "{API_BASE}/coins/markets?vs_currency=usd&order=market_cap_desc&per_page=20&page=1"
    );
    match client.get(&coins_url)
        .header("User-Agent", "NisabaEngine/0.1")
        .send().await
    {
        Ok(resp) if resp.status().is_success() => {
            let coins: Vec<CoinMarket> = resp.json().await.context("CoinGecko /coins parse")?;
            for coin in coins {
                let region = coin.symbol.to_uppercase();
                if let Some(price) = coin.current_price {
                    store_obs(&pool, "price_usd",
                        &format!("{} price (USD)", coin.name),
                        "USD", &region, today, price).await?;
                    total += 1;
                }
                if let Some(mcap) = coin.market_cap {
                    store_obs(&pool, "market_cap_usd",
                        &format!("{} market cap (USD)", coin.name),
                        "USD", &region, today, mcap).await?;
                    total += 1;
                }
                if let Some(vol) = coin.total_volume {
                    store_obs(&pool, "volume_24h_usd",
                        &format!("{} 24h volume (USD)", coin.name),
                        "USD", &region, today, vol).await?;
                    total += 1;
                }
                if let Some(chg) = coin.price_change_percentage_24h {
                    store_obs(&pool, "price_change_24h_pct",
                        &format!("{} 24h change (%)", coin.name),
                        "%", &region, today, chg).await?;
                    total += 1;
                }
                let _ = coin.id; // silence unused
            }
        }
        Ok(resp) => eprintln!("[coingecko] /coins HTTP {}", resp.status()),
        Err(e) => eprintln!("[coingecko] /coins error: {e:#}"),
    }

    println!("[coingecko] done — {} observations stored", total);
    Ok(())
}

async fn store_obs(
    pool: &sqlx::PgPool,
    series_id: &str,
    label: &str,
    unit: &str,
    region: &str,
    date: chrono::NaiveDate,
    value: f64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO provider_observations
            (provider, series_id, observation_date, value, label, region, unit, fetched_at)
         VALUES ('coingecko', $1, $2, $3, $4, $5, $6, NOW())
         ON CONFLICT (provider, series_id, region, observation_date)
         DO UPDATE SET value = EXCLUDED.value, fetched_at = NOW()",
    )
    .bind(series_id)
    .bind(date)
    .bind(value)
    .bind(label)
    .bind(region)
    .bind(unit)
    .execute(pool)
    .await
    .context("CoinGecko store")?;
    Ok(())
}
