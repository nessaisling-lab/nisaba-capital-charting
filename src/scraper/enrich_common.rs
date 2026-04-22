//! Shared helpers used by all enrichment modules.

use chrono::NaiveDate;
use pursuit_week4_automation::astrology::natal::NatalChart;

// ---------------------------------------------------------------------------
// Natal chart seeder
// ---------------------------------------------------------------------------

/// Inserts or updates natal positions for `ticker` using `ipo_date` as the
/// birth date.  Called by every enrichment module after setting ipo_date.
pub async fn seed_one_natal_chart(pool: &sqlx::PgPool, ticker: &str, ipo_date: NaiveDate) {
    let chart = NatalChart::compute(ticker, ipo_date);
    for pos in &chart.positions {
        let _ = sqlx::query(
            "INSERT INTO natal_positions \
             (ticker, planet, longitude, sign, degree, retrograde) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (ticker, planet) DO UPDATE \
             SET longitude = EXCLUDED.longitude, sign = EXCLUDED.sign, \
                 degree = EXCLUDED.degree, retrograde = EXCLUDED.retrograde",
        )
        .bind(ticker)
        .bind(pos.planet.name())
        .bind(pos.longitude)
        .bind(pos.sign)
        .bind(pos.degree)
        .bind(pos.retrograde)
        .execute(pool)
        .await;
    }
}

// ---------------------------------------------------------------------------
// Watchlist-first SQL helper
// ---------------------------------------------------------------------------

/// Builds the `ORDER BY CASE WHEN ticker IN (...) THEN 0 ELSE 1 END, ticker`
/// fragment used by enrichment queries to prioritise watchlist tickers.
pub fn watchlist_priority_sql() -> String {
    let list = crate::WATCHLIST
        .iter()
        .map(|t| format!("'{t}'"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("CASE WHEN ticker IN ({list}) THEN 0 ELSE 1 END, ticker")
}
