use crate::error::SqlResultExt;
use pursuit_week4_automation::models::{LagrangeAlert, LagrangeHistory};
use sqlx::PgPool;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Watchlist summary row (for the ranking panel)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WatchlistRow {
    pub ticker: String,
    pub astro_score: Option<f64>,
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
// Universe Explorer — paginated scored universe with filters
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UniverseRow {
    pub ticker:       String,
    pub company_name: Option<String>,
    pub sector:       Option<String>,
    pub score:        f32,
    pub label:        String,
    pub astro_score:  Option<f32>,
    pub fin_score:    Option<f32>,
    pub macro_score:  Option<f32>,
    pub short_score:  Option<f32>,
    pub concordance:  Option<String>,
}

/// Fetch a page of the scored universe, with optional zone, sector, and search filters.
/// `sort_col` is a SQL-safe column expression from `UniverseSortCol::sql_expr()`.
/// `sort_asc` controls ascending (true) vs descending (false) order.
pub async fn fetch_universe_page(
    pool: Arc<PgPool>,
    zone_filter: Option<String>,
    sector_filter: Option<String>,
    search: Option<String>,
    page: usize,
    page_size: usize,
    sort_col: &str,
    sort_asc: bool,
) -> Result<Vec<UniverseRow>, String> {
    let offset = (page * page_size) as i64;
    let limit = page_size as i64;
    // Convert search text to ILIKE pattern (prefix match on ticker, substring on company)
    let search_pattern = search.as_ref().map(|s| format!("%{}%", s.to_uppercase()));

    let direction = if sort_asc { "ASC" } else { "DESC" };
    // sort_col comes from our enum (not user input), so format! is safe here
    let query = format!(
        "WITH latest_astro AS (
             SELECT MAX(score_date) AS d FROM astro_scores
         ),
         latest_lagrange AS (
             SELECT MAX(score_date) AS d FROM lagrange_history
         )
         SELECT a.ticker,
                cm.company_name,
                cm.sector,
                COALESCE(lh.score, a.astro_score::real) AS score,
                COALESCE(lh.label, CASE
                    WHEN a.astro_score >= 70 THEN 'Optimal'
                    WHEN a.astro_score >= 55 THEN 'Favorable'
                    WHEN a.astro_score >= 45 THEN 'Neutral'
                    WHEN a.astro_score >= 30 THEN 'Unfavorable'
                    ELSE 'Misaligned'
                END) AS label,
                a.astro_score::real AS astro_score,
                lh.fin_score,
                lh.macro_score,
                lh.short_score,
                lh.concordance
         FROM astro_scores a
         CROSS JOIN latest_astro la
         LEFT JOIN company_metadata cm ON cm.ticker = a.ticker
         LEFT JOIN lagrange_history lh
              ON lh.ticker = a.ticker
             AND lh.score_date = (SELECT d FROM latest_lagrange)
         WHERE a.score_date = la.d
           AND ($1::text IS NULL OR COALESCE(lh.label, CASE
                    WHEN a.astro_score >= 70 THEN 'Optimal'
                    WHEN a.astro_score >= 55 THEN 'Favorable'
                    WHEN a.astro_score >= 45 THEN 'Neutral'
                    WHEN a.astro_score >= 30 THEN 'Unfavorable'
                    ELSE 'Misaligned'
                END) = $1)
           AND ($2::text IS NULL OR cm.sector = $2)
           AND ($5::text IS NULL
                OR UPPER(a.ticker) LIKE $5
                OR UPPER(COALESCE(cm.company_name, '')) LIKE $5)
         ORDER BY {sort_col} {direction} NULLS LAST
         LIMIT $3 OFFSET $4"
    );

    sqlx::query_as::<_, UniverseRow>(&query)
    .bind(&zone_filter)
    .bind(&sector_filter)
    .bind(limit)
    .bind(offset)
    .bind(&search_pattern)
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_universe_page")
}

/// Count total rows matching current filters (for pagination display).
pub async fn fetch_universe_count(
    pool: Arc<PgPool>,
    zone_filter: Option<String>,
    sector_filter: Option<String>,
    search: Option<String>,
) -> Result<i64, String> {
    let search_pattern = search.as_ref().map(|s| format!("%{}%", s.to_uppercase()));

    let count: Option<i64> = sqlx::query_scalar(
        "WITH latest_astro AS (
             SELECT MAX(score_date) AS d FROM astro_scores
         ),
         latest_lagrange AS (
             SELECT MAX(score_date) AS d FROM lagrange_history
         )
         SELECT COUNT(*)
         FROM astro_scores a
         CROSS JOIN latest_astro la
         LEFT JOIN company_metadata cm ON cm.ticker = a.ticker
         LEFT JOIN lagrange_history lh
              ON lh.ticker = a.ticker
             AND lh.score_date = (SELECT d FROM latest_lagrange)
         WHERE a.score_date = la.d
           AND ($1::text IS NULL OR COALESCE(lh.label, CASE
                    WHEN a.astro_score >= 70 THEN 'Optimal'
                    WHEN a.astro_score >= 55 THEN 'Favorable'
                    WHEN a.astro_score >= 45 THEN 'Neutral'
                    WHEN a.astro_score >= 30 THEN 'Unfavorable'
                    ELSE 'Misaligned'
                END) = $1)
           AND ($2::text IS NULL OR cm.sector = $2)
           AND ($3::text IS NULL
                OR UPPER(a.ticker) LIKE $3
                OR UPPER(COALESCE(cm.company_name, '')) LIKE $3)",
    )
    .bind(&zone_filter)
    .bind(&sector_filter)
    .bind(&search_pattern)
    .fetch_one(pool.as_ref())
    .await
    .ctx("fetch_universe_count")?;

    Ok(count.unwrap_or(0))
}

/// Fetch distinct sectors that have scored tickers.
pub async fn fetch_available_sectors(
    pool: Arc<PgPool>,
) -> Result<Vec<String>, String> {
    sqlx::query_scalar(
        "SELECT DISTINCT cm.sector
         FROM company_metadata cm
         JOIN lagrange_history lh ON lh.ticker = cm.ticker
         WHERE cm.sector IS NOT NULL
         ORDER BY cm.sector",
    )
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_available_sectors")
}

/// Fetch sector-level summary for heat map.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct SectorSummary {
    pub sector:         String,
    pub avg_astro:      Option<f64>,
    pub avg_lagrange:   Option<f64>,
    pub ticker_count:   i64,
}

pub async fn fetch_sector_summaries(
    pool: Arc<PgPool>,
) -> Result<Vec<SectorSummary>, String> {
    sqlx::query_as::<_, SectorSummary>(
        "WITH latest_date AS (
             SELECT MAX(score_date) AS d FROM lagrange_history
         )
         SELECT cm.sector,
                AVG(lh.astro_score)::float8 AS avg_astro,
                AVG(lh.score)::float8       AS avg_lagrange,
                COUNT(*)                     AS ticker_count
         FROM lagrange_history lh
         CROSS JOIN latest_date ld
         JOIN company_metadata cm ON cm.ticker = lh.ticker
         WHERE lh.score_date = ld.d
           AND cm.sector IS NOT NULL
         GROUP BY cm.sector
         ORDER BY AVG(lh.astro_score) DESC NULLS LAST",
    )
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_sector_summaries")
}

// ---------------------------------------------------------------------------
// Comparative analysis — fetch fundamentals + astro for multiple tickers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CompareRow {
    pub ticker:       String,
    pub pe_ratio:     Option<f64>,
    pub pb_ratio:     Option<f64>,
    pub ps_ratio:     Option<f64>,
    pub ev_ebitda:    Option<f64>,
    pub peg_ratio:    Option<f64>,
    pub roe:          Option<f64>,
    pub net_margin:   Option<f64>,
    pub debt_equity:  Option<f64>,
    pub fcf:          Option<i64>,
    pub market_cap:   Option<i64>,
    pub astro_score:  Option<f64>,
    pub astro_label:  Option<String>,
}

pub async fn fetch_compare_data(
    pool: Arc<PgPool>,
    tickers: Vec<String>,
) -> Result<Vec<CompareRow>, String> {
    sqlx::query_as::<_, CompareRow>(
        "SELECT f.ticker, f.pe_ratio, f.pb_ratio, f.ps_ratio, f.ev_ebitda,
                f.peg_ratio, f.roe, f.net_margin, f.debt_equity, f.fcf, f.market_cap,
                a.astro_score, a.astro_label
         FROM UNNEST($1::text[]) AS t(ticker)
         LEFT JOIN LATERAL (
             SELECT * FROM fundamental_metrics fm
             WHERE fm.ticker = t.ticker
             ORDER BY fm.fetch_date DESC LIMIT 1
         ) f ON true
         LEFT JOIN LATERAL (
             SELECT astro_score, astro_label FROM astro_scores asc_
             WHERE asc_.ticker = t.ticker
             ORDER BY asc_.score_date DESC LIMIT 1
         ) a ON true
         ORDER BY array_position($1::text[], t.ticker)",
    )
    .bind(&tickers)
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_compare_data")
}

/// Fetch up to 6 sector peers for a given ticker (same sector, exclude self).
pub async fn fetch_sector_peers(
    pool: Arc<PgPool>,
    ticker: String,
) -> Result<Vec<String>, String> {
    sqlx::query_scalar::<_, String>(
        "SELECT cm2.ticker
         FROM company_metadata cm1
         JOIN company_metadata cm2 ON cm2.sector = cm1.sector AND cm2.ticker != cm1.ticker
         WHERE cm1.ticker = $1 AND cm1.sector IS NOT NULL
         ORDER BY cm2.ticker
         LIMIT 6",
    )
    .bind(&ticker)
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_sector_peers")
}

// ---------------------------------------------------------------------------
// Market-wide queries
// ---------------------------------------------------------------------------

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
    .ctx("fetch_watchlist_summaries")
}

pub async fn fetch_market_fear_greed(pool: Arc<PgPool>) -> Result<(f32, String), String> {
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
    .fetch_one(pool.as_ref()).await.ctx("fetch_market_fear_greed/breadth")?;

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
    .fetch_one(pool.as_ref()).await.ctx("fetch_market_fear_greed/sentiment")?;

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
    .fetch_one(pool.as_ref()).await.ctx("fetch_market_fear_greed/buy_ratio")?;

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

pub async fn fetch_lagrange_history(
    pool: Arc<PgPool>,
    ticker: String,
) -> Result<Vec<LagrangeHistory>, String> {
    sqlx::query_as(
        "SELECT ticker, score_date, score, label, fin_score, astro_score, macro_score, short_score, concordance
         FROM lagrange_history
         WHERE ticker = $1
         ORDER BY score_date ASC
         LIMIT 90"
    )
    .bind(&ticker)
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_lagrange_history")
}

// ---------------------------------------------------------------------------
// Alerts
// ---------------------------------------------------------------------------

pub async fn fetch_alerts(pool: Arc<PgPool>) -> Result<Vec<LagrangeAlert>, String> {
    sqlx::query_as(
        "SELECT id, ticker, alert_date, score, label, prev_label, alert_type, is_read
         FROM lagrange_alerts
         ORDER BY alert_date DESC, ticker ASC
         LIMIT 50"
    )
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_alerts")
}

/// Fire-and-forget -- marks one alert as read in the DB.
pub async fn mark_alert_read(pool: Arc<PgPool>, id: i32) {
    if let Err(e) = sqlx::query("UPDATE lagrange_alerts SET is_read = true WHERE id = $1")
        .bind(id)
        .execute(pool.as_ref())
        .await
    {
        eprintln!("[Alerts] mark_alert_read({id}) failed: {e}");
    }
}

/// Mark all unread alerts as read.
pub async fn mark_all_alerts_read(pool: Arc<PgPool>) {
    if let Err(e) = sqlx::query("UPDATE lagrange_alerts SET is_read = true WHERE is_read = false")
        .execute(pool.as_ref())
        .await
    {
        eprintln!("[Alerts] mark_all_alerts_read failed: {e}");
    }
}

/// Delete a single alert from the DB.
pub async fn dismiss_alert(pool: Arc<PgPool>, id: i32) {
    if let Err(e) = sqlx::query("DELETE FROM lagrange_alerts WHERE id = $1")
        .bind(id)
        .execute(pool.as_ref())
        .await
    {
        eprintln!("[Alerts] dismiss_alert({id}) failed: {e}");
    }
}
