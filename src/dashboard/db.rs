use chrono::Utc;
use pursuit_week4_automation::models::{
    AnalystRating, AstroScore, DailyTransit, EarningsDate, FilingRow, HoldingRow,
    InsiderTradeRow, LagrangeHistory, MacroIndicator, NatalPosition, NewsArticle,
    PortfolioPosition, PriceRow, SentimentScore, ShortInterest,
};
use sqlx::PgPool;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Watchlist summary row (for the ranking panel)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WatchlistRow {
    pub ticker: String,
    pub astro_score: Option<i32>,
    pub astro_label: Option<String>,
    pub sentiment_score: Option<rust_decimal::Decimal>,
    pub sentiment_label: Option<String>,
    pub short_pct: Option<rust_decimal::Decimal>,
}

impl WatchlistRow {
    /// Quick composite score from available non-price signals.
    pub fn quick_score(&self) -> f32 {
        let astro = self.astro_score.map(|v| v as f32).unwrap_or(50.0);
        let sent = self.sentiment_score.as_ref()
            .and_then(|v| v.to_string().parse::<f32>().ok())
            .map(|v| (v + 1.0) * 50.0)
            .unwrap_or(50.0);
        (astro * 0.6 + sent * 0.4).clamp(0.0, 100.0)
    }
}

// ---------------------------------------------------------------------------
// Database connection
// ---------------------------------------------------------------------------

pub async fn connect_db(url: String) -> Result<Arc<PgPool>, String> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(3)
        .connect(&url)
        .await
        .map(Arc::new)
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Query helpers
// ---------------------------------------------------------------------------

pub async fn load_tickers(pool: Arc<PgPool>) -> Result<Vec<String>, String> {
    sqlx::query_scalar::<_, String>(
        "SELECT ticker FROM tickers WHERE active = true ORDER BY ticker ASC",
    )
    .fetch_all(pool.as_ref()).await.map_err(|e| e.to_string())
}

pub async fn fetch_prices(pool: Arc<PgPool>, ticker: String) -> Result<Vec<PriceRow>, String> {
    sqlx::query_as::<_, PriceRow>(
        "SELECT ticker, date, open, high, low, close, volume \
         FROM price_data WHERE ticker = $1 ORDER BY date DESC LIMIT 100",
    )
    .bind(&ticker).fetch_all(pool.as_ref()).await.map_err(|e| e.to_string())
}

pub async fn fetch_holdings(pool: Arc<PgPool>, ticker: String) -> Result<Vec<HoldingRow>, String> {
    sqlx::query_as::<_, HoldingRow>(
        "SELECT institution_name, report_period, shares_held, market_value, investment_discretion \
         FROM institutional_holdings WHERE ticker = $1 ORDER BY shares_held DESC LIMIT 10",
    )
    .bind(&ticker).fetch_all(pool.as_ref()).await.map_err(|e| e.to_string())
}

pub async fn fetch_8k_filings(pool: Arc<PgPool>, ticker: String) -> Result<Vec<FilingRow>, String> {
    sqlx::query_as::<_, FilingRow>(
        "SELECT ticker, filed_date, items, edgar_url \
         FROM filings WHERE ticker = $1 AND form_type = '8-K' \
         ORDER BY filed_date DESC LIMIT 10",
    )
    .bind(&ticker).fetch_all(pool.as_ref()).await.map_err(|e| e.to_string())
}

pub async fn fetch_insider_trades(pool: Arc<PgPool>, ticker: String) -> Result<Vec<InsiderTradeRow>, String> {
    sqlx::query_as::<_, InsiderTradeRow>(
        "SELECT ticker, insider_name, insider_title, transaction_date, \
         transaction_type, shares, price_per_share \
         FROM insider_trades WHERE ticker = $1 \
         ORDER BY transaction_date DESC LIMIT 20",
    )
    .bind(&ticker).fetch_all(pool.as_ref()).await.map_err(|e| e.to_string())
}

pub async fn fetch_news(pool: Arc<PgPool>, ticker: String) -> Result<Vec<NewsArticle>, String> {
    sqlx::query_as::<_, NewsArticle>(
        "SELECT ticker, headline, summary, source, url, published_at \
         FROM news_articles WHERE ticker = $1 \
         ORDER BY published_at DESC LIMIT 30",
    )
    .bind(&ticker).fetch_all(pool.as_ref()).await.map_err(|e| e.to_string())
}

pub async fn fetch_analyst_rating(pool: Arc<PgPool>, ticker: String) -> Result<Option<AnalystRating>, String> {
    sqlx::query_as::<_, AnalystRating>(
        "SELECT ticker, period, strong_buy, buy, hold, sell, strong_sell \
         FROM analyst_ratings WHERE ticker = $1 \
         ORDER BY period DESC LIMIT 1",
    )
    .bind(&ticker)
    .fetch_optional(pool.as_ref()).await.map_err(|e| e.to_string())
}

pub async fn fetch_sentiment(pool: Arc<PgPool>, ticker: String) -> Result<Option<SentimentScore>, String> {
    sqlx::query_as::<_, SentimentScore>(
        "SELECT ticker, fetch_date, sentiment_score, sentiment_label \
         FROM sentiment_scores WHERE ticker = $1 \
         ORDER BY fetch_date DESC LIMIT 1",
    )
    .bind(&ticker)
    .fetch_optional(pool.as_ref()).await.map_err(|e| e.to_string())
}

pub async fn fetch_astro_score(pool: Arc<PgPool>, ticker: String) -> Result<Option<AstroScore>, String> {
    let today = Utc::now().date_naive();
    sqlx::query_as::<_, AstroScore>(
        "SELECT ticker, score_date, astro_score, astro_label, moon_phase, moon_phase_deg, mercury_rx \
         FROM astro_scores \
         WHERE ticker = $1 AND score_date = $2",
    )
    .bind(&ticker)
    .bind(today)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| e.to_string())
}

pub async fn fetch_natal_chart(pool: Arc<PgPool>, ticker: String) -> Result<Vec<NatalPosition>, String> {
    sqlx::query_as::<_, NatalPosition>(
        "SELECT ticker, planet, longitude, sign, degree, retrograde \
         FROM natal_positions WHERE ticker = $1 \
         ORDER BY longitude ASC",
    )
    .bind(&ticker)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| e.to_string())
}

pub async fn fetch_daily_transits(pool: Arc<PgPool>) -> Result<Vec<DailyTransit>, String> {
    let today = Utc::now().date_naive();
    sqlx::query_as::<_, DailyTransit>(
        "SELECT fetch_date, planet, longitude, sign, retrograde \
         FROM daily_transits WHERE fetch_date = $1 \
         ORDER BY longitude ASC",
    )
    .bind(today)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| e.to_string())
}

pub async fn fetch_astro_active_aspects(pool: Arc<PgPool>, ticker: String) -> Result<serde_json::Value, String> {
    let today = Utc::now().date_naive();
    let raw: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT active_aspects FROM astro_scores WHERE ticker = $1 AND score_date = $2",
    )
    .bind(&ticker)
    .bind(today)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    Ok(raw.unwrap_or(serde_json::Value::Array(vec![])))
}

pub async fn fetch_macro_indicators(pool: Arc<PgPool>) -> Result<Vec<MacroIndicator>, String> {
    // Fetch latest per series, plus a synthetic CPI YoY% row.
    // Column aliases match MacroIndicator struct field names (obs_date, not observation_date).
    sqlx::query_as::<_, MacroIndicator>(
        "WITH latest AS (
             SELECT DISTINCT ON (series_id)
                 series_id, series_name,
                 observation_date AS obs_date,
                 value
             FROM macro_indicators
             ORDER BY series_id, observation_date DESC
         ),
         cpi_yoy AS (
             SELECT
                 'CPIAUCSL_YOY'::text AS series_id,
                 'CPI YoY %'::text    AS series_name,
                 cur.observation_date AS obs_date,
                 CASE WHEN prev.value IS NOT NULL AND prev.value != 0
                      THEN ROUND(((cur.value - prev.value) / prev.value * 100)::numeric, 2)
                      ELSE NULL END AS value
             FROM (SELECT DISTINCT ON (series_id) * FROM macro_indicators
                   WHERE series_id = 'CPIAUCSL' ORDER BY series_id, observation_date DESC) cur
             LEFT JOIN LATERAL (
                 SELECT value FROM macro_indicators
                 WHERE series_id = 'CPIAUCSL'
                   AND observation_date <= cur.observation_date - INTERVAL '12 months'
                 ORDER BY observation_date DESC LIMIT 1
             ) prev ON true
         )
         SELECT series_id, series_name, obs_date, value FROM latest
         UNION ALL
         SELECT series_id, series_name, obs_date, value FROM cpi_yoy",
    )
    .fetch_all(pool.as_ref()).await.map_err(|e| e.to_string())
}

pub async fn fetch_short_interest(pool: Arc<PgPool>, ticker: String) -> Result<Option<ShortInterest>, String> {
    sqlx::query_as::<_, ShortInterest>(
        "SELECT ticker, settlement_date, short_volume, total_volume, short_pct \
         FROM short_interest WHERE ticker = $1 \
         ORDER BY settlement_date DESC LIMIT 1",
    )
    .bind(&ticker)
    .fetch_optional(pool.as_ref()).await.map_err(|e| e.to_string())
}

pub async fn fetch_all_earnings(pool: Arc<PgPool>) -> Result<Vec<EarningsDate>, String> {
    sqlx::query_as::<_, EarningsDate>(
        "SELECT ticker, earnings_date, eps_estimate, eps_actual, revenue_estimate \
         FROM earnings_dates \
         ORDER BY earnings_date ASC",
    )
    .fetch_all(pool.as_ref()).await.map_err(|e| e.to_string())
}

pub async fn fetch_market_fear_greed(pool: Arc<PgPool>) -> Result<(f32, String), String> {
    // Component 1: Breadth — % of watchlist tickers where latest close > 50-day SMA
    let breadth: f64 = sqlx::query_scalar(
        "WITH ranked AS (
             SELECT ticker, close::float8 AS close,
                    ROW_NUMBER() OVER (PARTITION BY ticker ORDER BY date DESC) AS rn
             FROM price_data
         ),
         latest AS (SELECT ticker, close FROM ranked WHERE rn = 1),
         sma    AS (SELECT ticker, AVG(close) AS sma50 FROM ranked WHERE rn <= 50 GROUP BY ticker)
         SELECT COALESCE(
             AVG(CASE WHEN l.close > s.sma50 THEN 100.0::float8 ELSE 0.0::float8 END),
             50.0::float8)
         FROM latest l JOIN sma s USING (ticker)",
    )
    .fetch_one(pool.as_ref()).await.map_err(|e| e.to_string())?;

    // Component 2: Average AV news sentiment across all tickers, mapped to 0-100
    let sentiment: f64 = sqlx::query_scalar(
        "SELECT COALESCE(
             AVG((sentiment_score::float8 + 1.0::float8) * 50.0::float8),
             50.0::float8)
         FROM (
             SELECT DISTINCT ON (ticker) sentiment_score
             FROM sentiment_scores
             WHERE sentiment_score IS NOT NULL
             ORDER BY ticker, fetch_date DESC
         ) sub",
    )
    .fetch_one(pool.as_ref()).await.map_err(|e| e.to_string())?;

    // Component 3: Analyst buy ratio — (strong_buy + buy) / total, averaged across tickers
    let buy_ratio: f64 = sqlx::query_scalar(
        "SELECT COALESCE(
             AVG(CASE WHEN (strong_buy+buy+hold+sell+strong_sell) > 0
                      THEN (strong_buy+buy)::float8
                           / (strong_buy+buy+hold+sell+strong_sell)::float8
                           * 100.0::float8
                      ELSE 50.0::float8 END),
             50.0::float8)
         FROM (
             SELECT DISTINCT ON (ticker) strong_buy, buy, hold, sell, strong_sell
             FROM analyst_ratings
             ORDER BY ticker, period DESC
         ) sub",
    )
    .fetch_one(pool.as_ref()).await.map_err(|e| e.to_string())?;

    let score = (breadth * 0.50 + sentiment * 0.30 + buy_ratio * 0.20) as f32;
    let label = match score as u32 {
        0..=24  => "Extreme Fear",
        25..=44 => "Fear",
        45..=55 => "Neutral",
        56..=75 => "Greed",
        _       => "Extreme Greed",
    }.to_string();

    Ok((score, label))
}

pub async fn fetch_watchlist_summaries(pool: Arc<PgPool>) -> Result<Vec<WatchlistRow>, String> {
    sqlx::query_as::<_, WatchlistRow>(
        "WITH
         latest_astro AS (
             SELECT DISTINCT ON (ticker) ticker, astro_score, astro_label
             FROM astro_scores ORDER BY ticker, score_date DESC
         ),
         latest_sent AS (
             SELECT DISTINCT ON (ticker) ticker, sentiment_score, sentiment_label
             FROM sentiment_scores ORDER BY ticker, fetch_date DESC
         ),
         latest_si AS (
             SELECT DISTINCT ON (ticker) ticker, short_pct
             FROM short_interest ORDER BY ticker, settlement_date DESC
         )
         SELECT t.ticker,
                a.astro_score, a.astro_label,
                s.sentiment_score, s.sentiment_label,
                i.short_pct
         FROM tickers t
         LEFT JOIN latest_astro a ON a.ticker = t.ticker
         LEFT JOIN latest_sent  s ON s.ticker = t.ticker
         LEFT JOIN latest_si    i ON i.ticker = t.ticker
         WHERE t.active = true
         ORDER BY t.ticker",
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| e.to_string())
}

pub async fn fetch_fear_greed() -> Result<(f32, String), String> {
    // alternative.me Fear & Greed Index — free, bot-friendly, no API key needed.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let text = client
        .get("https://api.alternative.me/fng/?limit=1")
        .header("User-Agent", "FinancialDashboard/1.0")
        .send()
        .await
        .map_err(|e| format!("request: {e}"))?
        .text()
        .await
        .map_err(|e| format!("body: {e}"))?;

    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("json({e}): {}", &text[..text.len().min(120)]))?;

    let entry = &v["data"][0];

    let score = entry["value"]
        .as_str()
        .ok_or_else(|| format!("missing value in: {}", &text[..text.len().min(200)]))?
        .parse::<f32>()
        .map_err(|e| format!("parse score: {e}"))?;

    let label = entry["value_classification"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    Ok((score, label))
}

// ---------------------------------------------------------------------------
// Lagrange score history (for sparkline)
// ---------------------------------------------------------------------------

pub async fn fetch_lagrange_history(
    pool: Arc<PgPool>,
    ticker: String,
) -> Result<Vec<LagrangeHistory>, String> {
    sqlx::query_as(
        "SELECT ticker, score_date, score, label, fin_score, astro_score, macro_score, short_score
         FROM lagrange_history
         WHERE ticker = $1
         ORDER BY score_date ASC
         LIMIT 90"
    )
    .bind(&ticker)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Portfolio positions
// ---------------------------------------------------------------------------

pub async fn fetch_portfolio(pool: Arc<PgPool>) -> Result<Vec<PortfolioPosition>, String> {
    sqlx::query_as(
        "SELECT ticker, shares, avg_cost, notes
         FROM portfolio_positions
         ORDER BY ticker"
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| e.to_string())
}
