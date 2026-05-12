//! Re-exports shared indicator math from the lib crate.
//! The canonical implementations live in `nisaba_engine::indicators`.

pub use nisaba_engine::indicators::{compute_lagrange_score, Indicators};

use nisaba_engine::models::{PriceRow, SentimentScore};

// ---------------------------------------------------------------------------
// Dashboard-only fear/greed score (financial signals only, no macro/astro)
// ---------------------------------------------------------------------------

pub fn compute_ticker_score(
    indicators: &Indicators,
    rows: &[PriceRow],
    sentiment: &Option<SentimentScore>,
) -> (f32, String) {
    let latest = rows.first()
        .and_then(|r| r.close.to_string().parse::<f32>().ok())
        .unwrap_or(0.0);

    let rsi_v = Indicators::last(&indicators.rsi_vals).unwrap_or(50.0);

    let sma50 = Indicators::last(&indicators.sma50).unwrap_or(latest);
    let momentum = if sma50 > 0.0 {
        ((latest / sma50 - 1.0) * 500.0 + 50.0).clamp(0.0, 100.0)
    } else {
        50.0
    };

    let macd_v = Indicators::last(&indicators.macd_line).unwrap_or(0.0);
    let signal_v = Indicators::last(&indicators.macd_sig).unwrap_or(0.0);
    let macd_score = if latest > 0.0 {
        ((macd_v - signal_v) / latest * 500.0 + 50.0).clamp(0.0, 100.0)
    } else {
        50.0
    };

    let sent_score = sentiment.as_ref()
        .and_then(|s| s.sentiment_score.as_ref())
        .and_then(|v| v.to_string().parse::<f32>().ok())
        .map(|v| (v + 1.0) * 50.0)
        .unwrap_or(50.0);

    let score = (rsi_v * 0.30 + momentum * 0.30 + macd_score * 0.20 + sent_score * 0.20)
        .clamp(0.0, 100.0);

    let label = match score as u32 {
        0..=24  => "Extreme Fear",
        25..=44 => "Fear",
        45..=55 => "Neutral",
        56..=75 => "Greed",
        _       => "Extreme Greed",
    }.to_string();

    (score, label)
}
