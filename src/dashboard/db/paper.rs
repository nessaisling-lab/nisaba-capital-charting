//! Paper trading DB queries — account summary, positions with P&L, trades, daily values.

use sqlx::PgPool;
use std::sync::Arc;

use crate::error::SqlResultExt;
use pursuit_week4_automation::models::PaperTrade;

// ---------------------------------------------------------------------------
// Paper Account Summary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PaperAccountSummary {
    pub initial_capital: rust_decimal::Decimal,
    pub cash_balance:    rust_decimal::Decimal,
    pub last_sim_date:   Option<chrono::NaiveDate>,
    pub total_trades:    Option<i64>,
    pub portfolio_value: Option<rust_decimal::Decimal>,
}

pub async fn fetch_paper_account(
    pool: Arc<PgPool>,
) -> Result<Option<PaperAccountSummary>, String> {
    sqlx::query_as::<_, PaperAccountSummary>(
        "SELECT pa.initial_capital,
                pa.cash_balance,
                pa.last_sim_date,
                (SELECT COUNT(*) FROM paper_trades) AS total_trades,
                (SELECT SUM(pp.shares * pd.close)
                 FROM paper_portfolio pp
                 JOIN LATERAL (
                     SELECT close FROM price_data
                     WHERE ticker = pp.ticker ORDER BY date DESC LIMIT 1
                 ) pd ON true
                ) AS portfolio_value
         FROM paper_account pa
         LIMIT 1",
    )
    .fetch_optional(pool.as_ref())
    .await
    .ctx("fetch_paper_account")
}

// ---------------------------------------------------------------------------
// Paper Positions with current market value
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PaperPositionRow {
    pub ticker:      String,
    pub shares:      rust_decimal::Decimal,
    pub entry_price: rust_decimal::Decimal,
    pub entry_date:  chrono::NaiveDate,
    pub entry_score: Option<f32>,
    pub last_close:  Option<rust_decimal::Decimal>,
    pub last_score:  Option<f32>,
}

pub async fn fetch_paper_positions(
    pool: Arc<PgPool>,
) -> Result<Vec<PaperPositionRow>, String> {
    sqlx::query_as::<_, PaperPositionRow>(
        "SELECT pp.ticker, pp.shares, pp.entry_price, pp.entry_date, pp.entry_score,
                pd.close AS last_close,
                lh.score AS last_score
         FROM paper_portfolio pp
         LEFT JOIN LATERAL (
             SELECT close FROM price_data
             WHERE ticker = pp.ticker ORDER BY date DESC LIMIT 1
         ) pd ON true
         LEFT JOIN LATERAL (
             SELECT score FROM lagrange_history
             WHERE ticker = pp.ticker ORDER BY score_date DESC LIMIT 1
         ) lh ON true
         ORDER BY pp.ticker",
    )
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_paper_positions")
}

// ---------------------------------------------------------------------------
// Paper Trades (full log, most recent first)
// ---------------------------------------------------------------------------

pub async fn fetch_paper_trades(
    pool: Arc<PgPool>,
) -> Result<Vec<PaperTrade>, String> {
    sqlx::query_as::<_, PaperTrade>(
        "SELECT id, ticker, action, shares, price, score, trade_date, created_at
         FROM paper_trades
         ORDER BY trade_date DESC, id DESC
         LIMIT 200",
    )
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_paper_trades")
}

// ---------------------------------------------------------------------------
// Daily portfolio values (for Sharpe ratio + performance chart)
// ---------------------------------------------------------------------------

/// Returns (paper_values, spy_values) as parallel f64 vectors.
/// Each entry is the total portfolio value (cash + positions) for one sim day.
pub async fn fetch_paper_daily_values(
    pool: Arc<PgPool>,
) -> Result<(Vec<f64>, Vec<f64>), String> {
    // Reconstruct daily values from trade log + account snapshots.
    // For now, use a simplified approach: compute current-day value only
    // and accumulate as paper_trades grow over time.
    //
    // Full daily reconstruction requires a paper_snapshots table (future).
    // For v6.0, we return the trade-date series of portfolio snapshots.

    let rows: Vec<(f64,)> = sqlx::query_as(
        "SELECT DISTINCT ON (pt.trade_date)
                COALESCE(
                    (SELECT SUM(pp2.shares * pd2.close)
                     FROM paper_portfolio pp2
                     JOIN LATERAL (
                         SELECT close FROM price_data
                         WHERE ticker = pp2.ticker
                           AND date <= pt.trade_date
                         ORDER BY date DESC LIMIT 1
                     ) pd2 ON true),
                    0
                )::FLOAT8 + pa.cash_balance::FLOAT8 AS total_value
         FROM paper_trades pt
         CROSS JOIN paper_account pa
         ORDER BY pt.trade_date DESC",
    )
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    let paper_values: Vec<f64> = rows.into_iter().rev().map(|r| r.0).collect();

    // SPY benchmark: cumulative price series for the same date range
    let spy_rows: Vec<(f64,)> = sqlx::query_as(
        "SELECT close::FLOAT8 FROM price_data
         WHERE ticker = 'SPY'
         ORDER BY date DESC
         LIMIT $1",
    )
    .bind(paper_values.len().max(1) as i64)
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    let spy_values: Vec<f64> = spy_rows.into_iter().rev().map(|r| r.0).collect();

    Ok((paper_values, spy_values))
}
