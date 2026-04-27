//! Paper Trading Engine — Phase 5 of the daily scraper pipeline.
//!
//! Runs after Lagrange scoring completes. Evaluates all positions and
//! candidates using the Strategy engine with LagrangeAbove/LagrangeBelow
//! conditions, executes simulated BUY/SELL trades, and snapshots the
//! portfolio value.
//!
//! Key design decisions (from eng review):
//!   - Idempotency via `last_sim_date` on paper_account
//!   - Equal-weight sizing of available cash only (drift accepted)
//!   - Zombie detection: force sell after 10 stale trading days
//!   - Batch queries via LATERAL JOIN (no N+1)
//!   - Max 20 simultaneous positions

use chrono::{Datelike, NaiveDate, Weekday};
use std::sync::Arc;

/// Paper engine configuration — matches design doc provisional defaults.
/// Stored as constants; the design allows future migration to the settings table.
const BUY_THRESHOLD: f64 = 75.0;
const SELL_THRESHOLD: f64 = 40.0;
const MAX_POSITIONS: usize = 20;
const MAX_CANDIDATES: usize = 10;
const STALE_DAYS_LIMIT: i64 = 10;
/// If a position's value exceeds (target_weight * (1 + REBALANCE_DRIFT)) it gets trimmed.
/// 0.25 = 25% drift tolerance before rebalancing kicks in.
const REBALANCE_DRIFT: f64 = 0.25;
/// Hard stop-loss: sell if current price drops this fraction below entry price.
/// 0.15 = -15% max loss from cost basis.
const HARD_STOP_PCT: f64 = 0.15;
/// Trailing stop: sell if current price drops this fraction below peak close since entry.
/// 0.20 = -20% from the highest price the position has reached.
const TRAILING_STOP_PCT: f64 = 0.20;

/// A candidate ticker with its latest Lagrange score and price.
#[derive(Debug)]
struct Candidate {
    ticker: String,
    score: f64,
    close: f64,
}

/// An existing position's latest score and price for evaluation.
#[derive(Debug)]
struct PositionEval {
    latest_score: Option<f64>,
    latest_close: Option<f64>,
    days_since_score: Option<i64>,
    /// Highest closing price since position entry (for trailing stop).
    peak_close: Option<f64>,
}

/// Collect tickers that the paper engine needs fresh data for.
///
/// Returns a deduplicated list of:
///   1. All tickers in `paper_portfolio` (open positions — need daily prices for
///      stop-loss evaluation and P&L tracking)
///   2. Top Lagrange candidates above BUY_THRESHOLD (tickers the engine may buy
///      next — need current prices for sizing)
///
/// Called before Phase 2 of the pipeline so these tickers are included in
/// priority price fetching, sentiment, Finnhub, and other targeted data.
pub async fn collect_priority_tickers(pool: &sqlx::PgPool) -> Vec<String> {
    // 1. Open positions
    let held: Vec<String> = sqlx::query_scalar(
        "SELECT ticker FROM paper_portfolio",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // 2. Top Lagrange candidates (not already held)
    let candidates: Vec<String> = sqlx::query_scalar(
        "SELECT lh.ticker \
         FROM ( \
             SELECT DISTINCT ON (ticker) ticker, score \
             FROM lagrange_history \
             ORDER BY ticker, score_date DESC \
         ) lh \
         WHERE lh.score >= $1 \
           AND lh.ticker NOT IN (SELECT ticker FROM paper_portfolio) \
         ORDER BY lh.score DESC \
         LIMIT $2",
    )
    .bind(BUY_THRESHOLD)
    .bind(MAX_CANDIDATES as i32)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Dedup (positions first, then candidates)
    let mut result = held;
    for t in candidates {
        if !result.contains(&t) {
            result.push(t);
        }
    }
    result
}

/// Run the daily paper trading simulation. Called as Phase 5 of run_all_fetches().
pub async fn run_simulation(pool: Arc<sqlx::PgPool>) {
    println!("5.1 Checking paper account...");

    // ---- Load or auto-initialize account ----
    let account = match sqlx::query_as::<_, (i32, rust_decimal::Decimal, Option<chrono::NaiveDate>)>(
        "SELECT id, cash_balance, last_sim_date FROM paper_account LIMIT 1",
    )
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            println!("[Paper] No account found. Auto-initializing with $100,000...");
            match sqlx::query_as::<_, (i32, rust_decimal::Decimal, Option<chrono::NaiveDate>)>(
                "INSERT INTO paper_account (initial_capital, cash_balance) \
                 VALUES (100000, 100000) \
                 RETURNING id, cash_balance, last_sim_date",
            )
            .fetch_one(pool.as_ref())
            .await
            {
                Ok(row) => row,
                Err(e) => {
                    eprintln!("[Paper] Failed to initialize account: {e}");
                    return;
                }
            }
        }
        Err(e) => {
            eprintln!("[Paper] DB error loading account: {e}");
            return;
        }
    };

    let (account_id, cash_balance, last_sim_date) = account;
    let cash: f64 = cash_balance.to_string().parse().unwrap_or(0.0);

    // ---- Idempotency guard: skip if already simulated today ----
    let today = chrono::Utc::now().date_naive();
    // Use the most recent trading day (the date of the latest price data)
    let sim_date: chrono::NaiveDate = sqlx::query_scalar(
        "SELECT MAX(date) FROM price_data",
    )
    .fetch_optional(pool.as_ref())
    .await
    .ok()
    .flatten()
    .unwrap_or(today);

    if last_sim_date == Some(sim_date) {
        println!("[Paper] Already simulated for {sim_date}. Skipping (idempotency guard).");
        return;
    }

    // Weekend guard
    let dow = sim_date.weekday();
    if dow == Weekday::Sat || dow == Weekday::Sun {
        println!("[Paper] {sim_date} is a weekend ({dow}). Skipping.");
        return;
    }

    // NYSE holiday guard
    if is_nyse_holiday(sim_date) {
        println!("[Paper] {sim_date} is an NYSE holiday. Skipping.");
        return;
    }

    println!("[Paper] Simulating for trade date: {sim_date}");
    println!("[Paper] Cash available: ${cash:.2}");

    // ---- Load current positions ----
    let positions: Vec<(String, f64, f64, NaiveDate)> = sqlx::query_as(
        "SELECT ticker, shares::FLOAT8, entry_price::FLOAT8, entry_date FROM paper_portfolio",
    )
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    println!("[Paper] Current positions: {}", positions.len());

    // ---- Evaluate existing positions (batch: latest score + price per position) ----
    let mut sells: Vec<(String, f64, f64, Option<f32>)> = vec![]; // (ticker, shares, price, score)

    for (ticker, shares, entry_price, entry_date) in &positions {
        let eval = evaluate_position(pool.as_ref(), ticker, sim_date, *entry_date).await;
        match eval {
            Some(pe) => {
                let price = pe.latest_close.unwrap_or(*entry_price);
                let score_f32 = pe.latest_score.map(|s| s as f32);
                let should_sell = pe.latest_score.map(|s| s <= SELL_THRESHOLD).unwrap_or(false);
                let is_zombie = pe.days_since_score.map(|d| d >= STALE_DAYS_LIMIT).unwrap_or(false);

                // Hard stop-loss: price dropped too far from entry
                let hard_stop_price = entry_price * (1.0 - HARD_STOP_PCT);
                let hard_stop_hit = price < hard_stop_price;

                // Trailing stop: price dropped too far from peak
                let trailing_stop_hit = pe.peak_close.map(|peak| {
                    let trail_price = peak * (1.0 - TRAILING_STOP_PCT);
                    price < trail_price
                }).unwrap_or(false);

                if hard_stop_hit {
                    let loss_pct = (price - entry_price) / entry_price * 100.0;
                    println!("  SELL {ticker}: HARD STOP hit ({loss_pct:+.1}% from entry ${entry_price:.2})");
                    sells.push((ticker.clone(), *shares, price, score_f32));
                } else if trailing_stop_hit {
                    let peak = pe.peak_close.unwrap_or(*entry_price);
                    let drop_pct = (price - peak) / peak * 100.0;
                    println!("  SELL {ticker}: TRAILING STOP hit ({drop_pct:+.1}% from peak ${peak:.2})");
                    sells.push((ticker.clone(), *shares, price, score_f32));
                } else if should_sell {
                    println!("  SELL {ticker}: score {:.1} <= {SELL_THRESHOLD} (threshold)", pe.latest_score.unwrap_or(0.0));
                    sells.push((ticker.clone(), *shares, price, score_f32));
                } else if is_zombie {
                    let days = pe.days_since_score.unwrap_or(0);
                    println!("  SELL {ticker}: zombie position ({days} days without score update)");
                    sells.push((ticker.clone(), *shares, price, score_f32));
                }
            }
            None => {
                // No data at all for this ticker — treat as zombie
                println!("  SELL {ticker}: no price/score data found (zombie)");
                sells.push((ticker.clone(), *shares, *entry_price, None));
            }
        }
    }

    // ---- Execute sells ----
    let mut updated_cash = cash;
    for (ticker, shares, price, score) in &sells {
        let proceeds = shares * price;
        updated_cash += proceeds;

        if let Err(e) = execute_sell(pool.as_ref(), ticker, *shares, *price, *score, sim_date).await {
            eprintln!("[Paper] Failed to execute SELL {ticker}: {e}");
            continue;
        }
        println!("  -> Sold {shares:.2} shares of {ticker} @ ${price:.2} = ${proceeds:.2}");
    }

    // ---- Find buy candidates (Lagrange > BUY_THRESHOLD, not already held) ----
    let current_position_count = positions.len() - sells.len();
    let open_slots = MAX_POSITIONS.saturating_sub(current_position_count);

    if open_slots == 0 {
        println!("[Paper] Max positions ({MAX_POSITIONS}) reached. No new buys.");
    } else {
        let candidates = find_candidates(pool.as_ref(), BUY_THRESHOLD).await;
        let to_buy = candidates.into_iter().take(open_slots.min(MAX_CANDIDATES)).collect::<Vec<_>>();

        if to_buy.is_empty() {
            println!("[Paper] No qualifying candidates (Lagrange > {BUY_THRESHOLD}).");
        } else {
            // Equal-weight sizing across qualifying tickers
            let allocation_per = updated_cash / to_buy.len() as f64;

            for candidate in &to_buy {
                if allocation_per < 1.0 {
                    println!("[Paper] Insufficient cash for position in {}. Skipping.", candidate.ticker);
                    continue;
                }

                let shares = allocation_per / candidate.close;
                let cost = shares * candidate.close;

                if cost > updated_cash {
                    println!("[Paper] Insufficient cash for {}. Need ${cost:.2}, have ${updated_cash:.2}.", candidate.ticker);
                    continue;
                }

                if let Err(e) = execute_buy(
                    pool.as_ref(), &candidate.ticker, shares, candidate.close,
                    candidate.score as f32, sim_date,
                ).await {
                    eprintln!("[Paper] Failed to execute BUY {}: {e}", candidate.ticker);
                    continue;
                }

                updated_cash -= cost;
                println!("  BUY {}: {shares:.2} shares @ ${:.2} (score: {:.1})",
                    candidate.ticker, candidate.close, candidate.score);
            }
        }
    }

    // ---- Rebalance: trim positions that drifted too far from equal-weight ----
    updated_cash = rebalance_positions(pool.as_ref(), updated_cash, sim_date).await;

    // ---- Update account cash + sim date ----
    let cash_decimal = rust_decimal::Decimal::from_f64_retain(updated_cash)
        .unwrap_or(rust_decimal::Decimal::ZERO);

    if let Err(e) = sqlx::query(
        "UPDATE paper_account SET cash_balance = $1, last_sim_date = $2 WHERE id = $3",
    )
    .bind(cash_decimal)
    .bind(sim_date)
    .bind(account_id)
    .execute(pool.as_ref())
    .await
    {
        eprintln!("[Paper] Failed to update account: {e}");
    }

    // ---- Summary ----
    let final_positions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM paper_portfolio")
        .fetch_one(pool.as_ref())
        .await
        .unwrap_or(0);

    println!("[Paper] Simulation complete for {sim_date}.");
    println!("[Paper] Sells: {}, Buys: {}", sells.len(),
        positions.len() - sells.len() + final_positions as usize - current_position_count);
    println!("[Paper] Positions: {final_positions}, Cash: ${updated_cash:.2}");

    crate::log_fetch(pool.as_ref(), "paper_engine", None, "simulation", "ok", None).await;
}

/// Evaluate a single position: get its latest score, price, staleness, and peak price.
async fn evaluate_position(
    pool: &sqlx::PgPool,
    ticker: &str,
    sim_date: NaiveDate,
    entry_date: NaiveDate,
) -> Option<PositionEval> {
    // Latest Lagrange score for this ticker
    let score_row: Option<(f64, NaiveDate)> = sqlx::query_as(
        "SELECT score::FLOAT8, score_date \
         FROM lagrange_history \
         WHERE ticker = $1 \
         ORDER BY score_date DESC LIMIT 1",
    )
    .bind(ticker)
    .fetch_optional(pool)
    .await
    .ok()?;

    // Latest price for this ticker
    let price_row: Option<(f64,)> = sqlx::query_as(
        "SELECT close::FLOAT8 FROM price_data \
         WHERE ticker = $1 \
         ORDER BY date DESC LIMIT 1",
    )
    .bind(ticker)
    .fetch_optional(pool)
    .await
    .ok()?;

    // Peak close since entry (for trailing stop)
    let peak_row: Option<(f64,)> = sqlx::query_as(
        "SELECT MAX(close)::FLOAT8 FROM price_data \
         WHERE ticker = $1 AND date >= $2",
    )
    .bind(ticker)
    .bind(entry_date)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (score, score_date) = score_row.unzip();
    let days_since = score_date.map(|sd| (sim_date - sd).num_days());

    Some(PositionEval {
        latest_score: score,
        latest_close: price_row.map(|r| r.0),
        days_since_score: days_since,
        peak_close: peak_row.and_then(|r| if r.0 > 0.0 { Some(r.0) } else { None }),
    })
}

/// Find buy candidates: tickers with Lagrange > threshold, not already held,
/// ordered by score descending.
async fn find_candidates(
    pool: &sqlx::PgPool,
    threshold: f64,
) -> Vec<Candidate> {
    // Build the held-ticker exclusion list for the query
    // Uses a subquery approach to avoid dynamic SQL
    let rows: Vec<(String, f64, f64)> = sqlx::query_as(
        "SELECT lh.ticker, lh.score::FLOAT8, pd.close::FLOAT8 \
         FROM ( \
             SELECT DISTINCT ON (ticker) ticker, score \
             FROM lagrange_history \
             ORDER BY ticker, score_date DESC \
         ) lh \
         JOIN LATERAL ( \
             SELECT close FROM price_data \
             WHERE ticker = lh.ticker \
             ORDER BY date DESC LIMIT 1 \
         ) pd ON true \
         WHERE lh.score >= $1 \
           AND lh.ticker NOT IN (SELECT ticker FROM paper_portfolio) \
         ORDER BY lh.score DESC \
         LIMIT $2",
    )
    .bind(threshold)
    .bind(MAX_CANDIDATES as i32)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|(ticker, score, close)| Candidate { ticker, score, close })
        .collect()
}

/// Execute a BUY: insert into paper_portfolio + paper_trades, no account update here.
async fn execute_buy(
    pool: &sqlx::PgPool,
    ticker: &str,
    shares: f64,
    price: f64,
    score: f32,
    trade_date: chrono::NaiveDate,
) -> Result<(), String> {
    // Insert position (ON CONFLICT: add to existing position)
    sqlx::query(
        "INSERT INTO paper_portfolio (ticker, shares, entry_price, entry_date, entry_score) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (ticker) DO UPDATE SET \
             shares = paper_portfolio.shares + EXCLUDED.shares, \
             entry_price = (paper_portfolio.entry_price * paper_portfolio.shares + EXCLUDED.entry_price * EXCLUDED.shares) \
                           / (paper_portfolio.shares + EXCLUDED.shares), \
             entry_score = EXCLUDED.entry_score",
    )
    .bind(ticker)
    .bind(rust_decimal::Decimal::from_f64_retain(shares).unwrap_or_default())
    .bind(rust_decimal::Decimal::from_f64_retain(price).unwrap_or_default())
    .bind(trade_date)
    .bind(score)
    .execute(pool)
    .await
    .map_err(|e| format!("portfolio insert: {e}"))?;

    // Log trade
    sqlx::query(
        "INSERT INTO paper_trades (ticker, action, shares, price, score, trade_date) \
         VALUES ($1, 'BUY', $2, $3, $4, $5)",
    )
    .bind(ticker)
    .bind(rust_decimal::Decimal::from_f64_retain(shares).unwrap_or_default())
    .bind(rust_decimal::Decimal::from_f64_retain(price).unwrap_or_default())
    .bind(score)
    .bind(trade_date)
    .execute(pool)
    .await
    .map_err(|e| format!("trade log: {e}"))?;

    Ok(())
}

/// Execute a SELL: remove from paper_portfolio + log to paper_trades.
async fn execute_sell(
    pool: &sqlx::PgPool,
    ticker: &str,
    shares: f64,
    price: f64,
    score: Option<f32>,
    trade_date: chrono::NaiveDate,
) -> Result<(), String> {
    // Remove position
    sqlx::query("DELETE FROM paper_portfolio WHERE ticker = $1")
        .bind(ticker)
        .execute(pool)
        .await
        .map_err(|e| format!("portfolio delete: {e}"))?;

    // Log trade
    sqlx::query(
        "INSERT INTO paper_trades (ticker, action, shares, price, score, trade_date) \
         VALUES ($1, 'SELL', $2, $3, $4, $5)",
    )
    .bind(ticker)
    .bind(rust_decimal::Decimal::from_f64_retain(shares).unwrap_or_default())
    .bind(rust_decimal::Decimal::from_f64_retain(price).unwrap_or_default())
    .bind(score)
    .bind(trade_date)
    .execute(pool)
    .await
    .map_err(|e| format!("trade log: {e}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Equal-weight rebalancing — trim positions that drifted past threshold
// ---------------------------------------------------------------------------

/// After signal-driven buys/sells, check if any position's market value exceeds
/// the equal-weight target by more than REBALANCE_DRIFT. If so, trim it back to
/// target by selling the excess shares. Returns updated cash balance.
async fn rebalance_positions(
    pool: &sqlx::PgPool,
    mut cash: f64,
    sim_date: NaiveDate,
) -> f64 {
    // Load all current positions with their latest prices
    let positions: Vec<(String, f64, f64)> = sqlx::query_as(
        "SELECT pp.ticker, pp.shares::FLOAT8, \
                COALESCE(pd.close::FLOAT8, pp.entry_price::FLOAT8) \
         FROM paper_portfolio pp \
         LEFT JOIN LATERAL ( \
             SELECT close FROM price_data \
             WHERE ticker = pp.ticker ORDER BY date DESC LIMIT 1 \
         ) pd ON true",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let n = positions.len();
    if n < 2 {
        return cash; // Nothing to rebalance with 0 or 1 positions
    }

    // Total portfolio value (positions + cash)
    let positions_value: f64 = positions.iter().map(|(_, shares, price)| shares * price).sum();
    let total_value = positions_value + cash;
    let target_per_position = total_value / n as f64;
    let drift_ceiling = target_per_position * (1.0 + REBALANCE_DRIFT);

    let mut rebal_count = 0;

    for (ticker, shares, price) in &positions {
        let current_value = shares * price;

        if current_value > drift_ceiling && *price > 0.0 {
            // Trim to target weight
            let target_shares = target_per_position / price;
            let excess_shares = shares - target_shares;

            if excess_shares < 0.01 {
                continue; // Negligible drift
            }

            let proceeds = excess_shares * price;

            // Partial sell: reduce position shares (not full delete)
            let new_shares = rust_decimal::Decimal::from_f64_retain(target_shares)
                .unwrap_or_default();

            if let Err(e) = sqlx::query(
                "UPDATE paper_portfolio SET shares = $1 WHERE ticker = $2",
            )
            .bind(new_shares)
            .bind(ticker)
            .execute(pool)
            .await
            {
                eprintln!("[Paper] Rebalance failed for {ticker}: {e}");
                continue;
            }

            // Log as SELL trade (score = None indicates rebalance, not signal)
            if let Err(e) = sqlx::query(
                "INSERT INTO paper_trades (ticker, action, shares, price, score, trade_date) \
                 VALUES ($1, 'SELL', $2, $3, NULL, $4)",
            )
            .bind(ticker)
            .bind(rust_decimal::Decimal::from_f64_retain(excess_shares).unwrap_or_default())
            .bind(rust_decimal::Decimal::from_f64_retain(*price).unwrap_or_default())
            .bind(sim_date)
            .execute(pool)
            .await
            {
                eprintln!("[Paper] Rebalance trade log failed for {ticker}: {e}");
                continue;
            }

            cash += proceeds;
            rebal_count += 1;
            println!("  REBAL {ticker}: trimmed {excess_shares:.2} shares @ ${price:.2} (${proceeds:.2} freed)");
        }
    }

    if rebal_count > 0 {
        println!("[Paper] Rebalanced {rebal_count} position(s). Cash now ${cash:.2}");
    }

    cash
}

// ---------------------------------------------------------------------------
// NYSE Holiday Calendar
// ---------------------------------------------------------------------------

/// Returns true if the given date is an NYSE market holiday.
///
/// Covers the 9 regular NYSE holidays + observed substitution rules:
/// - If a holiday falls on Saturday, NYSE closes Friday.
/// - If a holiday falls on Sunday, NYSE closes Monday.
/// - Juneteenth added 2022+.
fn is_nyse_holiday(date: NaiveDate) -> bool {
    let y = date.year();

    // Helper: observed date for a fixed holiday (shift Sat->Fri, Sun->Mon)
    let observed = |month: u32, day: u32| -> NaiveDate {
        let raw = NaiveDate::from_ymd_opt(y, month, day).unwrap();
        match raw.weekday() {
            Weekday::Sat => raw.pred_opt().unwrap(),
            Weekday::Sun => raw.succ_opt().unwrap(),
            _ => raw,
        }
    };

    // Helper: Nth weekday of a month (1-indexed)
    let nth_weekday = |month: u32, weekday: Weekday, nth: u32| -> NaiveDate {
        let first = NaiveDate::from_ymd_opt(y, month, 1).unwrap();
        let offset = (weekday.num_days_from_monday() as i32
            - first.weekday().num_days_from_monday() as i32)
            .rem_euclid(7) as u32;
        NaiveDate::from_ymd_opt(y, month, 1 + offset + (nth - 1) * 7).unwrap()
    };

    // Helper: last Monday of a month
    let last_monday = |month: u32| -> NaiveDate {
        let next_month = if month == 12 {
            NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(y, month + 1, 1).unwrap()
        };
        let last_day = next_month.pred_opt().unwrap();
        let offset = (last_day.weekday().num_days_from_monday()) as i64;
        last_day - chrono::Duration::days(offset)
    };

    let mut holidays = vec![
        // Fixed holidays (with observed-date shifting)
        observed(1, 1),      // New Year's Day
        observed(7, 4),      // Independence Day
        observed(12, 25),    // Christmas Day

        // Floating holidays
        nth_weekday(1, Weekday::Mon, 3),  // MLK Day: 3rd Monday of January
        nth_weekday(2, Weekday::Mon, 3),  // Presidents' Day: 3rd Monday of February
        last_monday(5),                    // Memorial Day: last Monday of May
        nth_weekday(9, Weekday::Mon, 1),  // Labor Day: 1st Monday of September
        nth_weekday(11, Weekday::Thu, 4), // Thanksgiving: 4th Thursday of November
    ];

    // Juneteenth: NYSE holiday since 2022
    if y >= 2022 {
        holidays.push(observed(6, 19));
    }

    holidays.contains(&date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_holidays_2025() {
        let holidays = [
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),   // New Year's
            NaiveDate::from_ymd_opt(2025, 1, 20).unwrap(),  // MLK Day
            NaiveDate::from_ymd_opt(2025, 2, 17).unwrap(),  // Presidents' Day
            NaiveDate::from_ymd_opt(2025, 5, 26).unwrap(),  // Memorial Day
            NaiveDate::from_ymd_opt(2025, 6, 19).unwrap(),  // Juneteenth (Thursday)
            NaiveDate::from_ymd_opt(2025, 7, 4).unwrap(),   // Independence Day (Friday)
            NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),   // Labor Day
            NaiveDate::from_ymd_opt(2025, 11, 27).unwrap(), // Thanksgiving
            NaiveDate::from_ymd_opt(2025, 12, 25).unwrap(), // Christmas (Thursday)
        ];
        for h in &holidays {
            assert!(is_nyse_holiday(*h), "Expected {h} to be an NYSE holiday");
        }
    }

    #[test]
    fn test_regular_trading_day() {
        let regular = NaiveDate::from_ymd_opt(2025, 4, 15).unwrap();
        assert!(!is_nyse_holiday(regular));
    }

    #[test]
    fn test_observed_sunday_holiday() {
        // 2021-07-04 is Sunday -> observed Monday 07-05
        let observed = NaiveDate::from_ymd_opt(2021, 7, 5).unwrap();
        assert!(is_nyse_holiday(observed), "July 4 2021 (Sun) observed on Monday 07-05");
    }

    #[test]
    fn test_juneteenth_pre_2022() {
        // Juneteenth was not an NYSE holiday before 2022
        let pre = NaiveDate::from_ymd_opt(2021, 6, 18).unwrap();
        assert!(!is_nyse_holiday(pre), "Juneteenth not observed pre-2022");
    }

    #[test]
    fn test_thanksgiving_2026() {
        let thx = NaiveDate::from_ymd_opt(2026, 11, 26).unwrap();
        assert!(is_nyse_holiday(thx));
    }
}
