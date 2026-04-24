use crate::error::SqlResultExt;
use pursuit_week4_automation::models::{
    AnalystRating, EarningsDate, FilingRow, FundamentalMetric, GdeltEvent, HoldingRow,
    InsiderTradeRow, MacroIndicator, NewsArticle, PolymarketMarket, PriceRow,
    RssArticle, SentimentScore, ShortInterest,
};
use sqlx::PgPool;
use std::sync::Arc;

pub async fn fetch_prices(pool: Arc<PgPool>, ticker: String) -> Result<Vec<PriceRow>, String> {
    sqlx::query_as::<_, PriceRow>(
        "SELECT ticker, date, open, high, low, close, volume \
         FROM price_data WHERE ticker = $1 ORDER BY date DESC LIMIT 100",
    )
    .bind(&ticker).fetch_all(pool.as_ref()).await.ctx("fetch_prices")
}

pub async fn fetch_holdings(pool: Arc<PgPool>, ticker: String) -> Result<Vec<HoldingRow>, String> {
    sqlx::query_as::<_, HoldingRow>(
        "SELECT institution_name, report_period, shares_held, market_value, investment_discretion \
         FROM institutional_holdings WHERE ticker = $1 ORDER BY shares_held DESC LIMIT 10",
    )
    .bind(&ticker).fetch_all(pool.as_ref()).await.ctx("fetch_holdings")
}

pub async fn fetch_8k_filings(pool: Arc<PgPool>, ticker: String) -> Result<Vec<FilingRow>, String> {
    sqlx::query_as::<_, FilingRow>(
        "SELECT ticker, filed_date, items, edgar_url \
         FROM filings WHERE ticker = $1 AND form_type = '8-K' \
         ORDER BY filed_date DESC LIMIT 10",
    )
    .bind(&ticker).fetch_all(pool.as_ref()).await.ctx("fetch_8k_filings")
}

pub async fn fetch_insider_trades(pool: Arc<PgPool>, ticker: String) -> Result<Vec<InsiderTradeRow>, String> {
    sqlx::query_as::<_, InsiderTradeRow>(
        "SELECT ticker, insider_name, insider_title, transaction_date, \
         transaction_type, shares, price_per_share \
         FROM insider_trades WHERE ticker = $1 \
         ORDER BY transaction_date DESC LIMIT 20",
    )
    .bind(&ticker).fetch_all(pool.as_ref()).await.ctx("fetch_insider_trades")
}

pub async fn fetch_news(pool: Arc<PgPool>, ticker: String) -> Result<Vec<NewsArticle>, String> {
    sqlx::query_as::<_, NewsArticle>(
        "SELECT ticker, headline, summary, source, url, published_at \
         FROM news_articles WHERE ticker = $1 \
         ORDER BY published_at DESC LIMIT 30",
    )
    .bind(&ticker).fetch_all(pool.as_ref()).await.ctx("fetch_news")
}

pub async fn fetch_polymarket(pool: Arc<PgPool>) -> Result<Vec<PolymarketMarket>, String> {
    sqlx::query_as::<_, PolymarketMarket>(
        "SELECT question, category, outcome_yes, outcome_no, volume, active \
         FROM polymarket_markets WHERE active = true \
         ORDER BY volume DESC NULLS LAST LIMIT 10",
    )
    .fetch_all(pool.as_ref()).await.ctx("fetch_polymarket")
}

pub async fn fetch_gdelt(pool: Arc<PgPool>) -> Result<Vec<GdeltEvent>, String> {
    sqlx::query_as::<_, GdeltEvent>(
        "SELECT id, url, title, source_country, tone, themes, locations, domain, published_at \
         FROM gdelt_events ORDER BY published_at DESC LIMIT 30",
    )
    .fetch_all(pool.as_ref()).await.ctx("fetch_gdelt")
}

pub async fn fetch_rss_articles(pool: Arc<PgPool>) -> Result<Vec<RssArticle>, String> {
    sqlx::query_as::<_, RssArticle>(
        "SELECT feed_source, category, headline, summary, link, published_at \
         FROM rss_articles ORDER BY published_at DESC LIMIT 50",
    )
    .fetch_all(pool.as_ref()).await.ctx("fetch_rss_articles")
}

pub async fn fetch_analyst_rating(pool: Arc<PgPool>, ticker: String) -> Result<Option<AnalystRating>, String> {
    sqlx::query_as::<_, AnalystRating>(
        "SELECT ticker, period, strong_buy, buy, hold, sell, strong_sell \
         FROM analyst_ratings WHERE ticker = $1 \
         ORDER BY period DESC LIMIT 1",
    )
    .bind(&ticker)
    .fetch_optional(pool.as_ref()).await.ctx("fetch_analyst_rating")
}

pub async fn fetch_sentiment(pool: Arc<PgPool>, ticker: String) -> Result<Option<SentimentScore>, String> {
    sqlx::query_as::<_, SentimentScore>(
        "SELECT ticker, fetch_date, sentiment_score, sentiment_label \
         FROM sentiment_scores WHERE ticker = $1 \
         ORDER BY fetch_date DESC LIMIT 1",
    )
    .bind(&ticker)
    .fetch_optional(pool.as_ref()).await.ctx("fetch_sentiment")
}

pub async fn fetch_short_interest(pool: Arc<PgPool>, ticker: String) -> Result<Option<ShortInterest>, String> {
    sqlx::query_as::<_, ShortInterest>(
        "SELECT ticker, settlement_date, short_volume, total_volume, short_pct \
         FROM short_interest WHERE ticker = $1 \
         ORDER BY settlement_date DESC LIMIT 1",
    )
    .bind(&ticker)
    .fetch_optional(pool.as_ref()).await.ctx("fetch_short_interest")
}

#[allow(dead_code)] // kept for future watchlist-wide earnings calendar view
pub async fn fetch_all_earnings(pool: Arc<PgPool>) -> Result<Vec<EarningsDate>, String> {
    sqlx::query_as::<_, EarningsDate>(
        "SELECT ticker, earnings_date, eps_estimate, eps_actual, revenue_estimate \
         FROM earnings_dates \
         ORDER BY earnings_date ASC",
    )
    .fetch_all(pool.as_ref()).await.ctx("fetch_all_earnings")
}

/// Fetch earnings for a specific ticker only.
pub async fn fetch_ticker_earnings(pool: Arc<PgPool>, ticker: String) -> Result<Vec<EarningsDate>, String> {
    sqlx::query_as::<_, EarningsDate>(
        "SELECT ticker, earnings_date, eps_estimate, eps_actual, revenue_estimate \
         FROM earnings_dates WHERE ticker = $1 \
         ORDER BY earnings_date ASC",
    )
    .bind(&ticker)
    .fetch_all(pool.as_ref()).await.ctx("fetch_ticker_earnings")
}

pub async fn fetch_fundamentals(
    pool: Arc<PgPool>,
    ticker: String,
) -> Result<Option<FundamentalMetric>, String> {
    sqlx::query_as::<_, FundamentalMetric>(
        "SELECT ticker, fetch_date,
                market_cap, pe_ratio, pb_ratio, ps_ratio, ev_ebitda, peg_ratio, price_to_fcf,
                roe, roa, net_margin, operating_margin,
                debt_equity, current_ratio,
                fcf, operating_cf, revenue, net_income, eps,
                dividend_yield, shares_outstanding
         FROM fundamental_metrics
         WHERE ticker = $1
         ORDER BY fetch_date DESC
         LIMIT 1",
    )
    .bind(&ticker)
    .fetch_optional(pool.as_ref())
    .await
    .ctx("fetch_fundamentals")
}

pub async fn fetch_macro_indicators(pool: Arc<PgPool>) -> Result<Vec<MacroIndicator>, String> {
    sqlx::query_as::<_, MacroIndicator>(
        "WITH latest AS (
             SELECT DISTINCT ON (series_id)
                 series_id, series_name, obs_date, value
             FROM macro_indicators
             ORDER BY series_id, obs_date DESC
         ),
         cpi_yoy AS (
             SELECT
                 'CPIAUCSL_YOY'::text AS series_id,
                 'CPI YoY %'::text    AS series_name,
                 cur.obs_date,
                 CASE WHEN prev.value IS NOT NULL AND prev.value != 0
                      THEN ROUND(((cur.value - prev.value) / prev.value * 100)::numeric, 2)
                      ELSE NULL END AS value
             FROM (SELECT DISTINCT ON (series_id) * FROM macro_indicators
                   WHERE series_id = 'CPIAUCSL' ORDER BY series_id, obs_date DESC) cur
             LEFT JOIN LATERAL (
                 SELECT value FROM macro_indicators
                 WHERE series_id = 'CPIAUCSL'
                   AND obs_date <= cur.obs_date - INTERVAL '12 months'
                 ORDER BY obs_date DESC LIMIT 1
             ) prev ON true
         )
         SELECT series_id, series_name, obs_date, value FROM latest
         UNION ALL
         SELECT series_id, series_name, obs_date, value FROM cpi_yoy",
    )
    .fetch_all(pool.as_ref()).await.ctx("fetch_macro_indicators")
}

pub async fn fetch_fear_greed() -> Result<(f32, String), String> {
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
