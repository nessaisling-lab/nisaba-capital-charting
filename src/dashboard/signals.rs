use chrono::Utc;
use pursuit_week4_automation::models::{
    AnalystRating, AstroScore, EarningsDate, MacroIndicator, PriceRow,
    RssToneScore, SentimentScore, ShortInterest,
};

use crate::indicators::Indicators;

// ---------------------------------------------------------------------------
// Per-ticker signal bullets — plain-English synthesis of all loaded data
// ---------------------------------------------------------------------------

pub fn generate_signal_bullets(
    ticker: &str,
    indicators: &Indicators,
    rows: &[PriceRow],
    sentiment: &Option<SentimentScore>,
    astro_score: &Option<AstroScore>,
    macro_data: &[MacroIndicator],
    short_interest: &Option<ShortInterest>,
    analyst_rating: &Option<AnalystRating>,
    earnings: &[EarningsDate],
    rss_tone: &Option<RssToneScore>,
) -> Vec<String> {
    let mut bullets: Vec<String> = Vec::new();

    let latest = rows.first()
        .and_then(|r| r.close.to_string().parse::<f32>().ok())
        .unwrap_or(0.0);

    // RSI signal
    if let Some(rsi) = Indicators::last(&indicators.rsi_vals) {
        let msg = if rsi > 70.0 {
            format!("RSI {rsi:.0} — overbought, watch for reversal")
        } else if rsi < 30.0 {
            format!("RSI {rsi:.0} — oversold, potential bounce setup")
        } else {
            format!("RSI {rsi:.0} — neutral range")
        };
        bullets.push(msg);
    }

    // Price vs SMA50
    if let Some(sma50) = Indicators::last(&indicators.sma50) {
        if sma50 > 0.0 {
            let pct = (latest / sma50 - 1.0) * 100.0;
            let msg = if pct > 5.0 {
                format!("Price {pct:+.1}% above SMA50 — strong uptrend")
            } else if pct < -5.0 {
                format!("Price {pct:+.1}% below SMA50 — downtrend")
            } else {
                format!("Price {pct:+.1}% vs SMA50 — near equilibrium")
            };
            bullets.push(msg);
        }
    }

    // Bollinger band position
    if let (Some(upper), Some(lower)) = (
        Indicators::last(&indicators.bb_upper),
        Indicators::last(&indicators.bb_lower),
    ) {
        let band_width = upper - lower;
        if band_width > 0.0 {
            let pos = (latest - lower) / band_width;
            let msg = if pos > 0.95 {
                "Price at upper Bollinger band — overbought extreme".to_string()
            } else if pos < 0.05 {
                "Price at lower Bollinger band — oversold extreme, mean-reversion setup".to_string()
            } else if pos > 0.7 {
                format!("Price in upper Bollinger band ({:.0}%)", pos * 100.0)
            } else if pos < 0.3 {
                format!("Price in lower Bollinger band ({:.0}%)", pos * 100.0)
            } else {
                format!("Price mid-band ({:.0}%)", pos * 100.0)
            };
            bullets.push(msg);
        }
    }

    // MACD crossover
    if let (Some(macd_v), Some(sig_v)) = (
        Indicators::last(&indicators.macd_line),
        Indicators::last(&indicators.macd_sig),
    ) {
        let hist = macd_v - sig_v;
        let msg = if hist > 0.0 {
            format!("MACD bullish — histogram {hist:+.2}")
        } else {
            format!("MACD bearish — histogram {hist:+.2}")
        };
        bullets.push(msg);
    }

    // Short interest
    if let Some(si) = short_interest {
        if let Some(pct) = si.short_pct.as_ref()
            .and_then(|v| v.to_string().parse::<f32>().ok())
        {
            let price_rising = rows.len() >= 5 && {
                let old = rows[4].close.to_string().parse::<f32>().unwrap_or(latest);
                latest > old
            };
            let msg = if pct > 20.0 && price_rising {
                format!("Short interest {pct:.1}% — HIGH + rising price: squeeze setup active")
            } else if pct > 20.0 {
                format!("Short interest {pct:.1}% — HIGH, watch for covering rally")
            } else if pct > 10.0 {
                format!("Short interest {pct:.1}% — elevated")
            } else {
                format!("Short interest {pct:.1}% — normal")
            };
            bullets.push(msg);
        }
    }

    // News sentiment (Alpha Vantage)
    if let Some(s) = sentiment {
        let label = s.sentiment_label.as_deref().unwrap_or("—");
        let score = s.sentiment_score.as_ref()
            .and_then(|v| v.to_string().parse::<f32>().ok())
            .unwrap_or(0.0);
        bullets.push(format!("News sentiment: {label} ({score:+.2})"));
    }

    // RSS tone sentiment (keyword-based, from 25 feeds)
    if let Some(tone) = rss_tone {
        let label = tone.tone_label.as_deref().unwrap_or("—");
        let score = tone.tone_score.as_ref()
            .and_then(|v| v.to_string().parse::<f32>().ok())
            .unwrap_or(0.0);
        let count = tone.article_count.unwrap_or(0);
        bullets.push(format!("RSS tone: {label} ({score:+.2}) from {count} article(s)"));
    }

    // Analyst ratings
    if let Some(r) = analyst_rating {
        let total = r.strong_buy + r.buy + r.hold + r.sell + r.strong_sell;
        if total > 0 {
            let bullish = r.strong_buy + r.buy;
            let pct = bullish as f32 / total as f32 * 100.0;
            let msg = if pct > 70.0 {
                format!("Analysts: {bullish}/{total} bullish ({pct:.0}%) — strong consensus buy")
            } else if pct > 50.0 {
                format!("Analysts: {bullish}/{total} bullish ({pct:.0}%)")
            } else {
                let bearish = r.sell + r.strong_sell;
                format!("Analysts: mixed — {bullish} bull, {bearish} bear of {total}")
            };
            bullets.push(msg);
        }
    }

    // Macro context — VIX
    let find_macro = |id: &str| -> Option<f32> {
        macro_data.iter()
            .find(|m| m.series_id == id)
            .and_then(|m| m.value.as_ref())
            .and_then(|v| v.to_string().parse::<f32>().ok())
    };

    if let Some(vix) = find_macro("VIXCLS") {
        let msg = if vix < 15.0 {
            format!("VIX {vix:.1} — calm markets, low volatility regime")
        } else if vix < 25.0 {
            format!("VIX {vix:.1} — moderate volatility")
        } else {
            format!("VIX {vix:.1} — elevated fear, size positions carefully")
        };
        bullets.push(msg);
    }

    // Macro context — yield curve
    if let Some(spread) = find_macro("T10Y2Y") {
        let msg = if spread > 0.5 {
            format!("Yield curve: normal ({spread:+.2}%) — no recession signal")
        } else if spread > -0.25 {
            format!("Yield curve: flat ({spread:+.2}%) — watch for inversion")
        } else {
            format!("Yield curve: inverted ({spread:+.2}%) — recession risk elevated")
        };
        bullets.push(msg);
    }

    // Astrology context
    if let Some(astro) = astro_score {
        let score = astro.astro_score.unwrap_or(50.0) as f32;
        let label = astro.astro_label.as_deref().unwrap_or("—");
        let mercury = if astro.mercury_rx.unwrap_or(false) {
            " | Mercury Rx — avoid major commitments"
        } else {
            ""
        };
        let moon = astro.moon_phase.as_deref().unwrap_or("");
        let moon_str = if moon.is_empty() { String::new() } else { format!(" | {moon}") };
        bullets.push(format!("Astro: {label} ({score:.0}){moon_str}{mercury}"));
    }

    // Upcoming earnings
    let today = Utc::now().date_naive();
    if let Some(next) = earnings.iter()
        .filter(|e| e.ticker == ticker && e.earnings_date >= today)
        .min_by_key(|e| e.earnings_date)
    {
        let days = (next.earnings_date - today).num_days();
        let msg = if days == 0 {
            "EARNINGS TODAY — expect extreme volatility".to_string()
        } else if days <= 3 {
            format!("Earnings in {days} day(s) — volatility expected, consider reducing size")
        } else if days <= 14 {
            format!("Earnings in {days} days ({})", next.earnings_date)
        } else {
            format!("Next earnings: {}", next.earnings_date)
        };
        bullets.push(msg);
    }

    bullets
}
