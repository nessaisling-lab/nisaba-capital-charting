//! Shared financial statistics — used by both the backtest engine and the paper
//! trading engine. Extracted to avoid duplicating math across modules.

use chrono::NaiveDate;

/// Compute maximum drawdown percentage from a series of portfolio values.
/// Returns 0.0 if the series is empty.
pub fn max_drawdown_pct(values: &[f64]) -> f64 {
    let mut peak = f64::NEG_INFINITY;
    let mut max_dd = 0.0_f64;
    for &v in values {
        if v > peak {
            peak = v;
        }
        if peak > 0.0 {
            let dd = (peak - v) / peak * 100.0;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    max_dd
}

/// Compute win rate as a percentage from a slice of return percentages.
/// A "win" is any return > 0. Returns 0.0 if the slice is empty.
pub fn win_rate_pct(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let wins = returns.iter().filter(|&&r| r > 0.0).count();
    wins as f64 / returns.len() as f64 * 100.0
}

/// Annualized Sharpe ratio from a series of daily portfolio values.
///
/// Uses daily log returns, assumes 252 trading days/year, and a risk-free
/// rate of 0 (standard for paper trading comparisons). Returns 0.0 if fewer
/// than 2 data points or zero standard deviation.
pub fn sharpe_ratio(daily_values: &[f64]) -> f64 {
    if daily_values.len() < 2 {
        return 0.0;
    }

    // Daily log returns
    let returns: Vec<f64> = daily_values
        .windows(2)
        .map(|w| (w[1] / w[0]).ln())
        .filter(|r| r.is_finite())
        .collect();

    if returns.is_empty() {
        return 0.0;
    }

    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    if std_dev < 1e-12 {
        return 0.0;
    }

    // Annualize: multiply by sqrt(252)
    (mean / std_dev) * 252.0_f64.sqrt()
}

/// Average holding period in calendar days from a list of (entry_date, exit_date) pairs.
/// Returns 0.0 if no trades.
pub fn avg_holding_days(trades: &[(NaiveDate, NaiveDate)]) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }
    let total: i64 = trades
        .iter()
        .map(|(entry, exit)| (*exit - *entry).num_days())
        .sum();
    total as f64 / trades.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_max_drawdown_basic() {
        // Peak at 110, drops to 90 -> dd = (110-90)/110 * 100 = 18.18%
        let values = vec![100.0, 105.0, 110.0, 95.0, 90.0, 100.0];
        let dd = max_drawdown_pct(&values);
        assert!((dd - 18.18).abs() < 0.1, "Expected ~18.18%, got {dd:.2}%");
    }

    #[test]
    fn test_max_drawdown_no_drawdown() {
        let values = vec![100.0, 101.0, 102.0, 103.0];
        assert_eq!(max_drawdown_pct(&values), 0.0);
    }

    #[test]
    fn test_max_drawdown_empty() {
        assert_eq!(max_drawdown_pct(&[]), 0.0);
    }

    #[test]
    fn test_win_rate() {
        let returns = vec![5.0, -2.0, 3.0, -1.0, 10.0];
        let wr = win_rate_pct(&returns);
        assert!((wr - 60.0).abs() < 0.01, "Expected 60%, got {wr:.2}%");
    }

    #[test]
    fn test_win_rate_empty() {
        assert_eq!(win_rate_pct(&[]), 0.0);
    }

    #[test]
    fn test_sharpe_ratio_flat() {
        // Flat portfolio -> zero return -> zero Sharpe
        let values = vec![100.0, 100.0, 100.0, 100.0];
        assert_eq!(sharpe_ratio(&values), 0.0);
    }

    #[test]
    fn test_sharpe_ratio_positive() {
        // Steadily increasing portfolio should have positive Sharpe
        let values: Vec<f64> = (0..100).map(|i| 100.0 + i as f64 * 0.5).collect();
        let s = sharpe_ratio(&values);
        assert!(s > 0.0, "Expected positive Sharpe, got {s:.4}");
    }

    #[test]
    fn test_sharpe_ratio_too_few() {
        assert_eq!(sharpe_ratio(&[100.0]), 0.0);
        assert_eq!(sharpe_ratio(&[]), 0.0);
    }

    #[test]
    fn test_avg_holding_days() {
        let trades = vec![
            (
                NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 1, 11).unwrap(),
            ),
            (
                NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 2, 6).unwrap(),
            ),
        ];
        let avg = avg_holding_days(&trades);
        // (10 + 5) / 2 = 7.5 days
        assert!((avg - 7.5).abs() < 0.01, "Expected 7.5, got {avg:.2}");
    }

    #[test]
    fn test_avg_holding_days_empty() {
        assert_eq!(avg_holding_days(&[]), 0.0);
    }
}
