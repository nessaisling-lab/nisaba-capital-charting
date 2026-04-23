//! Strategy Builder — user-defined condition chains for buy/sell signals.
//!
//! Users compose rules like:
//!   IF [Astro Score > 75] AND [RSI < 70] THEN [Buy Signal]
//!   IF [Astro Score < 30] OR [RSI > 80] THEN [Sell Signal]
//!
//! Strategies can be backtested via the v2.8 backtest engine.

use chrono::NaiveDate;

/// A single condition in a strategy rule.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // MacdCrossUp/Down awaiting MACD indicator wiring
pub enum Condition {
    AstroAbove(f64),
    AstroBelow(f64),
    RsiAbove(f32),
    RsiBelow(f32),
    MacdCrossUp,
    MacdCrossDown,
    PriceAboveSma50,
    PriceBelowSma50,
}

impl Condition {
    #[allow(dead_code)] // Used by future condition picker UI
    pub fn all_options() -> &'static [(&'static str, fn(f64) -> Condition)] {
        &[
            ("Astro >", |v| Condition::AstroAbove(v)),
            ("Astro <", |v| Condition::AstroBelow(v)),
            ("RSI >",   |v| Condition::RsiAbove(v as f32)),
            ("RSI <",   |v| Condition::RsiBelow(v as f32)),
        ]
    }

    pub fn label(&self) -> String {
        match self {
            Self::AstroAbove(v) => format!("Astro > {v:.0}"),
            Self::AstroBelow(v) => format!("Astro < {v:.0}"),
            Self::RsiAbove(v)   => format!("RSI > {v:.0}"),
            Self::RsiBelow(v)   => format!("RSI < {v:.0}"),
            Self::MacdCrossUp   => "MACD Cross Up".to_string(),
            Self::MacdCrossDown => "MACD Cross Down".to_string(),
            Self::PriceAboveSma50 => "Price > SMA50".to_string(),
            Self::PriceBelowSma50 => "Price < SMA50".to_string(),
        }
    }
}

/// How multiple conditions combine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Logic {
    And,
    Or,
}

impl Logic {
    pub fn label(self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
        }
    }
}

/// A complete strategy with buy and sell rules.
#[derive(Debug, Clone)]
#[allow(dead_code)] // `name` used by future strategy save/load
pub struct Strategy {
    pub name: String,
    pub buy_conditions: Vec<Condition>,
    pub buy_logic: Logic,
    pub sell_conditions: Vec<Condition>,
    pub sell_logic: Logic,
}

impl Default for Strategy {
    fn default() -> Self {
        Self {
            name: "Astro Momentum".to_string(),
            buy_conditions: vec![Condition::AstroAbove(70.0), Condition::RsiBelow(70.0)],
            buy_logic: Logic::And,
            sell_conditions: vec![Condition::AstroBelow(35.0)],
            sell_logic: Logic::And,
        }
    }
}

/// A snapshot of indicator values for one day, used for strategy evaluation.
#[derive(Debug, Clone)]
pub struct DaySnapshot {
    pub date: NaiveDate,
    pub close: f64,
    pub astro_score: Option<f64>,
    pub rsi: Option<f32>,
    pub macd: Option<f32>,
    pub macd_prev: Option<f32>,
    pub sma50: Option<f32>,
}

impl Strategy {
    /// Evaluate whether the buy conditions are met for a given day.
    pub fn should_buy(&self, day: &DaySnapshot) -> bool {
        Self::evaluate(&self.buy_conditions, self.buy_logic, day)
    }

    /// Evaluate whether the sell conditions are met for a given day.
    pub fn should_sell(&self, day: &DaySnapshot) -> bool {
        Self::evaluate(&self.sell_conditions, self.sell_logic, day)
    }

    fn evaluate(conditions: &[Condition], logic: Logic, day: &DaySnapshot) -> bool {
        if conditions.is_empty() {
            return false;
        }
        let results: Vec<bool> = conditions.iter().map(|c| Self::check(c, day)).collect();
        match logic {
            Logic::And => results.iter().all(|&r| r),
            Logic::Or => results.iter().any(|&r| r),
        }
    }

    fn check(cond: &Condition, day: &DaySnapshot) -> bool {
        match cond {
            Condition::AstroAbove(threshold) => day.astro_score.map(|s| s >= *threshold).unwrap_or(false),
            Condition::AstroBelow(threshold) => day.astro_score.map(|s| s <= *threshold).unwrap_or(false),
            Condition::RsiAbove(threshold) => day.rsi.map(|r| r >= *threshold).unwrap_or(false),
            Condition::RsiBelow(threshold) => day.rsi.map(|r| r <= *threshold).unwrap_or(false),
            Condition::MacdCrossUp => {
                match (day.macd, day.macd_prev) {
                    (Some(curr), Some(prev)) => prev < 0.0 && curr >= 0.0,
                    _ => false,
                }
            }
            Condition::MacdCrossDown => {
                match (day.macd, day.macd_prev) {
                    (Some(curr), Some(prev)) => prev >= 0.0 && curr < 0.0,
                    _ => false,
                }
            }
            Condition::PriceAboveSma50 => day.sma50.map(|s| day.close > s as f64).unwrap_or(false),
            Condition::PriceBelowSma50 => day.sma50.map(|s| day.close < s as f64).unwrap_or(false),
        }
    }
}

/// Run a strategy-based backtest using DaySnapshots.
pub fn run_strategy_backtest(
    ticker: &str,
    data: &[DaySnapshot],
    strategy: &Strategy,
    initial_capital: f64,
) -> crate::backtest::BacktestResult {
    use crate::backtest::{BacktestResult, Trade};

    if data.len() < 30 {
        return BacktestResult {
            ticker: ticker.to_string(),
            trades: vec![],
            total_return_pct: 0.0,
            buy_hold_return_pct: 0.0,
            max_drawdown_pct: 0.0,
            win_rate_pct: 0.0,
            signal_accuracy_pct: 0.0,
            num_trades: 0,
            days_tested: data.len(),
            final_capital: initial_capital,
            insufficient_data: Some(format!(
                "Need 30+ days of data to backtest. Currently have {} day(s) for {}.",
                data.len(), ticker,
            )),
        };
    }

    let mut capital = initial_capital;
    let mut shares = 0.0_f64;
    let mut buy_price = 0.0;
    let mut buy_date = data[0].date;
    let mut in_position = false;
    let mut trades: Vec<Trade> = vec![];
    let mut peak = capital;
    let mut max_dd = 0.0_f64;

    for day in data {
        if !in_position && strategy.should_buy(day) {
            shares = capital / day.close;
            buy_price = day.close;
            buy_date = day.date;
            in_position = true;
        } else if in_position && strategy.should_sell(day) {
            let sell_val = shares * day.close;
            trades.push(Trade {
                buy_date,
                buy_price,
                sell_date: day.date,
                sell_price: day.close,
                return_pct: (day.close - buy_price) / buy_price * 100.0,
            });
            capital = sell_val;
            shares = 0.0;
            in_position = false;
        }

        let val = if in_position { shares * day.close } else { capital };
        if val > peak { peak = val; }
        let dd = (peak - val) / peak * 100.0;
        if dd > max_dd { max_dd = dd; }
    }

    // Close open position
    let Some(last) = data.last() else {
        return BacktestResult {
            ticker: ticker.to_string(), trades, total_return_pct: 0.0,
            buy_hold_return_pct: 0.0, max_drawdown_pct: 0.0, win_rate_pct: 0.0,
            signal_accuracy_pct: 0.0, num_trades: 0, days_tested: 0,
            final_capital: initial_capital,
            insufficient_data: Some("No data available".to_string()),
        };
    };
    if in_position {
        trades.push(Trade {
            buy_date,
            buy_price,
            sell_date: last.date,
            sell_price: last.close,
            return_pct: (last.close - buy_price) / buy_price * 100.0,
        });
        capital = shares * last.close;
    }

    let bh_ret = (last.close - data[0].close) / data[0].close * 100.0;
    let total_ret = (capital - initial_capital) / initial_capital * 100.0;
    let wins = trades.iter().filter(|t| t.return_pct > 0.0).count();
    let num_trades = trades.len();
    let win_rate = if num_trades == 0 { 0.0 } else { wins as f64 / num_trades as f64 * 100.0 };

    BacktestResult {
        ticker: ticker.to_string(),
        trades,
        total_return_pct: total_ret,
        buy_hold_return_pct: bh_ret,
        max_drawdown_pct: max_dd,
        win_rate_pct: win_rate,
        signal_accuracy_pct: 0.0, // Not applicable for strategy backtests
        num_trades,
        days_tested: data.len(),
        final_capital: capital,
        insufficient_data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn snap(y: i32, m: u32, d: u32, close: f64, astro: f64, rsi: f32) -> DaySnapshot {
        DaySnapshot {
            date: NaiveDate::from_ymd_opt(y, m, d).unwrap(),
            close,
            astro_score: Some(astro),
            rsi: Some(rsi),
            macd: None,
            macd_prev: None,
            sma50: None,
        }
    }

    #[test]
    fn test_strategy_and_logic() {
        let strategy = Strategy {
            name: "Test".to_string(),
            buy_conditions: vec![Condition::AstroAbove(70.0), Condition::RsiBelow(70.0)],
            buy_logic: Logic::And,
            sell_conditions: vec![Condition::AstroBelow(30.0)],
            sell_logic: Logic::And,
        };

        // Both conditions met
        let day = snap(2025, 1, 1, 100.0, 80.0, 50.0);
        assert!(strategy.should_buy(&day));

        // Astro met, RSI not met (RSI too high)
        let day2 = snap(2025, 1, 2, 100.0, 80.0, 75.0);
        assert!(!strategy.should_buy(&day2));

        // Sell condition met
        let day3 = snap(2025, 1, 3, 100.0, 25.0, 50.0);
        assert!(strategy.should_sell(&day3));
    }

    #[test]
    fn test_strategy_or_logic() {
        let strategy = Strategy {
            name: "Loose".to_string(),
            buy_conditions: vec![Condition::AstroAbove(80.0), Condition::RsiBelow(30.0)],
            buy_logic: Logic::Or,
            sell_conditions: vec![Condition::AstroBelow(20.0)],
            sell_logic: Logic::And,
        };

        // Only RSI condition met
        let day = snap(2025, 1, 1, 100.0, 50.0, 25.0);
        assert!(strategy.should_buy(&day)); // OR logic: one is enough
    }
}
