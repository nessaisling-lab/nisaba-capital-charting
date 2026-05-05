//! AI Agent Personas — template-based investment analysis through 4 philosophical lenses.
//!
//! Each agent reviews the astrological reading, fundamental metrics, and market
//! data through their own investment philosophy:
//!
//! - **Buffett**: Moat + FCF + margin of safety. Acknowledges astro when it
//!   confirms fundamentals. "When the cosmos and the balance sheet agree..."
//! - **Graham**: Pure valuation metrics. Politely skeptical of astrology.
//!   Focuses on P/B < 1.5, P/E < 15, margin of safety.
//! - **Lynch**: "Know what you own." PEG ratio, growth at reasonable price.
//!   Sees astrology as market psychology indicator.
//! - **Munger**: Mental models, durable moat, quality over price. Views
//!   astrology as one data point among many.

use pursuit_week4_automation::models::FundamentalMetric;

/// Template or LLM analysis mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMode {
    Template,
    Llm,
}

/// Which investment persona is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentPersona {
    Buffett,
    Graham,
    Lynch,
    Munger,
}

impl AgentPersona {
    pub fn all() -> &'static [AgentPersona] {
        &[Self::Buffett, Self::Graham, Self::Lynch, Self::Munger]
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Buffett => "Warren Buffett",
            Self::Graham  => "Benjamin Graham",
            Self::Lynch   => "Peter Lynch",
            Self::Munger  => "Charlie Munger",
        }
    }
    pub fn short_name(self) -> &'static str {
        match self {
            Self::Buffett => "Buffett",
            Self::Graham  => "Graham",
            Self::Lynch   => "Lynch",
            Self::Munger  => "Munger",
        }
    }
    pub fn philosophy(self) -> &'static str {
        match self {
            Self::Buffett => "Moat + Free Cash Flow + Margin of Safety",
            Self::Graham  => "Deep Value + Net-Net + Quantitative Discipline",
            Self::Lynch   => "Growth at Reasonable Price + Know What You Own",
            Self::Munger  => "Quality Businesses + Mental Models + Patience",
        }
    }
}

/// Everything the agent needs to form an opinion.
#[allow(dead_code)] // Fields populated for LLM agent path (v2.8)
pub struct AgentContext {
    pub ticker: String,
    pub fundamentals: Option<FundamentalMetric>,
    pub astro_score: Option<f64>,
    pub astro_label: Option<String>,
    pub dominant_theme: Option<String>,
    pub concordance: Option<String>,
    pub lagrange_score: Option<f32>,
    pub lagrange_label: Option<String>,
    pub current_price: Option<f64>,
    pub mercury_rx: bool,
    pub moon_phase: Option<String>,
    // v10.0 "The Signal" — richer context for verdicts
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub recent_headlines: Vec<String>,
    pub rss_tone_score: Option<f64>,
    pub rss_tone_label: Option<String>,
}

/// The agent's output.
#[derive(Debug, Clone)]
pub struct AgentAnalysis {
    pub persona: AgentPersona,
    pub headline: String,       // One-line summary
    pub analysis: String,       // 3-5 sentence analysis
    pub verdict: AgentVerdict,
    pub key_metrics: Vec<(String, String, String)>, // (metric, value, assessment)
    pub astro_take: String,     // How this agent views the astrological signal
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentVerdict {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
    InsufficientData,
}

impl AgentVerdict {
    pub fn label(self) -> &'static str {
        match self {
            Self::StrongBuy  => "Strong Buy",
            Self::Buy        => "Buy",
            Self::Hold       => "Hold",
            Self::Sell       => "Sell",
            Self::StrongSell => "Strong Sell",
            Self::InsufficientData => "Insufficient Data",
        }
    }
}

// ---------------------------------------------------------------------------
// Shared evaluation helpers (deduplicates metric/verdict/assembly patterns)
// ---------------------------------------------------------------------------

/// Evaluate a fundamental metric against tiered thresholds.
///
/// Tiers are checked in order; first match wins. Each tier is
/// `(threshold, above, assessment, points)` where `above = true` means
/// `val > threshold` and `above = false` means `val < threshold`.
fn eval_metric(
    metrics: &mut Vec<(String, String, String)>,
    score: &mut i32,
    name: &str,
    value: Option<f64>,
    format_fn: impl FnOnce(f64) -> String,
    tiers: &[(f64, bool, &str, i32)],
    fallback: (&str, i32),
) {
    let Some(val) = value else { return };
    let (assessment, pts) = tiers.iter()
        .find_map(|&(thr, above, text, pts)| {
            let hit = if above { val > thr } else { val < thr };
            hit.then_some((text, pts))
        })
        .unwrap_or(fallback);
    metrics.push((name.to_string(), format_fn(val), assessment.to_string()));
    *score += pts;
}

/// Map a numeric score to a verdict. Thresholds are checked descending;
/// first entry where `score >= min` wins. Falls back to StrongSell.
fn score_to_verdict(score: i32, thresholds: &[(i32, AgentVerdict)]) -> AgentVerdict {
    for &(min, verdict) in thresholds {
        if score >= min { return verdict; }
    }
    AgentVerdict::StrongSell
}

/// Shared final assembly: use narrative if fundamentals are present,
/// try the no-fundamentals fallback, or return the no-data message.
fn assemble_analysis(
    ctx: &AgentContext,
    persona: AgentPersona,
    metrics: Vec<(String, String, String)>,
    score: i32,
    astro_take: String,
    headline: String,
    verdict: AgentVerdict,
    narrative_fn: fn(&AgentContext, i32) -> String,
    no_data_msg: &str,
) -> AgentAnalysis {
    let (analysis, final_verdict, final_metrics) = if ctx.fundamentals.is_some() {
        (narrative_fn(ctx, score), verdict, metrics)
    } else if let Some((fb, fb_v, fb_m)) = no_fundamentals_fallback(ctx, persona) {
        (fb, fb_v, fb_m)
    } else {
        (no_data_msg.to_string(), AgentVerdict::InsufficientData, metrics)
    };
    AgentAnalysis {
        persona,
        headline,
        analysis,
        verdict: final_verdict,
        key_metrics: final_metrics,
        astro_take,
    }
}

// ---------------------------------------------------------------------------
// Template-based analysis (always available, free, deterministic)
// ---------------------------------------------------------------------------

pub fn analyze(persona: AgentPersona, ctx: &AgentContext) -> AgentAnalysis {
    match persona {
        AgentPersona::Buffett => analyze_buffett(ctx),
        AgentPersona::Graham  => analyze_graham(ctx),
        AgentPersona::Lynch   => analyze_lynch(ctx),
        AgentPersona::Munger  => analyze_munger(ctx),
    }
}

// ---------------------------------------------------------------------------
// Buffett — "Price is what you pay, value is what you get"
// ---------------------------------------------------------------------------

fn analyze_buffett(ctx: &AgentContext) -> AgentAnalysis {
    let f = ctx.fundamentals.as_ref();
    let mut metrics = Vec::new();
    let mut score = 0i32;

    eval_metric(&mut metrics, &mut score, "ROE",
        f.and_then(|f| f.roe).map(|v| v * 100.0), |v| format!("{v:.1}%"),
        &[(20.0, true, "Excellent. Durable competitive advantage likely.", 2),
          (15.0, true, "Good. Meets my minimum threshold.", 1),
          (10.0, true, "Mediocre. Not the kind of business I prefer.", 0)],
        ("Poor. This business doesn't earn its cost of capital.", -2));

    eval_metric(&mut metrics, &mut score, "Debt/Equity",
        f.and_then(|f| f.debt_equity), |v| format!("{v:.2}"),
        &[(0.5, false, "Conservative balance sheet. I like that.", 2),
          (1.0, false, "Reasonable leverage.", 1),
          (2.0, false, "Getting leveraged. Proceed with caution.", -1)],
        ("Too much debt. Debt destroys value in downturns.", -2));

    eval_metric(&mut metrics, &mut score, "Free Cash Flow",
        f.and_then(|f| f.fcf).map(|v| v as f64), |v| format_large_number(v as i64),
        &[(10_000_000_000.0, true, "Massive cash generation. This is a cash machine.", 2),
          (1_000_000_000.0, true, "Strong cash flow. The business is real.", 1),
          (0.0, true, "Positive FCF, but not yet compelling.", 0)],
        ("Negative FCF. This business is burning cash, not generating it.", -2));

    eval_metric(&mut metrics, &mut score, "P/E Ratio",
        f.and_then(|f| f.pe_ratio), |v| format!("{v:.1}"),
        &[(0.0, false, "Negative earnings. Not investable by my standards.", -2),
          (15.0, false, "Bargain territory. If the business is good, this is attractive.", 2),
          (25.0, false, "Fair price for a quality business.", 1),
          (40.0, false, "Mr. Market is optimistic. Make sure the moat justifies this.", 0)],
        ("Expensive. Even great businesses can be bad investments at the wrong price.", -1));

    eval_metric(&mut metrics, &mut score, "Net Margin",
        f.and_then(|f| f.net_margin).map(|v| v * 100.0), |v| format!("{v:.1}%"),
        &[(20.0, true, "Exceptional margin. Pricing power and moat evident.", 2),
          (10.0, true, "Healthy margin. The business has some competitive protection.", 1),
          (5.0, true, "Thin. Commodity-like business.", 0)],
        ("Razor-thin or negative. No pricing power.", -1));

    // v11.6.B — persona astro_take rewrite. Buffett speaks ONLY about
    // his own framework (moat, owner earnings, price vs intrinsic value).
    // Astro score remains visible elsewhere as data; the persona never
    // narrates it. Keeps voice authentic.
    let astro_take = match score {
        s if s >= 5 => "Rule No. 1: never lose money. Rule No. 2: never forget Rule No. 1. The numbers here pass both tests.".to_string(),
        s if s >= 3 => "Price is what you pay, value is what you get. I'd want a wider margin, but the business itself is sound.".to_string(),
        s if s >= 0 => "Time is the friend of the wonderful business and the enemy of the mediocre. This one's borderline.".to_string(),
        s if s >= -2 => "Our favorite holding period is forever — but only when the business deserves it. This one doesn't, yet.".to_string(),
        _ => "It's only when the tide goes out that you discover who's been swimming naked. I'd rather not find out the hard way.".to_string(),
    };

    let verdict = score_to_verdict(score, &[(5, AgentVerdict::StrongBuy), (3, AgentVerdict::Buy),
        (0, AgentVerdict::Hold), (-2, AgentVerdict::Sell)]);
    let v = headline_variant(ctx, score, 6);
    let roe_pct = ctx.fundamentals.as_ref().and_then(|f| f.roe).map(|v| v * 100.0);
    let pe = ctx.fundamentals.as_ref().and_then(|f| f.pe_ratio);
    let headline = match verdict {
        AgentVerdict::StrongBuy => match v {
            0 => format!("{} looks like a wonderful business at a fair price.", ctx.ticker),
            1 => format!("{} is the kind of franchise I'd hold for a decade.", ctx.ticker),
            2 => match roe_pct { Some(r) if r > 20.0 => format!("{} earns {:.0}% on equity. Wonderful business.", ctx.ticker, r), _ => format!("{} prints cash. I'd back up the truck.", ctx.ticker) },
            3 => format!("Mr. Market is offering me {} cheaper than its intrinsic value.", ctx.ticker),
            4 => format!("{} has the moat, the margins, and the management. All three.", ctx.ticker),
            _ => format!("{} reminds me of See's Candies — quality with pricing power.", ctx.ticker),
        },
        AgentVerdict::Buy => match v {
            0 => format!("{} has solid economics. I'd consider adding to my position.", ctx.ticker),
            1 => format!("The fundamentals of {} tell a good story. Worth owning.", ctx.ticker),
            2 => format!("{} earns its keep. This is the kind of business I understand.", ctx.ticker),
            3 => match pe { Some(p) if p < 18.0 => format!("{} at {p:.0}x earnings is reasonable for the quality.", ctx.ticker), _ => format!("{} is a buy if you have a long enough holding period.", ctx.ticker) },
            4 => format!("I'd own {} in size — but only at this price or lower.", ctx.ticker),
            _ => format!("{} passes the 'would I buy the whole company' test.", ctx.ticker),
        },
        AgentVerdict::Hold => match v {
            0 => format!("{} is decent but not compelling. I'd hold, not add.", ctx.ticker),
            1 => format!("{} has some merit, but the price doesn't scream value to me.", ctx.ticker),
            2 => format!("I see what {} does well, but the margin of safety isn't there yet.", ctx.ticker),
            3 => format!("Holding {} makes sense. Buying more here doesn't.", ctx.ticker),
            4 => format!("{} is acceptable. Not the kind of business I'd seek out.", ctx.ticker),
            _ => format!("Mr. Market is pricing {} fairly today. I'd wait for a fat pitch.", ctx.ticker),
        },
        AgentVerdict::Sell => format!("{} doesn't meet my quality threshold. Capital is better deployed elsewhere.", ctx.ticker),
        AgentVerdict::StrongSell => format!("{} has fundamental problems. This is outside my circle of competence.", ctx.ticker),
        AgentVerdict::InsufficientData => format!("I need more data on {} before forming an opinion.", ctx.ticker),
    };

    assemble_analysis(ctx, AgentPersona::Buffett, metrics, score, astro_take, headline, verdict,
        build_buffett_narrative, "I can't analyze what I can't see. Fetch the fundamentals first.")
}

fn build_buffett_narrative(ctx: &AgentContext, score: i32) -> String {
    let Some(f) = ctx.fundamentals.as_ref() else { return String::new() };
    let mut parts = Vec::new();
    // Sector context
    if let Some(ref sector) = ctx.sector {
        match sector.as_str() {
            "Technology" => parts.push(format!("I used to avoid tech, but businesses like {} can have wide moats if the switching costs are high enough.", ctx.ticker)),
            "Financial Services" => parts.push("Banks require understanding leverage deeply. I look for conservative management and consistent ROE.".to_string()),
            "Healthcare" => parts.push("Healthcare moats come from patents, regulation, and switching costs. I focus on the durability of the franchise.".to_string()),
            "Consumer Defensive" | "Consumer Cyclical" => parts.push("Consumer brands are my sweet spot. The question is always: will people still want this in 20 years?".to_string()),
            _ => {}
        }
    }
    if let Some(roe) = f.roe {
        let pct = roe * 100.0;
        if pct > 20.0 { parts.push(format!("ROE of {pct:.0}% suggests a durable competitive advantage.")); }
        else if pct < 10.0 { parts.push(format!("ROE of {pct:.0}% is below my threshold. The business isn't earning its cost of capital.")); }
    }
    if let Some(de) = f.debt_equity {
        if de > 2.0 { parts.push(format!("Debt-to-equity of {de:.1} concerns me. Leverage amplifies both gains and losses.")); }
        else if de < 0.3 { parts.push("The balance sheet is a fortress. Very little debt.".to_string()); }
    }
    if let Some(fcf) = f.fcf {
        if fcf > 10_000_000_000 { parts.push(format!("Free cash flow of {} means the owner is getting real money back.", format_large_number(fcf))); }
        else if fcf < 0 { parts.push("Negative free cash flow is a red flag. The business consumes more cash than it generates.".to_string()); }
    }
    // News context
    if let Some(ref tone_label) = ctx.rss_tone_label {
        match tone_label.as_str() {
            "Bullish" | "Somewhat-Bullish" => parts.push("Recent news coverage is positive, but I focus on the business, not the news cycle. Headlines fade; fundamentals endure.".to_string()),
            "Bearish" | "Somewhat-Bearish" => parts.push("The headlines are unfavorable right now. In my experience, that's often when the best opportunities appear — be greedy when others are fearful.".to_string()),
            _ => {}
        }
    }
    if score >= 5 { parts.push("Overall, this has the hallmarks of a wonderful business. I'd be comfortable owning it for decades.".to_string()); }
    else if score <= -3 { parts.push("I'd pass on this one. There are better places to put capital.".to_string()); }
    if parts.is_empty() { "The metrics paint a mixed picture. Not compelling enough to act.".to_string() } else { parts.join(" ") }
}

// ---------------------------------------------------------------------------
// Graham — "In the short run, the market is a voting machine..."
// ---------------------------------------------------------------------------

fn analyze_graham(ctx: &AgentContext) -> AgentAnalysis {
    let f = ctx.fundamentals.as_ref();
    let mut metrics = Vec::new();
    let mut score = 0i32;

    eval_metric(&mut metrics, &mut score, "P/E Ratio",
        f.and_then(|f| f.pe_ratio), |v| format!("{v:.1}"),
        &[(0.0, false, "Negative earnings. Not suitable for value investing.", -3),
          (10.0, false, "Deep value. This is the kind of statistical cheapness I look for.", 3),
          (15.0, false, "Within my acceptable range. P/E under 15 is the minimum standard.", 2),
          (20.0, false, "Slightly above my threshold. Acceptable only with exceptional balance sheet.", 0)],
        ("Too expensive by my standards. Speculation, not investment.", -2));

    eval_metric(&mut metrics, &mut score, "P/B Ratio",
        f.and_then(|f| f.pb_ratio), |v| format!("{v:.2}"),
        &[(1.0, false, "Trading below book value. This is where fortunes are made.", 3),
          (1.5, false, "Within my threshold of 1.5x book.", 2),
          (3.0, false, "Above my ideal range. The margin of safety is thinner.", 0)],
        ("Far above book value. The market is pricing in perfection.", -2));

    // Graham Number: compound metric (P/E × P/B < 22.5) — can't use eval_metric
    if let (Some(pe), Some(pb)) = (f.and_then(|f| f.pe_ratio), f.and_then(|f| f.pb_ratio)) {
        if pe > 0.0 && pb > 0.0 {
            let product = pe * pb;
            let (assessment, pts) = if product < 22.5 {
                ("P/E x P/B under 22.5. Passes my combined test.", 2)
            } else {
                ("P/E x P/B exceeds 22.5. Fails my combined valuation test.", -1)
            };
            metrics.push(("P/E x P/B".to_string(), format!("{product:.1}"), assessment.to_string()));
            score += pts;
        }
    }

    eval_metric(&mut metrics, &mut score, "Current Ratio",
        f.and_then(|f| f.current_ratio), |v| format!("{v:.2}"),
        &[(2.0, true, "Strong liquidity. Can cover short-term obligations twice over.", 2),
          (1.5, true, "Adequate liquidity.", 1),
          (1.0, true, "Tight. Barely covering current liabilities.", -1)],
        ("Below 1.0. This company may struggle to pay its bills.", -2));

    eval_metric(&mut metrics, &mut score, "Dividend Yield",
        f.and_then(|f| f.dividend_yield).map(|v| v * 100.0), |v| format!("{v:.2}%"),
        &[(3.0, true, "Strong dividend. Tangible return to shareholders.", 2),
          (1.0, true, "Modest dividend. Shows commitment to returning capital.", 1),
          (0.0, true, "Token dividend.", 0)],
        ("No dividend. I prefer companies that share profits.", -1));

    // v11.6.B — Graham closing thought, pure quantitative voice.
    let astro_take = match score {
        s if s >= 7 => "The defensive investor's three rules: protect principal, accept adequate return, avoid speculation. This passes all three.".to_string(),
        s if s >= 4 => "An adequate margin of safety. Not exciting, but excitement is the enemy of returns.".to_string(),
        s if s >= 0 => "Mr. Market offers many prices each day. Today, this one isn't the bargain I'm looking for.".to_string(),
        s if s >= -3 => "An investment operation is one which, upon thorough analysis, promises safety of principal. This fails on safety.".to_string(),
        _ => "In the short run the market is a voting machine; in the long run, a weighing machine. The weight here is wanting.".to_string(),
    };

    let verdict = score_to_verdict(score, &[(7, AgentVerdict::StrongBuy), (4, AgentVerdict::Buy),
        (0, AgentVerdict::Hold), (-3, AgentVerdict::Sell)]);
    let v = headline_variant(ctx, score, 6);
    let pb = ctx.fundamentals.as_ref().and_then(|f| f.pb_ratio);
    let pe = ctx.fundamentals.as_ref().and_then(|f| f.pe_ratio);
    let headline = match verdict {
        AgentVerdict::StrongBuy => match v {
            0 => format!("{} is statistically cheap with a strong balance sheet. A textbook value investment.", ctx.ticker),
            1 => match (pe, pb) { (Some(e), Some(b)) => format!("{} at P/E {e:.0}, P/B {b:.1} — passes the 22.5 rule decisively.", ctx.ticker), _ => format!("{} is a net-net candidate. Buy with margin of safety.", ctx.ticker) },
            2 => format!("{} screens like the bargains I documented in Security Analysis.", ctx.ticker),
            3 => format!("The numbers for {} are unambiguous. This is deep value.", ctx.ticker),
            _ => format!("{} offers protection of capital and chance for profit. Both criteria met.", ctx.ticker),
        },
        AgentVerdict::Buy => match v {
            0 => format!("{} passes most of my quantitative screens. A reasonable investment.", ctx.ticker),
            1 => format!("The numbers for {} check out. Defensive investor territory.", ctx.ticker),
            2 => format!("{} offers adequate margin of safety at current prices.", ctx.ticker),
            3 => format!("{} qualifies under my enterprising-investor criteria.", ctx.ticker),
            4 => format!("Statistically, {} is in my buy zone. Not glamorous, but sound.", ctx.ticker),
            _ => format!("{} could fit a diversified portfolio of value names.", ctx.ticker),
        },
        AgentVerdict::Hold => match v {
            0 => format!("{} is neither cheap enough to buy nor expensive enough to sell.", ctx.ticker),
            1 => format!("Mr. Market is pricing {} fairly — no statistical bargain here.", ctx.ticker),
            2 => format!("{} sits in no-man's land: too expensive for value, too cheap to short.", ctx.ticker),
            3 => format!("{} doesn't insult my intelligence, but doesn't reward it either.", ctx.ticker),
            4 => format!("Hold on {}. The margin of safety is razor-thin.", ctx.ticker),
            _ => format!("{} fails the 'would I lend money against this' test by a hair.", ctx.ticker),
        },
        AgentVerdict::Sell => format!("{} fails my valuation criteria. The margin of safety is inadequate.", ctx.ticker),
        AgentVerdict::StrongSell => format!("{} is speculative at this price. No margin of safety.", ctx.ticker),
        AgentVerdict::InsufficientData => format!("I need financial statements for {} before I can run my screens.", ctx.ticker),
    };

    assemble_analysis(ctx, AgentPersona::Graham, metrics, score, astro_take, headline, verdict,
        build_graham_narrative, "Without financial data, I cannot perform my quantitative analysis.")
}

fn build_graham_narrative(ctx: &AgentContext, score: i32) -> String {
    let Some(f) = ctx.fundamentals.as_ref() else { return String::new() };
    let mut parts = Vec::new();
    if let (Some(pe), Some(pb)) = (f.pe_ratio, f.pb_ratio) {
        if pe > 0.0 && pb > 0.0 && pe * pb < 22.5 {
            parts.push(format!("The Graham Number test passes: P/E ({pe:.1}) x P/B ({pb:.1}) = {:.1}, under my 22.5 threshold.", pe * pb));
        } else if pe > 0.0 && pb > 0.0 {
            parts.push(format!("P/E ({pe:.1}) x P/B ({pb:.1}) = {:.1}, which exceeds my 22.5 ceiling. The stock is priced for growth, not value.", pe * pb));
        }
    }
    if let Some(cr) = f.current_ratio {
        if cr < 1.5 { parts.push(format!("Current ratio of {cr:.2} is below my 2.0 minimum. Liquidity risk.")); }
    }
    // News context — Graham is skeptical
    if !ctx.recent_headlines.is_empty() {
        let headline = &ctx.recent_headlines[0];
        if headline.len() > 10 {
            parts.push(format!("The headlines say '{}' — Mr. Market is excitable. I trust the balance sheet over the news ticker.", truncate_str(headline, 60)));
        }
    }
    if score >= 7 { parts.push("This passes my full battery of quantitative tests. It has the statistical profile of a sound investment.".to_string()); }
    else if score <= -3 { parts.push("The numbers don't lie. This fails on multiple quantitative criteria. Pass.".to_string()); }
    if parts.is_empty() { "The quantitative picture is mixed. Not a clear buy or sell.".to_string() } else { parts.join(" ") }
}

// ---------------------------------------------------------------------------
// Lynch — "Know what you own, and know why you own it"
// ---------------------------------------------------------------------------

fn analyze_lynch(ctx: &AgentContext) -> AgentAnalysis {
    let f = ctx.fundamentals.as_ref();
    let mut metrics = Vec::new();
    let mut score = 0i32;

    eval_metric(&mut metrics, &mut score, "PEG Ratio",
        f.and_then(|f| f.peg_ratio), |v| format!("{v:.2}"),
        &[(0.0, false, "Negative PEG means negative earnings growth. That's not growth investing.", -2),
          (1.0, false, "PEG under 1.0! The market hasn't fully priced in the growth. This is my sweet spot.", 3),
          (1.5, false, "PEG around 1.0-1.5. Fairly valued for its growth rate.", 1),
          (2.5, false, "Getting pricey relative to growth. The growth better be real.", 0)],
        ("Way too expensive for the growth you're getting.", -2));

    eval_metric(&mut metrics, &mut score, "P/E Ratio",
        f.and_then(|f| f.pe_ratio), |v| format!("{v:.1}"),
        &[(0.0, false, "No earnings. Can't be a GARP investment without the 'E'.", -2),
          (20.0, false, "Reasonable P/E. If this company is growing, it could be a bargain.", 2),
          (35.0, false, "Moderate P/E. Fine if the growth rate is comparable.", 1),
          (50.0, false, "High P/E. The growth story needs to be very compelling.", 0)],
        ("Nosebleed territory. A lot of future growth is already priced in.", -1));

    eval_metric(&mut metrics, &mut score, "Revenue",
        f.and_then(|f| f.revenue).map(|v| v as f64), |v| format_large_number(v as i64),
        &[(100_000_000_000.0, true, "Mega-cap revenue. Growth will be harder from here, but the moat is real.", 1),
          (10_000_000_000.0, true, "Large business. Still room to grow if the story is right.", 1),
          (1_000_000_000.0, true, "Mid-size. This is where the best growth stocks live.", 2)],
        ("Small revenue base. Could be a multi-bagger if the product is right.", 1));

    eval_metric(&mut metrics, &mut score, "Net Margin",
        f.and_then(|f| f.net_margin).map(|v| v * 100.0), |v| format!("{v:.1}%"),
        &[(15.0, true, "Strong margins. The business is printing money.", 2),
          (5.0, true, "Decent margins. Room to expand with scale.", 1),
          (0.0, true, "Thin margins. Watch for margin compression.", 0)],
        ("Losing money. This isn't GARP, it's hope.", -2));

    eval_metric(&mut metrics, &mut score, "Debt/Equity",
        f.and_then(|f| f.debt_equity), |v| format!("{v:.2}"),
        &[(0.5, false, "Clean balance sheet. Growth funded organically.", 1),
          (1.5, false, "Moderate leverage. Normal for most industries.", 0)],
        ("Heavy debt. If growth stalls, this leverage becomes a problem.", -1));

    // v11.6.B — Lynch closing thought, voice anchored on his actual rules.
    let astro_take = match score {
        s if s >= 7 => "The simpler the story, the better. I can summarize this one in two sentences — that's a buying sign.".to_string(),
        s if s >= 4 => "The investor's chief problem — and even his worst enemy — is likely to be himself. The numbers here are decent.".to_string(),
        s if s >= 1 => "Your investor's edge is not something you get from Wall Street experts. It's something you already have. Look at this one yourself.".to_string(),
        s if s >= -2 => "If you can't explain to a 10-year-old why you own a stock, you shouldn't own it. This one's hard to explain.".to_string(),
        _ => "Selling your winners and holding your losers is like cutting the flowers and watering the weeds. This one's a weed.".to_string(),
    };

    let verdict = score_to_verdict(score, &[(7, AgentVerdict::StrongBuy), (4, AgentVerdict::Buy),
        (1, AgentVerdict::Hold), (-2, AgentVerdict::Sell)]);
    let v = headline_variant(ctx, score, 6);
    let peg = ctx.fundamentals.as_ref().and_then(|f| f.peg_ratio);
    let sector_short = ctx.sector.as_deref().unwrap_or("");
    let headline = match verdict {
        AgentVerdict::StrongBuy => match v {
            0 => format!("{} is growing fast and the market hasn't fully priced it in. Classic GARP.", ctx.ticker),
            1 => match peg { Some(p) if p < 1.0 => format!("{} at PEG {p:.2}? The market is asleep on this one.", ctx.ticker), _ => format!("{} is a tenbagger candidate hiding in plain sight.", ctx.ticker) },
            2 => format!("{} is the kind of {} story I'd buy and forget for five years.", ctx.ticker, if sector_short.is_empty() { "growth".into() } else { sector_short.to_lowercase() }),
            3 => format!("Walk into a {} retailer — you'll see why this is a tenbagger.", ctx.ticker),
            4 => format!("{} is fast-grower territory at value-investor prices. Rare combo.", ctx.ticker),
            _ => format!("Buy {}, hold it, ignore the chatter. Growth wins.", ctx.ticker),
        },
        AgentVerdict::Buy => match v {
            0 => format!("{} has a good growth story at a reasonable price.", ctx.ticker),
            1 => format!("I like what I see in {}. Growth is real and the price is fair.", ctx.ticker),
            2 => format!("{} is the kind of stock I'd find walking through a mall.", ctx.ticker),
            3 => format!("{} fits my GARP screen. The 'G' and the 'P' both check out.", ctx.ticker),
            4 => format!("If {} can keep this growth rate, today's price is a steal.", ctx.ticker),
            _ => format!("{} — I'd own this in a growth allocation, not a value one.", ctx.ticker),
        },
        AgentVerdict::Hold => match v {
            0 => format!("{} is interesting but the price-to-growth ratio isn't compelling yet.", ctx.ticker),
            1 => format!("I'd want to walk into a {} store before committing.", ctx.ticker),
            2 => format!("{} has potential, but the growth rate must accelerate first.", ctx.ticker),
            3 => format!("{} is a stalwart for now. Watch for a fast-grower phase.", ctx.ticker),
            4 => format!("I'd hold {} and wait for the next earnings catalyst.", ctx.ticker),
            _ => format!("{} — neither cheap enough nor growing fast enough. Hold.", ctx.ticker),
        },
        AgentVerdict::Sell => format!("{} is either too expensive for its growth or the growth isn't there.", ctx.ticker),
        AgentVerdict::StrongSell => format!("{} fails my growth-at-reasonable-price test on multiple fronts.", ctx.ticker),
        AgentVerdict::InsufficientData => format!("I need to see the numbers for {} before I can form an opinion.", ctx.ticker),
    };

    assemble_analysis(ctx, AgentPersona::Lynch, metrics, score, astro_take, headline, verdict,
        build_lynch_narrative, "I can't tell you if it's a good stock without seeing the business.")
}

fn build_lynch_narrative(ctx: &AgentContext, score: i32) -> String {
    let Some(f) = ctx.fundamentals.as_ref() else { return String::new() };
    let mut parts = Vec::new();
    // Sector categorization — Lynch loved classifying stocks
    if let Some(ref industry) = ctx.industry {
        let category = classify_lynch_category(f);
        parts.push(format!("I'd classify {} in {industry} as a {category}.", ctx.ticker));
    }
    if let Some(peg) = f.peg_ratio {
        if peg > 0.0 && peg < 1.0 { parts.push(format!("PEG of {peg:.2} is the standout here. The market is pricing in less growth than the company is delivering.")); }
        else if peg > 2.0 { parts.push(format!("PEG of {peg:.2} means you're paying a premium for growth. Make sure you understand what's driving it.")); }
    }
    // News awareness
    if let Some(ref tone_label) = ctx.rss_tone_label {
        if tone_label == "Bullish" || tone_label == "Somewhat-Bullish" {
            parts.push("The news is favorable right now. Good — but I'd still want to understand the business before the stock.".to_string());
        } else if tone_label == "Bearish" || tone_label == "Somewhat-Bearish" {
            parts.push("Negative press can create buying opportunities if the business is sound. I'd investigate what's behind the headlines.".to_string());
        }
    }
    if score >= 5 { parts.push("This has the profile of a growth stock at a reasonable price. The kind I'd want to own.".to_string()); }
    else if score <= -2 { parts.push("I'd pass here. There are better growth stories at better prices.".to_string()); }
    if parts.is_empty() { "The growth-value picture is mixed. I'd want to dig deeper into the actual business.".to_string() } else { parts.join(" ") }
}

/// Lynch's stock categories based on fundamental profile.
fn classify_lynch_category(f: &FundamentalMetric) -> &'static str {
    let growth_hint = f.peg_ratio.unwrap_or(2.0);
    let pe = f.pe_ratio.unwrap_or(20.0);
    let div = f.dividend_yield.unwrap_or(0.0);
    if growth_hint < 1.0 && pe < 25.0 { "fast grower" }
    else if pe < 15.0 && div > 0.02 { "stalwart" }
    else if pe > 40.0 { "speculative grower" }
    else if div > 0.04 { "slow grower" }
    else { "stalwart" }
}

// ---------------------------------------------------------------------------
// Munger — "All I want to know is where I'm going to die, so I'll never go there"
// ---------------------------------------------------------------------------

fn analyze_munger(ctx: &AgentContext) -> AgentAnalysis {
    let f = ctx.fundamentals.as_ref();
    let mut metrics = Vec::new();
    let mut score = 0i32;

    eval_metric(&mut metrics, &mut score, "ROE",
        f.and_then(|f| f.roe).map(|v| v * 100.0), |v| format!("{v:.1}%"),
        &[(25.0, true, "Outstanding. This business has a wide moat.", 3),
          (15.0, true, "Good returns on equity. Competitive position is solid.", 2),
          (8.0, true, "Average. Not the kind of quality I gravitate toward.", 0)],
        ("Poor capital allocation. Management should return the capital to shareholders.", -2));

    eval_metric(&mut metrics, &mut score, "Operating Margin",
        f.and_then(|f| f.operating_margin).map(|v| v * 100.0), |v| format!("{v:.1}%"),
        &[(30.0, true, "Exceptional pricing power. The moat is deep.", 3),
          (15.0, true, "Solid operations. Some competitive protection.", 1),
          (5.0, true, "Thin margins. Commoditized business.", 0)],
        ("No operating leverage. A dollar in, barely a dime out.", -2));

    eval_metric(&mut metrics, &mut score, "EV/EBITDA",
        f.and_then(|f| f.ev_ebitda), |v| format!("{v:.1}"),
        &[(0.0, false, "Negative EBITDA. Not investable.", -2),
          (10.0, false, "Cheap by enterprise value. Is the quality here too?", 1),
          (20.0, false, "Fair enterprise value.", 0)],
        ("Premium valuation. The quality needs to be exceptional to justify this.", -1));

    eval_metric(&mut metrics, &mut score, "FCF",
        f.and_then(|f| f.fcf).map(|v| v as f64), |v| format_large_number(v as i64),
        &[(5_000_000_000.0, true, "Excellent cash generation. This business funds its own growth.", 2),
          (0.0, true, "Positive cash flow. The business sustains itself.", 1)],
        ("Cash burn. Avoid.", -2));

    eval_metric(&mut metrics, &mut score, "Debt/Equity",
        f.and_then(|f| f.debt_equity), |v| format!("{v:.2}"),
        &[(0.3, false, "Almost no debt. The best kind of balance sheet.", 2),
          (1.0, false, "Manageable leverage.", 1),
          (2.0, false, "I'd want to understand why they need this much debt.", -1)],
        ("Excessive leverage. Three things ruin smart people: liquor, ladies, and leverage.", -2));

    // v11.6.B — Munger closing thought, mental-model voice. Diversified
    // across score band + ticker hash so consecutive tickers don't echo.
    let mv = headline_variant(ctx, score, 4);
    let astro_take = match (score, mv) {
        (s, _) if s >= 8 => "Take a simple idea and take it seriously. Quality + reasonable price + competent management. This one has all three.".to_string(),
        (s, 0) if s >= 5 => "The big money is not in the buying or the selling, but in the waiting. This is a wait-and-compound name.".to_string(),
        (s, _) if s >= 5 => "If you are going to invest in stocks for the long term, you can take advantage of stock declines. Add on weakness.".to_string(),
        (s, 0) if s >= 1 => "Knowing what you don't know is more useful than being brilliant. I don't fully know this one yet.".to_string(),
        (s, 1) if s >= 1 => "It's remarkable how much long-term advantage people have gotten by trying to be consistently not stupid. Hold and watch.".to_string(),
        (s, _) if s >= 1 => "A great business at a fair price is superior to a fair business at a great price. This is fair-business territory.".to_string(),
        (s, 0) if s >= -2 => "Three things ruin people: drugs, liquor, and leverage. Two of those show up here.".to_string(),
        (s, _) if s >= -2 => "Invert, always invert. What would make this position fail? Too many things.".to_string(),
        (_, 0) => "It is remarkable how much long-term advantage people have gotten by trying to be consistently not stupid. Avoiding this is the smart play.".to_string(),
        _ => "There are worse situations than drowning in cash and sitting in your own private aviation. Pass on this one.".to_string(),
    };

    let verdict = score_to_verdict(score, &[(8, AgentVerdict::StrongBuy), (5, AgentVerdict::Buy),
        (1, AgentVerdict::Hold), (-2, AgentVerdict::Sell)]);
    let v = headline_variant(ctx, score, 6);
    let om = ctx.fundamentals.as_ref().and_then(|f| f.operating_margin).map(|v| v * 100.0);
    let headline = match verdict {
        AgentVerdict::StrongBuy => match v {
            0 => format!("{} is a high-quality business with a durable moat. I'd hold forever.", ctx.ticker),
            1 => match om { Some(m) if m > 30.0 => format!("{} runs at {:.0}% operating margin. That's not luck — that's a moat.", ctx.ticker, m), _ => format!("{} has the rare combination: quality and reasonable price.", ctx.ticker) },
            2 => format!("{} clears my quality bar. The world's wonderful businesses are few.", ctx.ticker),
            3 => format!("{} is the kind of compounder Warren and I would buy outright.", ctx.ticker),
            4 => format!("Quality, moat, management — {} has all three. Rare.", ctx.ticker),
            _ => format!("{} — invert: what would make me sell? Almost nothing comes to mind.", ctx.ticker),
        },
        AgentVerdict::Buy => match v {
            0 => format!("{} shows good quality indicators. Worth owning.", ctx.ticker),
            1 => format!("The economics of {} are sound. Not perfect, but the moat is real.", ctx.ticker),
            2 => format!("{} reminds me of businesses I've admired. Quality deserves a fair price.", ctx.ticker),
            3 => format!("{} has the right architectural traits — I'd hold a position.", ctx.ticker),
            4 => format!("{} is good quality at a fair price. Buffett's formula.", ctx.ticker),
            _ => format!("Owning {} won't keep me up at night. That's the test.", ctx.ticker),
        },
        AgentVerdict::Hold => match v {
            0 => format!("{} is acceptable but not the kind of quality that excites me.", ctx.ticker),
            1 => format!("I don't hate {} but I don't love it either. 'Okay' isn't good enough.", ctx.ticker),
            2 => format!("{} is mediocre. The world is full of mediocre businesses I don't need to own.", ctx.ticker),
            3 => format!("{} is in my too-hard pile. Move on.", ctx.ticker),
            4 => format!("{} — not bad, not wonderful. Quality compounders are rare.", ctx.ticker),
            _ => format!("Hold {} if you must. I'd rather wait for a fat pitch.", ctx.ticker),
        },
        AgentVerdict::Sell => format!("{} doesn't meet my quality bar. I'd rather own a wonderful business at a fair price.", ctx.ticker),
        AgentVerdict::StrongSell => format!("{} shows multiple quality red flags. Invert: what would make this fail? Too much.", ctx.ticker),
        AgentVerdict::InsufficientData => format!("I need to see the economics of {} before I can form a judgment.", ctx.ticker),
    };

    assemble_analysis(ctx, AgentPersona::Munger, metrics, score, astro_take, headline, verdict,
        build_munger_narrative, "Show me the business economics. Then we can talk.")
}

fn build_munger_narrative(ctx: &AgentContext, score: i32) -> String {
    let Some(f) = ctx.fundamentals.as_ref() else { return String::new() };
    let mut parts = Vec::new();
    // Sector-specific mental models
    if let Some(ref sector) = ctx.sector {
        let model = match sector.as_str() {
            "Technology" => "network effects and switching costs",
            "Financial Services" => "leverage and trust",
            "Healthcare" => "regulatory moats and patent cliffs",
            "Energy" => "commodity cycles and capital intensity",
            "Consumer Defensive" => "brand loyalty and habitual consumption",
            "Industrials" => "scale economics and installed base",
            _ => "competitive dynamics and durable advantages",
        };
        parts.push(format!("In this sector, the key mental model is {model}. Let me evaluate {} through that lens.", ctx.ticker));
    }
    if let Some(roe) = f.roe {
        let pct = roe * 100.0;
        if pct > 25.0 { parts.push(format!("ROE of {pct:.0}% is exceptional. When you see this consistently, there's usually a moat.")); }
    }
    if let Some(om) = f.operating_margin {
        let pct = om * 100.0;
        if pct > 30.0 { parts.push(format!("Operating margin of {pct:.0}% shows real pricing power. Competitors can't touch this.")); }
        else if pct < 5.0 { parts.push("Thin operating margin suggests a commoditized business. No pricing power.".to_string()); }
    }
    // News as signal vs noise
    if !ctx.recent_headlines.is_empty() {
        if let Some(ref tone_label) = ctx.rss_tone_label {
            match tone_label.as_str() {
                "Bearish" | "Somewhat-Bearish" => parts.push("The news cycle is negative. Remember: in the short run, the market is a voting machine. In the long run, a weighing machine. I weigh.".to_string()),
                "Bullish" => parts.push("Positive sentiment is nice, but I've seen enthusiasm destroy more wealth than pessimism. The question remains: what is the quality of this business?".to_string()),
                _ => {}
            }
        }
    }
    if score >= 8 { parts.push("This is the kind of quality business I want to own forever. The key question is price.".to_string()); }
    else if score <= -2 { parts.push("Invert, always invert. What would destroy this investment? Too many things. Pass.".to_string()); }
    if parts.is_empty() { "The quality picture is mixed. I'd keep watching but not act.".to_string() } else { parts.join(" ") }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Pick a headline variant index using ticker chars + score so different
/// tickers (and same ticker at different scores) get different phrasings.
/// v11.3 — replaces simpler `ticker.len() % 3` to cut "same template" feel.
fn headline_variant(ctx: &AgentContext, score: i32, pool_size: usize) -> usize {
    let char_sum: u32 = ctx.ticker.chars().map(|c| c as u32).sum();
    ((char_sum.wrapping_add(score.unsigned_abs())) as usize) % pool_size
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max { s.to_string() }
    else { format!("{}...", s.chars().take(max).collect::<String>()) }
}

fn format_large_number(n: i64) -> String {
    let f = n as f64;
    if f.abs() >= 1_000_000_000.0 { format!("${:.1}B", f / 1_000_000_000.0) }
    else if f.abs() >= 1_000_000.0 { format!("${:.1}M", f / 1_000_000.0) }
    else { format!("${f:.0}") }
}

/// When fundamentals are missing, generate a partial analysis from available
/// signals (astro, lagrange, price, moon phase). Returns (analysis, verdict, metrics).
/// Returns None if there's truly nothing to work with.
fn no_fundamentals_fallback(
    ctx: &AgentContext,
    persona: AgentPersona,
) -> Option<(String, AgentVerdict, Vec<(String, String, String)>)> {
    let has_astro = ctx.astro_score.is_some();
    let has_lagrange = ctx.lagrange_score.is_some();
    let has_price = ctx.current_price.is_some();

    if !has_astro && !has_lagrange && !has_price {
        return None;
    }

    let mut metrics = Vec::new();
    let mut points = 0i32;

    if let Some(astro) = ctx.astro_score {
        let (assessment, pts) = if astro > 75.0 {
            ("Strongly favorable astrological alignment.", 2)
        } else if astro > 55.0 {
            ("Moderately favorable conditions.", 1)
        } else if astro > 40.0 {
            ("Neutral astrological positioning.", 0)
        } else {
            ("Unfavorable alignment. Caution warranted.", -1)
        };
        metrics.push(("Astro Score".to_string(), format!("{astro:.0}"), assessment.to_string()));
        points += pts;
    }

    if let (Some(score), Some(label)) = (ctx.lagrange_score, ctx.lagrange_label.as_deref()) {
        let pts = if score > 70.0 { 2 } else if score > 55.0 { 1 } else if score > 40.0 { 0 } else { -1 };
        metrics.push(("Lagrange Score".to_string(), format!("{score:.0} ({label})"), {
            if score > 70.0 { "Composite signal is strong." }
            else if score > 55.0 { "Composite signal is positive." }
            else if score > 40.0 { "Mixed signals." }
            else { "Composite signal is weak." }
        }.to_string()));
        points += pts;
    }

    if let Some(price) = ctx.current_price {
        metrics.push(("Price".to_string(), format!("${price:.2}"), "No fundamental context to assess valuation.".to_string()));
    }

    if ctx.mercury_rx {
        metrics.push(("Mercury".to_string(), "Retrograde".to_string(), "Communications disrupted. Caution on new positions.".to_string()));
        points -= 1;
    }

    if let Some(phase) = ctx.moon_phase.as_deref() {
        metrics.push(("Moon Phase".to_string(), phase.to_string(), String::new()));
    }

    let verdict = match points {
        3.. => AgentVerdict::Buy,
        1..=2 => AgentVerdict::Hold,
        _ => AgentVerdict::Sell,
    };

    // v11.6.B — fallback notes also strip astro refs.
    let persona_note = match persona {
        AgentPersona::Buffett => "Without financials, I'm flying blind. I invest in businesses, not stories. Run the scraper and let me see the numbers — then we can talk.",
        AgentPersona::Graham => "My quantitative screens require financial data. Without P/E, P/B, current ratio, and dividend history, there's nothing for me to evaluate.",
        AgentPersona::Lynch => "Know what you own. I can't tell you what to own without seeing what the company actually does and how it's growing.",
        AgentPersona::Munger => "Show me the business economics. Show me the return on equity, the operating margin, the management's track record. Then we can talk.",
    };

    let analysis = format!(
        "{} {}",
        persona_note,
        if points >= 2 { "The available signals lean positive." }
        else if points >= 0 { "The signals are mixed. No strong conviction either way." }
        else { "The available signals lean cautious." }
    );

    Some((analysis, verdict, metrics))
}

// ---------------------------------------------------------------------------
// LLM-backed analysis via Anthropic Claude API
// ---------------------------------------------------------------------------

/// Build the system prompt for a persona + financial context.
fn build_system_prompt(persona: AgentPersona, ctx: &AgentContext) -> String {
    // v11.6.B — persona prompts speak ONLY about the persona's own
    // framework. Astrology is provided as background data context but the
    // persona must NOT mention astrology, planets, stars, horoscope,
    // zodiac, etc. in the response prose. They evaluate the ticker
    // through their own discipline.
    let persona_desc = match persona {
        AgentPersona::Buffett => "\
You are Warren Buffett, the Oracle of Omaha. You evaluate companies through the lens of durable \
competitive moats, free cash flow generation, and margin of safety. You are patient, folksy, and \
speak in plain language. You care about return on equity, debt levels, and whether you'd be happy \
owning the entire business for a decade. \
IMPORTANT: do NOT mention astrology, planets, stars, horoscopes, or any cosmic/celestial language \
in your response. Speak strictly in your own financial framework.",
        AgentPersona::Graham => "\
You are Benjamin Graham, the father of value investing. You are methodical, quantitative, and \
skeptical of anything that cannot be measured. You focus on P/E under 15, P/B under 1.5, the \
Graham Number, current ratio above 2, and dividend history. You never overpay, even for quality. \
IMPORTANT: do NOT mention astrology, planets, stars, horoscopes, or any cosmic/celestial language \
in your response. Speak strictly in your quantitative discipline.",
        AgentPersona::Lynch => "\
You are Peter Lynch, the legendary Fidelity Magellan fund manager. Your mantra is 'know what you \
own and why you own it.' You love the PEG ratio, growth at a reasonable price, and companies whose \
products you can see and touch. You are enthusiastic, practical, and classify stocks into \
categories. \
IMPORTANT: do NOT mention astrology, planets, stars, horoscopes, or any cosmic/celestial language \
in your response. Speak strictly about the business and its growth story.",
        AgentPersona::Munger => "\
You are Charlie Munger, Warren Buffett's partner and a polymath investor. You think in mental \
models drawn from psychology, physics, and biology. You focus on business quality over price: high \
return on equity, wide operating margins, and rational management. You are blunt, witty, and \
allergic to foolishness. \
IMPORTANT: do NOT mention astrology, planets, stars, horoscopes, or any cosmic/celestial language \
in your response. Speak strictly through mental models and business economics.",
    };

    let context_block = format_context_for_llm(ctx);

    format!(
        "{persona_desc}\n\n\
         ## Financial Data for {ticker}\n\n\
         {context_block}\n\n\
         ## Response Format\n\n\
         Respond with a JSON object (no markdown fences) containing:\n\
         - \"headline\": one-line summary, max 120 characters\n\
         - \"analysis\": 3-5 sentence analysis in character (NO astrology references)\n\
         - \"verdict\": one of \"StrongBuy\", \"Buy\", \"Hold\", \"Sell\", \"StrongSell\"\n\
         - \"key_metrics\": array of [metric_name, value, assessment] arrays (3-6 items)\n\
         - \"astro_take\": your in-character closing thought, 1-2 sentences. This must ALSO avoid \
            astrology references — speak in your own framework's voice as a final remark.\n\n\
         Stay in character throughout. Do not break persona. Do not mention astrology, planets, \
         stars, horoscopes, or zodiac signs anywhere in the response.",
        ticker = ctx.ticker
    )
}

/// Serialize AgentContext fields to readable text for the LLM prompt.
fn format_context_for_llm(ctx: &AgentContext) -> String {
    let mut lines = vec![format!("Ticker: {}", ctx.ticker)];

    if let Some(price) = ctx.current_price {
        lines.push(format!("Current Price: ${price:.2}"));
    }
    if let Some(astro) = ctx.astro_score {
        lines.push(format!("Astro Score: {astro:.1}/100"));
    }
    if let Some(ref label) = ctx.astro_label {
        lines.push(format!("Astro Zone: {label}"));
    }
    if let Some(score) = ctx.lagrange_score {
        lines.push(format!("Lagrange Composite Score: {score:.1}"));
    }
    if let Some(ref label) = ctx.lagrange_label {
        lines.push(format!("Lagrange Zone: {label}"));
    }
    if let Some(ref conc) = ctx.concordance {
        lines.push(format!("Concordance (Astro-Financial): {conc}"));
    }
    if ctx.mercury_rx {
        lines.push("Mercury Retrograde: ACTIVE (caution on communications/contracts)".to_string());
    }
    if let Some(ref phase) = ctx.moon_phase {
        lines.push(format!("Moon Phase: {phase}"));
    }
    if let Some(ref theme) = ctx.dominant_theme {
        lines.push(format!("Dominant Astrological Theme: {theme}"));
    }

    // v10.0 context enrichment
    if let Some(ref sector) = ctx.sector {
        lines.push(format!("Sector: {sector}"));
    }
    if let Some(ref industry) = ctx.industry {
        lines.push(format!("Industry: {industry}"));
    }
    if let Some(tone) = ctx.rss_tone_score {
        let label = ctx.rss_tone_label.as_deref().unwrap_or("—");
        lines.push(format!("RSS Tone Sentiment: {label} ({tone:+.2})"));
    }
    if !ctx.recent_headlines.is_empty() {
        lines.push("\n--- Recent Headlines ---".to_string());
        for h in &ctx.recent_headlines {
            lines.push(format!("- {h}"));
        }
    }

    if let Some(ref f) = ctx.fundamentals {
        lines.push("\n--- Fundamental Metrics ---".to_string());
        if let Some(v) = f.market_cap { lines.push(format!("Market Cap: ${v}")); }
        if let Some(v) = f.pe_ratio { lines.push(format!("P/E Ratio: {v:.2}")); }
        if let Some(v) = f.pb_ratio { lines.push(format!("P/B Ratio: {v:.2}")); }
        if let Some(v) = f.ps_ratio { lines.push(format!("P/S Ratio: {v:.2}")); }
        if let Some(v) = f.ev_ebitda { lines.push(format!("EV/EBITDA: {v:.2}")); }
        if let Some(v) = f.peg_ratio { lines.push(format!("PEG Ratio: {v:.2}")); }
        if let Some(v) = f.roe { lines.push(format!("ROE: {v:.1}%")); }
        if let Some(v) = f.roa { lines.push(format!("ROA: {v:.1}%")); }
        if let Some(v) = f.net_margin { lines.push(format!("Net Margin: {v:.1}%")); }
        if let Some(v) = f.operating_margin { lines.push(format!("Operating Margin: {v:.1}%")); }
        if let Some(v) = f.debt_equity { lines.push(format!("Debt/Equity: {v:.2}")); }
        if let Some(v) = f.current_ratio { lines.push(format!("Current Ratio: {v:.2}")); }
        if let Some(v) = f.fcf { lines.push(format!("Free Cash Flow: ${v}")); }
        if let Some(v) = f.revenue { lines.push(format!("Revenue: ${v}")); }
        if let Some(v) = f.net_income { lines.push(format!("Net Income: ${v}")); }
        if let Some(v) = f.eps { lines.push(format!("EPS: ${v:.2}")); }
        if let Some(v) = f.dividend_yield { lines.push(format!("Dividend Yield: {v:.2}%")); }
    } else {
        lines.push("\nFundamental data not available. Analyze based on astro/price signals.".to_string());
    }

    lines.join("\n")
}

/// Call Anthropic Claude API for LLM-backed agent analysis.
pub async fn analyze_llm(
    persona: AgentPersona,
    ctx: AgentContext,
    api_key: String,
) -> Result<AgentAnalysis, String> {
    let system_prompt = build_system_prompt(persona, &ctx);

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "claude-sonnet-4-20250514",
        "max_tokens": 1024,
        "system": system_prompt,
        "messages": [{
            "role": "user",
            "content": format!(
                "Analyze {} and provide your investment assessment as {}.",
                ctx.ticker, persona.name()
            )
        }]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let err_body = response.text().await.unwrap_or_default();
        return Err(format!("API error {status}: {}", err_body.chars().take(200).collect::<String>()));
    }

    let resp: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;

    parse_llm_response(persona, &resp)
}

/// Parse Claude's response JSON into an AgentAnalysis.
fn parse_llm_response(
    persona: AgentPersona,
    resp: &serde_json::Value,
) -> Result<AgentAnalysis, String> {
    let text = resp["content"][0]["text"]
        .as_str()
        .ok_or_else(|| "No text content in API response".to_string())?;

    // Try direct JSON parse, then fall back to extracting JSON from markdown
    let parsed: serde_json::Value = serde_json::from_str(text)
        .or_else(|_| {
            let start = text.find('{').ok_or("No JSON object found in response")?;
            let end = text.rfind('}').ok_or("No closing brace found")? + 1;
            serde_json::from_str(&text[start..end])
                .map_err(|e| format!("JSON parse: {e}"))
        })
        .map_err(|e: String| format!("Failed to parse LLM response: {e}"))?;

    let verdict = match parsed["verdict"].as_str().unwrap_or("Hold") {
        "StrongBuy" => AgentVerdict::StrongBuy,
        "Buy" => AgentVerdict::Buy,
        "Sell" => AgentVerdict::Sell,
        "StrongSell" => AgentVerdict::StrongSell,
        _ => AgentVerdict::Hold,
    };

    let key_metrics = parsed["key_metrics"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let a = item.as_array()?;
                    Some((
                        a.first()?.as_str()?.to_string(),
                        a.get(1)?.as_str()?.to_string(),
                        a.get(2)?.as_str()?.to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(AgentAnalysis {
        persona,
        headline: parsed["headline"].as_str().unwrap_or("LLM analysis complete").to_string(),
        analysis: parsed["analysis"].as_str().unwrap_or(text).to_string(),
        verdict,
        key_metrics,
        astro_take: parsed["astro_take"].as_str().unwrap_or("").to_string(),
    })
}
