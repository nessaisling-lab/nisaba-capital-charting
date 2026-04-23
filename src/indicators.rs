//! Technical indicator math shared between the scraper and dashboard binaries.

use crate::models::{AstroScore, MacroIndicator, PriceRow, SentimentScore, ShortInterest};

// ---------------------------------------------------------------------------
// Core math
// ---------------------------------------------------------------------------

pub fn sma(prices: &[f32], period: usize) -> Vec<Option<f32>> {
    let mut out = vec![None; prices.len()];
    for i in (period - 1)..prices.len() {
        let sum: f32 = prices[(i + 1 - period)..=i].iter().sum();
        out[i] = Some(sum / period as f32);
    }
    out
}

pub fn ema(prices: &[f32], period: usize) -> Vec<Option<f32>> {
    let mut out = vec![None; prices.len()];
    if prices.len() < period { return out; }
    let k = 2.0 / (period as f32 + 1.0);
    let seed: f32 = prices[..period].iter().sum::<f32>() / period as f32;
    out[period - 1] = Some(seed);
    for i in period..prices.len() {
        if let Some(prev) = out[i - 1] {
            out[i] = Some(prices[i] * k + prev * (1.0 - k));
        }
    }
    out
}

pub fn bollinger_bands(prices: &[f32], period: usize) -> (Vec<Option<f32>>, Vec<Option<f32>>, Vec<Option<f32>>) {
    let mid = sma(prices, period);
    let mut upper = vec![None; prices.len()];
    let mut lower = vec![None; prices.len()];
    for i in (period - 1)..prices.len() {
        if let Some(m) = mid[i] {
            let variance = prices[(i + 1 - period)..=i]
                .iter()
                .map(|&p| (p - m).powi(2))
                .sum::<f32>()
                / period as f32;
            let sd = variance.sqrt();
            upper[i] = Some(m + 2.0 * sd);
            lower[i] = Some(m - 2.0 * sd);
        }
    }
    (mid, upper, lower)
}

pub fn rsi(prices: &[f32], period: usize) -> Vec<Option<f32>> {
    let mut out = vec![None; prices.len()];
    if prices.len() <= period { return out; }
    let mut avg_gain = 0.0f32;
    let mut avg_loss = 0.0f32;
    for i in 1..=period {
        let d = prices[i] - prices[i - 1];
        if d >= 0.0 { avg_gain += d; } else { avg_loss -= d; }
    }
    avg_gain /= period as f32;
    avg_loss /= period as f32;
    let rs_val = |ag: f32, al: f32| if al == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + ag / al) };
    out[period] = Some(rs_val(avg_gain, avg_loss));
    for i in (period + 1)..prices.len() {
        let d = prices[i] - prices[i - 1];
        let gain = if d > 0.0 { d } else { 0.0 };
        let loss = if d < 0.0 { -d } else { 0.0 };
        avg_gain = (avg_gain * (period as f32 - 1.0) + gain) / period as f32;
        avg_loss = (avg_loss * (period as f32 - 1.0) + loss) / period as f32;
        out[i] = Some(rs_val(avg_gain, avg_loss));
    }
    out
}

pub fn macd(prices: &[f32]) -> (Vec<Option<f32>>, Vec<Option<f32>>) {
    let e12 = ema(prices, 12);
    let e26 = ema(prices, 26);
    let mut macd_line = vec![None; prices.len()];
    for i in 0..prices.len() {
        if let (Some(a), Some(b)) = (e12[i], e26[i]) {
            macd_line[i] = Some(a - b);
        }
    }
    let start = macd_line.iter().position(|v| v.is_some()).unwrap_or(0);
    let dense: Vec<f32> = macd_line[start..].iter().filter_map(|&v| v).collect();
    let signal_dense = ema(&dense, 9);
    let mut signal = vec![None; prices.len()];
    let mut di = 0usize;
    for i in start..prices.len() {
        if macd_line[i].is_some() {
            if let Some(v) = signal_dense.get(di).and_then(|&x| x) {
                signal[i] = Some(v);
            }
            di += 1;
        }
    }
    (macd_line, signal)
}

// ---------------------------------------------------------------------------
// Indicators bundle
// ---------------------------------------------------------------------------

pub struct Indicators {
    pub sma20:     Vec<Option<f32>>,
    pub sma50:     Vec<Option<f32>>,
    pub sma200:    Vec<Option<f32>>,
    pub bb_upper:  Vec<Option<f32>>,
    pub bb_lower:  Vec<Option<f32>>,
    pub rsi_vals:  Vec<Option<f32>>,
    pub macd_line: Vec<Option<f32>>,
    pub macd_sig:  Vec<Option<f32>>,
}

impl Indicators {
    pub fn compute(prices: &[f32]) -> Self {
        let sma20 = sma(prices, 20);
        let sma50 = sma(prices, 50);
        let sma200 = sma(prices, 200);
        let (_, bb_upper, bb_lower) = bollinger_bands(prices, 20);
        let rsi_vals = rsi(prices, 14);
        let (macd_line, macd_sig) = macd(prices);
        Self { sma20, sma50, sma200, bb_upper, bb_lower, rsi_vals, macd_line, macd_sig }
    }

    pub fn last<T: Copy>(v: &[Option<T>]) -> Option<T> {
        v.iter().rev().find_map(|&x| x)
    }
}

// ---------------------------------------------------------------------------
// Lagrange Score — shared computation used by both scraper and dashboard
// ---------------------------------------------------------------------------
//
// Component weights (v2.0.5 rebalance — astrology leads):
//   Astrology    40%  (today's astro_score — lead signal)
//   Financial    25%  (RSI + momentum + MACD + sentiment — verification)
//   Macro        20%  (VIX + yield curve)
//   Short Squeeze 15% (short % × price direction)
//
// Labels: Misaligned / Unfavorable / Neutral / Favorable / Optimal

pub fn compute_lagrange_score(
    indicators: &Indicators,
    rows: &[PriceRow],
    sentiment: &Option<SentimentScore>,
    astro_score: &Option<AstroScore>,
    macro_data: &[MacroIndicator],
    short_interest: &Option<ShortInterest>,
) -> (f32, String, LagrangeComponents) {
    let latest = rows.first()
        .and_then(|r| r.close.to_string().parse::<f32>().ok())
        .unwrap_or(0.0);

    // Financial component (RSI + momentum + MACD + sentiment)
    let rsi_v = Indicators::last(&indicators.rsi_vals).unwrap_or(50.0);
    let sma50 = Indicators::last(&indicators.sma50).unwrap_or(latest);
    let momentum = if sma50 > 0.0 {
        ((latest / sma50 - 1.0) * 500.0 + 50.0).clamp(0.0, 100.0)
    } else { 50.0 };
    let macd_v  = Indicators::last(&indicators.macd_line).unwrap_or(0.0);
    let sig_v   = Indicators::last(&indicators.macd_sig).unwrap_or(0.0);
    let macd_score = if latest > 0.0 {
        ((macd_v - sig_v) / latest * 500.0 + 50.0).clamp(0.0, 100.0)
    } else { 50.0 };
    let sent_score = sentiment.as_ref()
        .and_then(|s| s.sentiment_score.as_ref())
        .and_then(|v| v.to_string().parse::<f32>().ok())
        .map(|v| (v + 1.0) * 50.0)
        .unwrap_or(50.0);
    let fin_score = (rsi_v * 0.30 + momentum * 0.30 + macd_score * 0.20 + sent_score * 0.20)
        .clamp(0.0, 100.0);

    // Astrology component
    let astro = astro_score
        .as_ref()
        .and_then(|s| s.astro_score.map(|v| v as f32))
        .unwrap_or(50.0);

    // Macro component (VIX + yield spread)
    let find_macro = |id: &str| -> Option<f32> {
        macro_data.iter()
            .find(|m| m.series_id == id)
            .and_then(|m| m.value.as_ref())
            .and_then(|v| v.to_string().parse::<f32>().ok())
    };
    let vix_score = find_macro("VIXCLS")
        .map(|v| (90.0 - (v - 10.0) * 1.4).clamp(0.0, 100.0))
        .unwrap_or(50.0);
    let spread_score = find_macro("T10Y2Y")
        .map(|s| ((s + 1.0) * 20.0 + 30.0).clamp(0.0, 100.0))
        .unwrap_or(50.0);
    let macro_score = (vix_score * 0.6 + spread_score * 0.4).clamp(0.0, 100.0);

    // Short squeeze component
    let short_score = short_interest.as_ref().and_then(|si| {
        let pct = si.short_pct.as_ref()?.to_string().parse::<f32>().ok()?;
        let price_rising = rows.len() >= 5 && {
            let old = rows[4].close.to_string().parse::<f32>().unwrap_or(latest);
            latest > old
        };
        let base: f32 = if pct > 30.0 { 75.0 } else if pct > 20.0 { 65.0 } else if pct > 10.0 { 50.0 } else { 40.0 };
        let bonus: f32 = if price_rising && pct > 15.0 { 15.0 } else { 0.0 };
        Some((base + bonus).clamp(0.0, 100.0))
    }).unwrap_or(50.0);

    // v2.0.5: Astrology leads (40%), Financial verifies (25%), Macro context (20%), Short squeeze (15%)
    let score = (astro * 0.40 + fin_score * 0.25 + macro_score * 0.20 + short_score * 0.15)
        .clamp(0.0, 100.0);

    let label = match score as u32 {
        0..=24  => "Misaligned",
        25..=44 => "Unfavorable",
        45..=55 => "Neutral",
        56..=75 => "Favorable",
        _       => "Optimal",
    }.to_string();

    let concordance = compute_concordance(astro, fin_score);

    let components = LagrangeComponents {
        fin_score,
        astro_score: astro,
        macro_score,
        short_score,
        concordance,
    };

    (score, label, components)
}

/// Breakdown of each Lagrange component — stored in lagrange_history for debugging.
pub struct LagrangeComponents {
    pub fin_score:    f32,
    pub astro_score:  f32,
    pub macro_score:  f32,
    pub short_score:  f32,
    pub concordance:  Concordance,
}

// ---------------------------------------------------------------------------
// Concordance — astro vs financial agreement indicator
// ---------------------------------------------------------------------------
//
// When astrology and financials agree, confidence is high.
// When they disagree, the system flags it for review.
// This is Principle #2 (Candor): when data conflicts, say so plainly.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Concordance {
    /// Astro favorable (>60) AND financials strong (>60). High confidence.
    StrongConfirm,
    /// Astro favorable (>60) AND financials neutral (40-60). Moderate confidence.
    MildConfirm,
    /// Astro and financials point in different directions. Flag for review.
    Divergence,
    /// Astro unfavorable (<40) AND financials neutral (40-60). Moderate caution.
    MildDeny,
    /// Astro unfavorable (<40) AND financials weak (<40). High conviction to avoid.
    StrongDeny,
}

impl Concordance {
    pub fn name(self) -> &'static str {
        match self {
            Concordance::StrongConfirm => "Strong Confirm",
            Concordance::MildConfirm   => "Mild Confirm",
            Concordance::Divergence    => "Divergence",
            Concordance::MildDeny      => "Mild Deny",
            Concordance::StrongDeny    => "Strong Deny",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Concordance::StrongConfirm => "++",
            Concordance::MildConfirm   => "+",
            Concordance::Divergence    => "!",
            Concordance::MildDeny      => "-",
            Concordance::StrongDeny    => "--",
        }
    }
}

/// Compute concordance between astrology and financial scores.
pub fn compute_concordance(astro: f32, fin: f32) -> Concordance {
    let astro_favorable = astro > 60.0;
    let astro_unfavorable = astro < 40.0;
    let fin_strong = fin > 60.0;
    let fin_weak = fin < 40.0;

    match (astro_favorable, astro_unfavorable, fin_strong, fin_weak) {
        (true, _, true, _)  => Concordance::StrongConfirm,
        (true, _, _, true)  => Concordance::Divergence,  // Astro says buy, financials say sell
        (true, _, _, _)     => Concordance::MildConfirm, // Astro favorable, financials neutral
        (_, true, true, _)  => Concordance::Divergence,  // Astro says sell, financials say buy
        (_, true, _, true)  => Concordance::StrongDeny,
        (_, true, _, _)     => Concordance::MildDeny,    // Astro unfavorable, financials neutral
        _                   => Concordance::MildConfirm, // Both neutral
    }
}
