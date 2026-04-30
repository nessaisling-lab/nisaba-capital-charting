//! RSS Tone Sentiment — keyword-based sentiment scoring from RSS feeds.
//!
//! Scans `rss_articles` (last 3 days) for ticker mentions in headline/summary,
//! scores each matched article using a financial word list, and stores daily
//! per-ticker tone scores in `rss_tone_scores`.
//!
//! Supplements Alpha Vantage NEWS_SENTIMENT which is rate-limited to 25 calls/day.
//! Uses the same -1.0 to +1.0 scale for interoperability with `sentiment_scores`.

use std::collections::HashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Financial sentiment word lists (Loughran-McDonald inspired)
// ---------------------------------------------------------------------------

const BULLISH_WORDS: &[&str] = &[
    "surge", "surges", "surging", "surged",
    "rally", "rallies", "rallying", "rallied",
    "beat", "beats", "beating",
    "upgrade", "upgrades", "upgraded",
    "record", "records",
    "growth", "growing",
    "soar", "soars", "soaring", "soared",
    "profit", "profits", "profitable",
    "gain", "gains", "gained",
    "outperform", "outperforms", "outperformed",
    "bullish",
    "strong", "stronger", "strongest", "strength",
    "breakout",
    "exceeds", "exceeded", "exceeding",
    "optimistic", "optimism",
    "boom", "booming",
    "recover", "recovery", "recovering",
    "upbeat",
    "positive",
];

const BEARISH_WORDS: &[&str] = &[
    "plunge", "plunges", "plunging", "plunged",
    "crash", "crashes", "crashing", "crashed",
    "miss", "misses", "missed", "missing",
    "downgrade", "downgrades", "downgraded",
    "loss", "losses",
    "decline", "declines", "declining", "declined",
    "sell", "selloff", "sell-off",
    "weak", "weaker", "weakest", "weakness",
    "bearish",
    "layoff", "layoffs",
    "bankruptcy",
    "default", "defaults", "defaulted",
    "warning", "warns", "warned",
    "underperform", "underperforms", "underperformed",
    "cut", "cuts", "cutting",
    "slump", "slumps", "slumping",
    "tumble", "tumbles", "tumbling",
    "pessimistic", "pessimism",
    "recession",
    "negative",
    "disappointing", "disappointed",
    "risk", "risky",
];

/// Recency window for RSS articles (days).
const RECENCY_DAYS: i64 = 3;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn compute_rss_tone(pool: Arc<sqlx::PgPool>) {
    // 1. Load ticker → company name mapping for matching
    let ticker_names: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT ticker, company_name FROM company_metadata ORDER BY ticker",
    )
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    if ticker_names.is_empty() {
        println!("[RSS Tone] No tickers in company_metadata — skipping.");
        return;
    }

    // Build lookup: lowercase name → ticker, plus ticker itself
    let mut name_to_ticker: HashMap<String, String> = HashMap::new();
    let mut ticker_set: Vec<String> = Vec::new();
    for (ticker, name) in &ticker_names {
        ticker_set.push(ticker.clone());
        if let Some(n) = name {
            let clean = n.trim().to_lowercase();
            // Skip ambiguous short names (< 4 chars) — too many false positives
            if clean.len() >= 4 {
                name_to_ticker.insert(clean, ticker.clone());
            }
        }
    }

    // 2. Load recent RSS articles (last N days)
    let cutoff = chrono::Utc::now().date_naive() - chrono::Duration::days(RECENCY_DAYS);
    let articles: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT headline, summary FROM rss_articles \
         WHERE published_at >= $1 ORDER BY published_at DESC",
    )
    .bind(cutoff)
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    if articles.is_empty() {
        println!("[RSS Tone] No recent articles (last {RECENCY_DAYS} days) — skipping.");
        return;
    }

    println!("[RSS Tone] Scanning {} articles against {} tickers...", articles.len(), ticker_set.len());

    // 3. Match articles to tickers and score
    let mut ticker_scores: HashMap<String, Vec<f64>> = HashMap::new();

    for (headline, summary) in &articles {
        let text = match summary {
            Some(s) => format!("{headline} {s}"),
            None => headline.clone(),
        };
        let text_lower = text.to_lowercase();
        let text_words: Vec<&str> = text_lower.split_whitespace().collect();

        // Find which tickers this article mentions
        let mut matched_tickers: Vec<String> = Vec::new();

        // Check ticker symbols (word-boundary match)
        for ticker in &ticker_set {
            let ticker_lower = ticker.to_lowercase();
            // Word-boundary check: ticker must appear as standalone word
            if text_words.iter().any(|w| {
                let stripped = w.trim_matches(|c: char| !c.is_alphanumeric());
                stripped == ticker_lower
            }) {
                matched_tickers.push(ticker.clone());
            }
        }

        // Check company names
        for (name, ticker) in &name_to_ticker {
            if text_lower.contains(name.as_str()) && !matched_tickers.contains(ticker) {
                matched_tickers.push(ticker.clone());
            }
        }

        if matched_tickers.is_empty() {
            continue;
        }

        // Score this article
        let score = score_text(&text_words);

        for ticker in matched_tickers {
            ticker_scores.entry(ticker).or_default().push(score);
        }
    }

    if ticker_scores.is_empty() {
        println!("[RSS Tone] No ticker mentions found in recent articles.");
        return;
    }

    // 4. Average and store
    let today = chrono::Utc::now().date_naive();
    let mut stored = 0u32;

    for (ticker, scores) in &ticker_scores {
        let avg = scores.iter().sum::<f64>() / scores.len() as f64;
        let label = tone_label(avg);
        let score_decimal = format!("{avg:.4}")
            .parse::<rust_decimal::Decimal>()
            .unwrap_or_default();

        let result = sqlx::query(
            "INSERT INTO rss_tone_scores (ticker, score_date, tone_score, tone_label, article_count) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (ticker, score_date) DO UPDATE SET \
                 tone_score = EXCLUDED.tone_score, \
                 tone_label = EXCLUDED.tone_label, \
                 article_count = EXCLUDED.article_count",
        )
        .bind(ticker)
        .bind(today)
        .bind(score_decimal)
        .bind(label)
        .bind(scores.len() as i32)
        .execute(pool.as_ref())
        .await;

        match result {
            Ok(_) => stored += 1,
            Err(e) => eprintln!("[RSS Tone] {ticker}: DB error: {e}"),
        }
    }

    println!("[RSS Tone] Scored {stored} tickers from {} matched articles.",
        ticker_scores.values().map(|v| v.len()).sum::<usize>());
    crate::log_fetch(&pool, "rss_tone", None, "rss_tone_scores", "ok", None).await;
}

// ---------------------------------------------------------------------------
// Scoring helpers
// ---------------------------------------------------------------------------

/// Score a single article's words. Returns -1.0 to +1.0.
fn score_text(words: &[&str]) -> f64 {
    let mut bullish = 0i32;
    let mut bearish = 0i32;

    for word in words {
        let stripped = word.trim_matches(|c: char| !c.is_alphanumeric());
        if BULLISH_WORDS.contains(&stripped) {
            bullish += 1;
        } else if BEARISH_WORDS.contains(&stripped) {
            bearish += 1;
        }
    }

    let total = bullish + bearish;
    if total == 0 {
        return 0.0; // Neutral if no sentiment words found
    }

    // Normalize to -1.0 .. +1.0
    (bullish - bearish) as f64 / total as f64
}

/// Map tone score to label (same thresholds as AV sentiment_label).
fn tone_label(score: f64) -> &'static str {
    if score <= -0.35      { "Bearish" }
    else if score <= -0.15 { "Somewhat-Bearish" }
    else if score < 0.15   { "Neutral" }
    else if score < 0.35   { "Somewhat-Bullish" }
    else                   { "Bullish" }
}
