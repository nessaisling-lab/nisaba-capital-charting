use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Serde types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub(crate) struct AvSentimentResponse {
    pub feed: Option<Vec<AvSentimentArticle>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AvSentimentArticle {
    pub ticker_sentiment: Vec<AvTickerSentiment>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AvTickerSentiment {
    pub ticker: String,
    pub ticker_sentiment_score: String,
    pub relevance_score: String,
}

// ---------------------------------------------------------------------------

/// Returns how many Alpha Vantage calls have been logged today.
pub(crate) async fn av_calls_today(pool: &sqlx::PgPool) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM fetch_log \
         WHERE source = 'alpha_vantage' AND fetched_at::date = CURRENT_DATE",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0)
}

pub async fn fetch_av_sentiment_all(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    api_key: Arc<String>,
) {
    for ticker in crate::WATCHLIST {
        let calls_used = av_calls_today(&pool).await;
        if calls_used >= 24 {
            eprintln!("[sentiment] AV daily limit reached ({calls_used} calls today). Skipping remaining tickers.");
            break;
        }

        match fetch_av_sentiment(ticker, &pool, &client, &api_key).await {
            Ok(()) => {
                println!("[{ticker}] AV sentiment: stored");
                crate::log_fetch(&pool, "alpha_vantage", Some(ticker), "sentiment", "ok", None).await;
            }
            Err(e) => {
                eprintln!("[{ticker}] AV sentiment error (skipping): {e:#}");
                crate::log_fetch(&pool, "alpha_vantage", Some(ticker), "sentiment", "error", Some(&e.to_string())).await;
            }
        }
    }
}

async fn fetch_av_sentiment(
    ticker: &str,
    pool: &sqlx::PgPool,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<()> {
    let url = format!(
        "https://www.alphavantage.co/query\
         ?function=NEWS_SENTIMENT&tickers={ticker}&apikey={api_key}"
    );

    let resp = client.get(&url).send().await
        .context("AV sentiment request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("AV sentiment returned HTTP {}", resp.status());
    }

    let body: serde_json::Value = resp.json().await
        .context("Failed to parse AV sentiment response")?;

    if body.get("Note").is_some() || body.get("Information").is_some() {
        anyhow::bail!("AV rate limit or info message received");
    }

    let data: AvSentimentResponse = serde_json::from_value(body)
        .context("Failed to deserialize AV sentiment")?;

    let feed = match data.feed {
        Some(f) if !f.is_empty() => f,
        _ => return Ok(()),
    };

    let mut scores: Vec<f64> = Vec::new();
    for article in &feed {
        for ts in &article.ticker_sentiment {
            if ts.ticker != ticker { continue; }
            let relevance: f64 = ts.relevance_score.parse().unwrap_or(0.0);
            if relevance < 0.3 { continue; }
            if let Ok(score) = ts.ticker_sentiment_score.parse::<f64>() {
                scores.push(score);
            }
        }
    }

    if scores.is_empty() { return Ok(()); }

    let avg          = scores.iter().sum::<f64>() / scores.len() as f64;
    let label        = sentiment_label(avg);
    let score_decimal = format!("{avg:.4}")
        .parse::<rust_decimal::Decimal>()
        .unwrap_or_default();

    sqlx::query(
        "INSERT INTO sentiment_scores (ticker, fetch_date, sentiment_score, sentiment_label) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (ticker, fetch_date) DO NOTHING",
    )
    .bind(ticker)
    .bind(Utc::now().date_naive())
    .bind(score_decimal)
    .bind(label)
    .execute(pool)
    .await
    .context("Failed to insert sentiment score")?;

    Ok(())
}

fn sentiment_label(score: f64) -> &'static str {
    if score <= -0.35      { "Bearish" }
    else if score <= -0.15 { "Somewhat-Bearish" }
    else if score < 0.15   { "Neutral" }
    else if score < 0.35   { "Somewhat-Bullish" }
    else                   { "Bullish" }
}
