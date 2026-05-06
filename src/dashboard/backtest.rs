//! Astro-driven backtesting engine.
//!
//! Tests the thesis: "Does buying when the astro score is high and selling when
//! it's low outperform buy-and-hold?" Joins `astro_scores` and `price_data` by
//! (ticker, date) and simulates a long-only strategy.
//!
//! Key metric: **Astro Signal Accuracy** — what percentage of the time did a
//! favorable astro score predict a price increase within 30 days?

use chrono::{Datelike, NaiveDate};

/// User-configurable backtest parameters.
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    /// Buy when astro score rises above this level (e.g., 65.0).
    pub buy_threshold: f64,
    /// Sell when astro score drops below this level (e.g., 35.0).
    pub sell_threshold: f64,
    /// Starting capital in dollars.
    pub initial_capital: f64,
    /// Wave 9.I2 — Time window for backtest. Defaults to `All` (use every
    /// row supplied). Other variants pre-filter the data slice before
    /// running the strategy. Lets users compare "Saturn return zone" vs
    /// "all time" performance.
    pub time_window: TimeWindow,
}

/// Wave 9.I2 — Selectable backtest window.
///
/// `LastYears`, `Custom`, and `ReturnZone` variants are constructed by
/// the UI (Settings → Backtest) — wired in a follow-up. The variants are
/// public + tested so the engine can already consume them.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TimeWindow {
    /// Use every row provided.
    All,
    /// Most-recent N years from the supplied data's last date.
    LastYears(u32),
    /// Custom date range. Both inclusive.
    Custom(NaiveDate, NaiveDate),
    /// Wave 9.I2 + 9.A3 — Cycle-aligned: only include days falling within
    /// a planet's *return-zone* (±N days from each exact return).
    /// E.g. Saturn return zone = ±365 days from each Saturn return moment.
    /// Empty if the natal chart has no returns of `planet` in the dataset.
    ReturnZone {
        planet: pursuit_week4_automation::astrology::ephemeris::Planet,
        return_dates: Vec<NaiveDate>,
        zone_days: i64,
    },
}

impl Default for TimeWindow {
    fn default() -> Self { Self::All }
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            buy_threshold: 65.0,
            sell_threshold: 35.0,
            initial_capital: 10_000.0,
            time_window: TimeWindow::default(),
        }
    }
}

impl BacktestConfig {
    /// Wave 9.I2 — Apply the time_window filter to a sorted-ascending
    /// slice of days. Returns the filtered subset.
    pub fn filter_days(&self, days: &[BacktestDay]) -> Vec<BacktestDay> {
        match &self.time_window {
            TimeWindow::All => days.to_vec(),
            TimeWindow::LastYears(n) => {
                if days.is_empty() { return Vec::new(); }
                let last = days.last().unwrap().date;
                // Subtract `n` calendar years from `last` — same month/day.
                let start = NaiveDate::from_ymd_opt(
                    last.year() - *n as i32,
                    last.month(),
                    last.day(),
                ).unwrap_or(last);
                days.iter().filter(|d| d.date >= start).cloned().collect()
            }
            TimeWindow::Custom(start, end) => {
                days.iter()
                    .filter(|d| d.date >= *start && d.date <= *end)
                    .cloned()
                    .collect()
            }
            TimeWindow::ReturnZone { return_dates, zone_days, .. } => {
                days.iter()
                    .filter(|d| {
                        return_dates.iter().any(|r| {
                            (d.date - *r).num_days().abs() <= *zone_days
                        })
                    })
                    .cloned()
                    .collect()
            }
        }
    }
}

/// One day of joined astro + price data for backtesting.
#[derive(Debug, Clone)]
pub struct BacktestDay {
    pub date: NaiveDate,
    pub close: f64,
    pub astro_score: f64,
}

/// A single trade executed by the backtester.
#[derive(Debug, Clone)]
pub struct Trade {
    pub buy_date: NaiveDate,
    pub buy_price: f64,
    pub sell_date: NaiveDate,
    pub sell_price: f64,
    pub return_pct: f64,
    /// v11.0: Real-world events (news headlines, filings) during this trade window.
    pub events: Vec<String>,
}

/// Full backtest results.
#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub ticker: String,
    pub trades: Vec<Trade>,
    pub total_return_pct: f64,
    pub buy_hold_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub win_rate_pct: f64,
    pub signal_accuracy_pct: f64,
    pub num_trades: usize,
    pub days_tested: usize,
    pub final_capital: f64,
    /// Minimum-data guard: None = sufficient data, Some(msg) = insufficient.
    pub insufficient_data: Option<String>,
}

/// Run the backtest on a time-ordered series of joined astro + price data.
///
/// Strategy: go long when astro_score crosses above `buy_threshold`, exit when
/// it drops below `sell_threshold`. Fully invested while in a trade, fully cash
/// while out. No shorting.
pub fn run_backtest(
    ticker: &str,
    data: &[BacktestDay],
    config: &BacktestConfig,
) -> BacktestResult {
    // Wave 9.I2 — apply time_window filter before any min-data checks so
    // that ReturnZone / Custom modes that select sparse subsets surface
    // a clear "insufficient data" message instead of running a degenerate
    // backtest on filtered-down rows.
    let filtered = config.filter_days(data);
    let data = filtered.as_slice();
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
            final_capital: config.initial_capital,
            insufficient_data: Some(format!(
                "Need 30+ days of astro + price data to backtest. Currently have {} day(s) for {}.",
                data.len(), ticker,
            )),
        };
    }

    let mut capital = config.initial_capital;
    let mut shares = 0.0_f64;
    let mut buy_price = 0.0;
    let mut buy_date = data[0].date;
    let mut in_position = false;
    let mut trades: Vec<Trade> = vec![];
    let mut peak_capital = capital;
    let mut max_drawdown = 0.0_f64;

    for day in data {
        if !in_position && day.astro_score >= config.buy_threshold {
            // BUY signal
            shares = capital / day.close;
            buy_price = day.close;
            buy_date = day.date;
            in_position = true;
        } else if in_position && day.astro_score <= config.sell_threshold {
            // SELL signal
            let sell_value = shares * day.close;
            let ret_pct = (day.close - buy_price) / buy_price * 100.0;
            trades.push(Trade {
                buy_date,
                buy_price,
                sell_date: day.date,
                sell_price: day.close,
                return_pct: ret_pct,
                events: vec![],
            });
            capital = sell_value;
            shares = 0.0;
            in_position = false;
        }

        // Track drawdown
        let current_value = if in_position { shares * day.close } else { capital };
        if current_value > peak_capital {
            peak_capital = current_value;
        }
        let dd = (peak_capital - current_value) / peak_capital * 100.0;
        if dd > max_drawdown {
            max_drawdown = dd;
        }
    }

    // Close open position at last price
    let Some(last) = data.last() else {
        return BacktestResult {
            ticker: ticker.to_string(), trades, total_return_pct: 0.0,
            buy_hold_return_pct: 0.0, max_drawdown_pct: 0.0, win_rate_pct: 0.0,
            signal_accuracy_pct: 0.0, num_trades: 0, days_tested: 0,
            final_capital: config.initial_capital,
            insufficient_data: Some("No data available".to_string()),
        };
    };
    if in_position {
        let sell_value = shares * last.close;
        let ret_pct = (last.close - buy_price) / buy_price * 100.0;
        trades.push(Trade {
            buy_date,
            buy_price,
            sell_date: last.date,
            sell_price: last.close,
            return_pct: ret_pct,
            events: vec![],
        });
        capital = sell_value;
    }

    let first = &data[0];
    let buy_hold_ret = (last.close - first.close) / first.close * 100.0;
    let total_ret = (capital - config.initial_capital) / config.initial_capital * 100.0;
    let wins = trades.iter().filter(|t| t.return_pct > 0.0).count();
    let win_rate = if trades.is_empty() {
        0.0
    } else {
        wins as f64 / trades.len() as f64 * 100.0
    };

    // Astro Signal Accuracy: what % of days with astro > buy_threshold saw
    // price increase within 30 days?
    let signal_accuracy = compute_signal_accuracy(data, config.buy_threshold);
    let num_trades = trades.len();

    BacktestResult {
        ticker: ticker.to_string(),
        trades,
        total_return_pct: total_ret,
        buy_hold_return_pct: buy_hold_ret,
        max_drawdown_pct: max_drawdown,
        win_rate_pct: win_rate,
        signal_accuracy_pct: signal_accuracy,
        num_trades,
        days_tested: data.len(),
        final_capital: capital,
        insufficient_data: None,
    }
}

/// What percentage of favorable-astro days were followed by a price increase
/// within the next 30 trading days?
fn compute_signal_accuracy(data: &[BacktestDay], threshold: f64) -> f64 {
    let mut correct = 0u32;
    let mut total = 0u32;

    for (i, day) in data.iter().enumerate() {
        if day.astro_score >= threshold {
            total += 1;
            // Look ahead up to 30 trading days
            let lookahead = data.len().min(i + 31);
            let future_max = data[i + 1..lookahead]
                .iter()
                .map(|d| d.close)
                .fold(f64::NEG_INFINITY, f64::max);
            if future_max > day.close {
                correct += 1;
            }
        }
    }

    if total == 0 {
        0.0
    } else {
        correct as f64 / total as f64 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn make_day(y: i32, m: u32, d: u32, close: f64, astro: f64) -> BacktestDay {
        BacktestDay {
            date: NaiveDate::from_ymd_opt(y, m, d).unwrap(),
            close,
            astro_score: astro,
        }
    }

    #[test]
    fn test_insufficient_data() {
        // 5 days is below the 30-day minimum
        let data = vec![
            make_day(2025, 1, 1, 100.0, 50.0),
            make_day(2025, 1, 2, 101.0, 70.0),
            make_day(2025, 1, 3, 105.0, 60.0),
            make_day(2025, 1, 4, 110.0, 30.0),
            make_day(2025, 1, 5, 108.0, 40.0),
        ];
        let config = BacktestConfig::default();
        let result = run_backtest("TEST", &data, &config);
        assert!(result.insufficient_data.is_some());
        assert_eq!(result.trades.len(), 0);
    }

    #[test]
    fn test_basic_backtest() {
        // Generate 35 days: neutral -> buy signal -> hold -> sell signal -> neutral
        let mut data = Vec::new();
        for i in 0..10 {
            data.push(make_day(2025, 1, 1 + i, 100.0 + i as f64 * 0.2, 50.0));
        }
        // Day 10: astro goes high -> BUY
        data.push(make_day(2025, 1, 11, 101.0, 70.0));
        for i in 12..=20 {
            data.push(make_day(2025, 1, i, 101.0 + (i - 11) as f64, 60.0));
        }
        // Day ~21: astro drops -> SELL at higher price
        data.push(make_day(2025, 1, 21, 110.0, 30.0));
        for i in 22..=31 {
            data.push(make_day(2025, 1, i, 108.0, 40.0));
        }
        // Pad to > 31 days
        data.push(make_day(2025, 2, 1, 107.0, 45.0));

        let config = BacktestConfig::default();
        let result = run_backtest("TEST", &data, &config);

        assert!(result.insufficient_data.is_none());
        assert_eq!(result.trades.len(), 1);
        assert!(result.trades[0].return_pct > 0.0); // bought at ~101, sold at 110
        assert!(result.total_return_pct > 0.0);
        assert!(result.win_rate_pct > 99.0); // 1/1 = 100%
    }

    #[test]
    fn test_signal_accuracy() {
        // Days above threshold followed by price increase
        let data = vec![
            make_day(2025, 1, 1, 100.0, 70.0),
            make_day(2025, 1, 2, 105.0, 70.0),  // price went up -> accurate
            make_day(2025, 1, 3, 103.0, 30.0),
            make_day(2025, 1, 4, 100.0, 70.0),
            make_day(2025, 1, 5, 98.0, 30.0),   // price went down -> inaccurate
        ];
        let acc = compute_signal_accuracy(&data, 65.0);
        // Day 0 (score 70): next days have 105 -> accurate
        // Day 1 (score 70): next days have 103, 100, 98 -> 103 > 105? no. But 103 > 105? no. Wait:
        // Day 1 close = 105. Future: 103, 100, 98. Max future = 103 < 105. Inaccurate.
        // Day 3 (score 70): next day = 98. Max future = 98 < 100. Inaccurate.
        // Total: 1 correct out of 3 = 33.3%
        assert!(acc > 30.0 && acc < 40.0);
    }

    // ── Wave 9.I2 — TimeWindow filter tests ─────────────────────────

    fn day_seq(start_year: i32, count: usize) -> Vec<BacktestDay> {
        (0..count).map(|i| {
            let date = NaiveDate::from_ymd_opt(start_year, 1, 1).unwrap()
                + chrono::Duration::days(i as i64);
            make_day(date.year(), date.month(), date.day(), 100.0, 50.0)
        }).collect()
    }

    #[test]
    fn time_window_all_keeps_everything() {
        let data = day_seq(2020, 100);
        let cfg = BacktestConfig { time_window: TimeWindow::All, ..Default::default() };
        let filtered = cfg.filter_days(&data);
        assert_eq!(filtered.len(), 100);
    }

    #[test]
    fn time_window_last_years_filters_recent() {
        // 5 years worth of days
        let data = day_seq(2020, 5 * 365);
        let cfg = BacktestConfig {
            time_window: TimeWindow::LastYears(2),
            ..Default::default()
        };
        let filtered = cfg.filter_days(&data);
        // Last 2 calendar years from data's last date (2024-12-30) =
        // 2022-12-30 .. 2024-12-30 ≈ 731 days
        assert!(filtered.len() > 600 && filtered.len() < 800,
            "Expected ~730 days for LastYears(2), got {}", filtered.len());
        // First filtered date should be ≥ (last - 2y)
        let last = data.last().unwrap().date;
        let first = filtered.first().unwrap().date;
        assert!(first >= NaiveDate::from_ymd_opt(last.year() - 2, last.month(), last.day()).unwrap());
    }

    #[test]
    fn time_window_custom_range() {
        let data = day_seq(2020, 365);
        let cfg = BacktestConfig {
            time_window: TimeWindow::Custom(
                NaiveDate::from_ymd_opt(2020, 4, 1).unwrap(),
                NaiveDate::from_ymd_opt(2020, 6, 30).unwrap(),
            ),
            ..Default::default()
        };
        let filtered = cfg.filter_days(&data);
        // April-June 2020: 30+31+30 = 91 days
        assert_eq!(filtered.len(), 91);
    }

    #[test]
    fn time_window_return_zone() {
        let data = day_seq(2020, 365);
        // Single "return" event in the middle of the year, ±10-day zone
        let cfg = BacktestConfig {
            time_window: TimeWindow::ReturnZone {
                planet: pursuit_week4_automation::astrology::ephemeris::Planet::Saturn,
                return_dates: vec![NaiveDate::from_ymd_opt(2020, 6, 15).unwrap()],
                zone_days: 10,
            },
            ..Default::default()
        };
        let filtered = cfg.filter_days(&data);
        // ±10 days = 21-day window
        assert_eq!(filtered.len(), 21);
    }

    #[test]
    fn time_window_return_zone_multiple_returns() {
        let data = day_seq(2020, 365 * 5);
        let cfg = BacktestConfig {
            time_window: TimeWindow::ReturnZone {
                planet: pursuit_week4_automation::astrology::ephemeris::Planet::Jupiter,
                return_dates: vec![
                    NaiveDate::from_ymd_opt(2020, 6, 15).unwrap(),
                    NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                ],
                zone_days: 30,
            },
            ..Default::default()
        };
        let filtered = cfg.filter_days(&data);
        // Two windows of 61 days each = 122 days max
        assert!(filtered.len() > 60 && filtered.len() <= 122,
            "Expected 60-122 days from two return zones, got {}", filtered.len());
    }
}
