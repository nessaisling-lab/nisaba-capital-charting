//! Daily Lagrange Score computation — runs after all data fetches.
//! Reads the latest loaded data per ticker, computes the blended score,
//! and upserts one row per ticker per day into lagrange_history.

use std::sync::Arc;

use pursuit_week4_automation::indicators::{compute_lagrange_score, Indicators};
use pursuit_week4_automation::models::{AstroScore, MacroIndicator, PriceRow, SentimentScore, ShortInterest};

pub async fn compute_all_scores(pool: Arc<sqlx::PgPool>) {
    println!("Computing Lagrange scores for all tickers...");

    // --- Macro data (shared across all tickers) ---
    // MacroIndicator fields: series_id, series_name, obs_date, value
    // DB column is obs_date (see migration 0012).
    let macro_data: Vec<MacroIndicator> = sqlx::query_as(
        "SELECT DISTINCT ON (series_id)
             series_id, series_name, obs_date, value
         FROM macro_indicators
         ORDER BY series_id, obs_date DESC"
    )
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    let today = chrono::Utc::now().date_naive();

    // DB-driven universe: all tickers with scoring_active=true that have
    // at least 26 price rows.  Replaces the hardcoded WATCHLIST constant so
    // Lagrange scoring automatically expands as Tiingo fills in history.
    let scoreable: Vec<String> = sqlx::query_scalar(
        "SELECT pd.ticker
         FROM price_data pd
         JOIN tickers t ON t.ticker = pd.ticker
         WHERE t.scoring_active = true
         GROUP BY pd.ticker
         HAVING COUNT(*) >= 26
         ORDER BY pd.ticker"
    )
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    if scoreable.is_empty() {
        println!("[Lagrange] No tickers with scoring_active=true and 26+ price rows. Skipping.");
        return;
    }

    println!("[Lagrange] Scoring {} ticker(s)...", scoreable.len());

    for ticker in &scoreable {
        if let Err(e) = score_one_ticker(pool.as_ref(), ticker, &macro_data, today).await {
            eprintln!("Lagrange [{ticker}] error: {e}");
        }
    }

    println!("Lagrange scores done.");
}

async fn score_one_ticker(
    pool: &sqlx::PgPool,
    ticker: &str,
    macro_data: &[MacroIndicator],
    today: chrono::NaiveDate,
) -> anyhow::Result<()> {
    // Price rows (newest first, 200 rows for indicators)
    let rows: Vec<PriceRow> = sqlx::query_as(
        "SELECT ticker, date, open, high, low, close, volume
         FROM price_data
         WHERE ticker = $1
         ORDER BY date DESC
         LIMIT 200"
    )
    .bind(ticker)
    .fetch_all(pool)
    .await?;

    if rows.len() < 26 {
        // Not enough price history yet
        return Ok(());
    }

    // Build prices array oldest→newest for indicator math
    let prices: Vec<f32> = rows.iter().rev()
        .filter_map(|r| r.close.to_string().parse::<f32>().ok())
        .collect();

    let indicators = Indicators::compute(&prices);

    // Sentiment — SentimentScore fields: ticker, fetch_date, sentiment_score, sentiment_label
    let sentiment: Option<SentimentScore> = sqlx::query_as(
        "SELECT ticker, fetch_date, sentiment_score, sentiment_label
         FROM sentiment_scores
         WHERE ticker = $1
         ORDER BY fetch_date DESC
         LIMIT 1"
    )
    .bind(ticker)
    .fetch_optional(pool)
    .await?;

    // Astro score — AstroScore fields: ticker, score_date, astro_score, astro_label,
    //                                   moon_phase, moon_phase_deg, mercury_rx
    // (no computed_at column in the model)
    let astro_score: Option<AstroScore> = sqlx::query_as(
        "SELECT ticker, score_date, astro_score, astro_label,
                moon_phase, moon_phase_deg, mercury_rx
         FROM astro_scores
         WHERE ticker = $1
         ORDER BY score_date DESC
         LIMIT 1"
    )
    .bind(ticker)
    .fetch_optional(pool)
    .await?;

    // Short interest — ShortInterest fields: ticker, settlement_date, short_volume,
    //                                         total_volume, short_pct
    let short_interest: Option<ShortInterest> = sqlx::query_as(
        "SELECT ticker, settlement_date, short_volume, total_volume, short_pct
         FROM short_interest
         WHERE ticker = $1
         ORDER BY settlement_date DESC
         LIMIT 1"
    )
    .bind(ticker)
    .fetch_optional(pool)
    .await?;

    let (score, label, components) = compute_lagrange_score(
        &indicators,
        &rows,
        &sentiment,
        &astro_score,
        macro_data,
        &short_interest,
    );

    let concordance_name = components.concordance.name();

    sqlx::query(
        "INSERT INTO lagrange_history
            (ticker, score_date, score, label, fin_score, astro_score, macro_score, short_score, concordance)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT (ticker, score_date) DO NOTHING"
    )
    .bind(ticker)
    .bind(today)
    .bind(score)
    .bind(&label)
    .bind(components.fin_score)
    .bind(components.astro_score)
    .bind(components.macro_score)
    .bind(components.short_score)
    .bind(concordance_name)
    .execute(pool)
    .await?;

    // Crossing detection — compare today's label to the most recent prior day.
    // Uses ORDER BY score_date DESC LIMIT 1 (not = today - 1) so weekends/holidays
    // don't create spurious null gaps.
    check_alert_crossing(pool, ticker, today, score, &label).await;

    println!("  {ticker}: {score:.1} ({label}) [{}]", concordance_name);
    Ok(())
}

async fn check_alert_crossing(
    pool: &sqlx::PgPool,
    ticker: &str,
    today: chrono::NaiveDate,
    score: f32,
    label: &str,
) {
    // Look up the most recent prior label (skip if this is the ticker's first ever score)
    let prev_label: Option<String> = sqlx::query_scalar(
        "SELECT label FROM lagrange_history
         WHERE ticker = $1 AND score_date < $2
         ORDER BY score_date DESC LIMIT 1"
    )
    .bind(ticker)
    .bind(today)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let prev = match &prev_label {
        Some(p) => p.as_str(),
        None => return, // First ever score — no crossing to detect
    };

    let alert_type = match label {
        "Optimal"    if prev != "Optimal"    => "entered_optimal",
        "Misaligned" if prev != "Misaligned" => "entered_misaligned",
        _ => return, // No crossing into an extreme zone
    };

    let result = sqlx::query(
        "INSERT INTO lagrange_alerts
             (ticker, alert_date, score, label, prev_label, alert_type)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (ticker, alert_date, alert_type) DO NOTHING"
    )
    .bind(ticker)
    .bind(today)
    .bind(score)
    .bind(label)
    .bind(&prev_label)
    .bind(alert_type)
    .execute(pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 =>
            println!("  [ALERT] {ticker}: entered {label} (was {prev}) — score {score:.1}"),
        Ok(_)  => {} // Already recorded
        Err(e) => eprintln!("  [ALERT] Failed to store alert for {ticker}: {e}"),
    }
}
