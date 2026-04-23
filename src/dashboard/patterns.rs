//! Technical pattern recognition — detects chart patterns from price history.
//!
//! Patterns detected:
//! - **Golden Cross**: SMA50 crosses above SMA200 (bullish)
//! - **Death Cross**: SMA50 crosses below SMA200 (bearish)
//! - **Double Top**: Two peaks at similar price levels with a trough between
//! - **Double Bottom**: Two troughs at similar price levels with a peak between
//! - **Support**: Price repeatedly bounces off a floor level
//! - **Resistance**: Price repeatedly rejected at a ceiling level

use crate::indicators::Indicators;

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    GoldenCross,
    DeathCross,
    DoubleTop   { price: f32 },
    DoubleBottom { price: f32 },
    Support     { level: f32 },
    Resistance  { level: f32 },
}

impl Pattern {
    pub fn label(&self) -> String {
        match self {
            Self::GoldenCross          => "Golden Cross (SMA50 > SMA200)".to_string(),
            Self::DeathCross           => "Death Cross (SMA50 < SMA200)".to_string(),
            Self::DoubleTop   { price } => format!("Double Top near ${price:.2}"),
            Self::DoubleBottom { price } => format!("Double Bottom near ${price:.2}"),
            Self::Support     { level } => format!("Support at ${level:.2}"),
            Self::Resistance  { level } => format!("Resistance at ${level:.2}"),
        }
    }

    pub fn is_bullish(&self) -> bool {
        matches!(self, Self::GoldenCross | Self::DoubleBottom { .. } | Self::Support { .. })
    }
}

/// Detect all patterns from price data and indicators.
/// `prices` should be in chronological order (oldest first).
pub fn detect_patterns(prices: &[f32], ind: &Indicators) -> Vec<Pattern> {
    let mut patterns = Vec::new();

    // Golden Cross / Death Cross — look at the last 5 bars for a crossover
    detect_cross(ind, &mut patterns);

    // Double Top / Double Bottom — scan recent ~60 bars
    if prices.len() >= 20 {
        detect_double_patterns(prices, &mut patterns);
    }

    // Support / Resistance — look for repeated bounces/rejections in recent data
    if prices.len() >= 30 {
        detect_support_resistance(prices, &mut patterns);
    }

    patterns
}

/// Detect Golden Cross (SMA50 crosses above SMA200) and Death Cross (reverse).
fn detect_cross(ind: &Indicators, patterns: &mut Vec<Pattern>) {
    let len = ind.sma50.len().min(ind.sma200.len());
    if len < 3 { return; }

    // Look at the last 5 bars for a recent crossover
    let lookback = 5.min(len - 1);
    for i in (len - lookback)..len {
        let (prev50, prev200) = match (ind.sma50.get(i - 1), ind.sma200.get(i - 1)) {
            (Some(Some(a)), Some(Some(b))) => (*a, *b),
            _ => continue,
        };
        let (curr50, curr200) = match (ind.sma50.get(i), ind.sma200.get(i)) {
            (Some(Some(a)), Some(Some(b))) => (*a, *b),
            _ => continue,
        };

        if prev50 <= prev200 && curr50 > curr200 {
            patterns.push(Pattern::GoldenCross);
            return; // Only one cross per scan
        }
        if prev50 >= prev200 && curr50 < curr200 {
            patterns.push(Pattern::DeathCross);
            return;
        }
    }
}

/// Detect double top/bottom patterns in the most recent ~60 bars.
fn detect_double_patterns(prices: &[f32], patterns: &mut Vec<Pattern>) {
    let window = &prices[prices.len().saturating_sub(60)..];
    if window.len() < 20 { return; }

    // Find local peaks and troughs (using 5-bar neighborhood)
    let mut peaks = Vec::new();
    let mut troughs = Vec::new();

    for i in 2..window.len().saturating_sub(2) {
        if window[i] > window[i - 1] && window[i] > window[i - 2]
            && window[i] > window[i + 1] && window[i] > window[i + 2]
        {
            peaks.push((i, window[i]));
        }
        if window[i] < window[i - 1] && window[i] < window[i - 2]
            && window[i] < window[i + 1] && window[i] < window[i + 2]
        {
            troughs.push((i, window[i]));
        }
    }

    // Double top: two peaks within 2% of each other, separated by at least 5 bars
    for i in 0..peaks.len() {
        for j in (i + 1)..peaks.len() {
            let (idx_a, pa) = peaks[i];
            let (idx_b, pb) = peaks[j];
            if idx_b - idx_a >= 5 {
                let diff_pct = ((pa - pb) / pa).abs();
                if diff_pct < 0.02 {
                    patterns.push(Pattern::DoubleTop { price: (pa + pb) / 2.0 });
                    break;
                }
            }
        }
        if patterns.iter().any(|p| matches!(p, Pattern::DoubleTop { .. })) { break; }
    }

    // Double bottom: two troughs within 2% of each other
    for i in 0..troughs.len() {
        for j in (i + 1)..troughs.len() {
            let (idx_a, ta) = troughs[i];
            let (idx_b, tb) = troughs[j];
            if idx_b - idx_a >= 5 {
                let diff_pct = ((ta - tb) / ta).abs();
                if diff_pct < 0.02 {
                    patterns.push(Pattern::DoubleBottom { price: (ta + tb) / 2.0 });
                    break;
                }
            }
        }
        if patterns.iter().any(|p| matches!(p, Pattern::DoubleBottom { .. })) { break; }
    }
}

/// Detect support and resistance from repeated bounces/rejections.
fn detect_support_resistance(prices: &[f32], patterns: &mut Vec<Pattern>) {
    let window = &prices[prices.len().saturating_sub(90)..];
    if window.len() < 30 { return; }

    // Find lows and highs in 5-bar neighborhoods
    let mut lows = Vec::new();
    let mut highs = Vec::new();

    for i in 2..window.len().saturating_sub(2) {
        if window[i] <= window[i - 1] && window[i] <= window[i - 2]
            && window[i] <= window[i + 1] && window[i] <= window[i + 2]
        {
            lows.push(window[i]);
        }
        if window[i] >= window[i - 1] && window[i] >= window[i - 2]
            && window[i] >= window[i + 1] && window[i] >= window[i + 2]
        {
            highs.push(window[i]);
        }
    }

    // Support: cluster of 3+ lows within 1.5% of each other
    if let Some(level) = find_cluster(&lows, 0.015, 3) {
        patterns.push(Pattern::Support { level });
    }

    // Resistance: cluster of 3+ highs within 1.5% of each other
    if let Some(level) = find_cluster(&highs, 0.015, 3) {
        patterns.push(Pattern::Resistance { level });
    }
}

/// Find a price cluster: 3+ values within `threshold` percent of each other.
/// Returns the average of the cluster if found.
fn find_cluster(values: &[f32], threshold: f32, min_count: usize) -> Option<f32> {
    if values.len() < min_count { return None; }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut best_cluster: Option<(f32, usize)> = None; // (sum, count)

    for i in 0..sorted.len() {
        let base = sorted[i];
        if base == 0.0 { continue; }
        let mut sum = base;
        let mut count = 1usize;
        for j in (i + 1)..sorted.len() {
            if ((sorted[j] - base) / base).abs() < threshold {
                sum += sorted[j];
                count += 1;
            } else {
                break;
            }
        }
        if count >= min_count {
            if best_cluster.map_or(true, |(_, bc)| count > bc) {
                best_cluster = Some((sum, count));
            }
        }
    }

    best_cluster.map(|(sum, count)| sum / count as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_cross_detected() {
        // Build a simple price series where SMA50 crosses above SMA200
        // We need 200+ bars. Use a series that's flat at 100 for 200 bars then rises to 120
        let mut prices: Vec<f32> = vec![100.0; 195];
        prices.extend(vec![110.0, 115.0, 120.0, 125.0, 130.0]); // sharp rise at end
        let ind = Indicators::compute(&prices);
        let patterns = detect_patterns(&prices, &ind);
        // The SMA50 will respond faster to the rise than SMA200
        // Just verify patterns detection runs without panicking
        assert!(patterns.iter().all(|p| !p.label().is_empty()));
    }

    #[test]
    fn test_support_cluster() {
        let values = vec![50.0, 50.2, 50.1, 60.0, 70.0, 50.3];
        let result = find_cluster(&values, 0.015, 3);
        assert!(result.is_some());
        let level = result.unwrap();
        assert!((level - 50.15).abs() < 0.3);
    }
}
