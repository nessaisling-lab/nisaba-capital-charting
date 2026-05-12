use crate::error::SqlResultExt;
use chrono::Utc;
use nisaba_engine::models::{AstroScore, DailyTransit, NatalAngles, NatalPosition};
use sqlx::PgPool;
use std::sync::Arc;

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
    .ctx("fetch_astro_score")
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
    .ctx("fetch_natal_chart")
}

pub async fn fetch_natal_angles(pool: Arc<PgPool>, ticker: String) -> Result<Option<NatalAngles>, String> {
    sqlx::query_as::<_, NatalAngles>(
        "SELECT ticker, ascendant, mc FROM natal_angles WHERE ticker = $1",
    )
    .bind(&ticker)
    .fetch_optional(pool.as_ref())
    .await
    .ctx("fetch_natal_angles")
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
    .ctx("fetch_daily_transits")
}

/// A retrograde station event: a planet changing from direct to retrograde or vice versa.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RetroEvent {
    pub fetch_date: chrono::NaiveDate,
    pub planet:     String,
    pub station:    String, // "Rx" or "D"
}

/// Find retrograde station changes (Rx start / Direct start) within a date range.
/// Uses LAG window function to detect when `retrograde` flips for each planet.
pub async fn fetch_retrograde_events(
    pool: Arc<PgPool>,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> Result<Vec<RetroEvent>, String> {
    sqlx::query_as::<_, RetroEvent>(
        "SELECT fetch_date, planet, station FROM (
             SELECT fetch_date, planet, retrograde,
                    LAG(retrograde) OVER (PARTITION BY planet ORDER BY fetch_date) AS prev_rx,
                    CASE WHEN retrograde THEN 'Rx' ELSE 'D' END AS station
             FROM daily_transits
             WHERE fetch_date BETWEEN $1 AND $2
               AND planet IN ('Mercury', 'Venus', 'Mars', 'Jupiter', 'Saturn')
         ) sub
         WHERE retrograde IS DISTINCT FROM prev_rx
           AND prev_rx IS NOT NULL
         ORDER BY fetch_date"
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_retrograde_events")
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
    .ctx("fetch_astro_active_aspects")?;

    Ok(raw.unwrap_or(serde_json::Value::Array(vec![])))
}

/// Fetch the latest horoscope reading for a ticker.
/// Returns the JSONB reading column, reconstructed into HoroscopeReading.
pub async fn fetch_horoscope(
    pool: Arc<PgPool>,
    ticker: String,
) -> Result<Option<nisaba_engine::astrology::interpretation::HoroscopeReading>, String> {
    let today = chrono::Utc::now().date_naive();
    let row: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT reading FROM horoscope_readings \
         WHERE ticker = $1 AND reading_date = $2",
    )
    .bind(&ticker)
    .bind(today)
    .fetch_optional(pool.as_ref())
    .await
    .ctx("fetch_horoscope")?;

    Ok(row.and_then(|(json,)| {
        nisaba_engine::astrology::interpretation::horoscope_from_json(&ticker, today, &json)
    }))
}

/// Fetch astro scores for a ticker over a date range (for calendar view).
pub async fn fetch_astro_calendar(
    pool: Arc<PgPool>,
    ticker: String,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
) -> Result<Vec<(chrono::NaiveDate, f64, Option<String>)>, String> {
    sqlx::query_as(
        "SELECT score_date, astro_score, astro_label
         FROM astro_scores
         WHERE ticker = $1 AND score_date >= $2 AND score_date <= $3
           AND astro_score IS NOT NULL
         ORDER BY score_date ASC",
    )
    .bind(&ticker)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_astro_calendar")
}

/// One day of joined price + astro data for backtesting.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BacktestDayRow {
    pub date:        chrono::NaiveDate,
    pub close:       rust_decimal::Decimal,
    pub astro_score: f64,
}

/// Fetch joined price + astro data for a ticker, ordered by date ascending.
pub async fn fetch_backtest_data(
    pool: Arc<PgPool>,
    ticker: String,
) -> Result<Vec<BacktestDayRow>, String> {
    sqlx::query_as::<_, BacktestDayRow>(
        "SELECT p.date, p.close, a.astro_score
         FROM price_data p
         INNER JOIN astro_scores a ON a.ticker = p.ticker AND a.score_date = p.date
         WHERE p.ticker = $1 AND a.astro_score IS NOT NULL
         ORDER BY p.date ASC",
    )
    .bind(&ticker)
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_backtest_data")
}

