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
    let mut score = 0i32; // internal scoring for verdict

    // ROE — Buffett wants > 15%
    if let Some(roe) = f.and_then(|f| f.roe) {
        let pct = roe * 100.0;
        let (assessment, pts) = if pct > 20.0 {
            ("Excellent. Durable competitive advantage likely.", 2)
        } else if pct > 15.0 {
            ("Good. Meets my minimum threshold.", 1)
        } else if pct > 10.0 {
            ("Mediocre. Not the kind of business I prefer.", 0)
        } else {
            ("Poor. This business doesn't earn its cost of capital.", -2)
        };
        metrics.push(("ROE".to_string(), format!("{pct:.1}%"), assessment.to_string()));
        score += pts;
    }

    // Debt/Equity — Buffett prefers low leverage
    if let Some(de) = f.and_then(|f| f.debt_equity) {
        let (assessment, pts) = if de < 0.5 {
            ("Conservative balance sheet. I like that.", 2)
        } else if de < 1.0 {
            ("Reasonable leverage.", 1)
        } else if de < 2.0 {
            ("Getting leveraged. Proceed with caution.", -1)
        } else {
            ("Too much debt. Debt destroys value in downturns.", -2)
        };
        metrics.push(("Debt/Equity".to_string(), format!("{de:.2}"), assessment.to_string()));
        score += pts;
    }

    // FCF — Buffett loves free cash flow
    if let Some(fcf) = f.and_then(|f| f.fcf) {
        let (assessment, pts) = if fcf > 10_000_000_000 {
            ("Massive cash generation. This is a cash machine.", 2)
        } else if fcf > 1_000_000_000 {
            ("Strong cash flow. The business is real.", 1)
        } else if fcf > 0 {
            ("Positive FCF, but not yet compelling.", 0)
        } else {
            ("Negative FCF. This business is burning cash, not generating it.", -2)
        };
        let fcf_str = format_large_number(fcf);
        metrics.push(("Free Cash Flow".to_string(), fcf_str, assessment.to_string()));
        score += pts;
    }

    // P/E — Buffett will pay up for quality, but not insanely
    if let Some(pe) = f.and_then(|f| f.pe_ratio) {
        let (assessment, pts) = if pe < 0.0 {
            ("Negative earnings. Not investable by my standards.", -2)
        } else if pe < 15.0 {
            ("Bargain territory. If the business is good, this is attractive.", 2)
        } else if pe < 25.0 {
            ("Fair price for a quality business.", 1)
        } else if pe < 40.0 {
            ("Mr. Market is optimistic. Make sure the moat justifies this.", 0)
        } else {
            ("Expensive. Even great businesses can be bad investments at the wrong price.", -1)
        };
        metrics.push(("P/E Ratio".to_string(), format!("{pe:.1}"), assessment.to_string()));
        score += pts;
    }

    // Net Margin — Buffett wants pricing power
    if let Some(nm) = f.and_then(|f| f.net_margin) {
        let pct = nm * 100.0;
        let (assessment, pts) = if pct > 20.0 {
            ("Exceptional margin. Pricing power and moat evident.", 2)
        } else if pct > 10.0 {
            ("Healthy margin. The business has some competitive protection.", 1)
        } else if pct > 5.0 {
            ("Thin. Commodity-like business.", 0)
        } else {
            ("Razor-thin or negative. No pricing power.", -1)
        };
        metrics.push(("Net Margin".to_string(), format!("{pct:.1}%"), assessment.to_string()));
        score += pts;
    }

    // Astro take — Buffett acknowledges when it confirms
    let astro_take = match (ctx.astro_score, ctx.concordance.as_deref()) {
        (Some(s), Some("Strong Confirm")) if s > 60.0 => format!(
            "The stars suggest favorable conditions (score {s:.0}), and the fundamentals confirm it. \
             When the cosmos and the balance sheet agree, I pay attention. But I never buy a stock \
             because the stars say so. I buy because the business is wonderful at a fair price."
        ),
        (Some(s), Some("Divergence")) => format!(
            "I note the astrological divergence (score {s:.0}). The stars and the numbers disagree. \
             In my experience, the numbers win over the long term. I trust the balance sheet."
        ),
        (Some(s), _) if s > 70.0 => format!(
            "The astrological reading is favorable ({s:.0}). That's a nice tailwind, but it's not \
             the reason to buy. The reason is the business economics."
        ),
        (Some(s), _) if s < 30.0 => format!(
            "The stars are cautious ({s:.0}). I don't make investment decisions based on planetary \
             positions, but I do respect any signal that makes me think twice."
        ),
        _ => "I prefer to let the financial statements speak for themselves.".to_string(),
    };

    let verdict = match score {
        5.. => AgentVerdict::StrongBuy,
        3..=4 => AgentVerdict::Buy,
        0..=2 => AgentVerdict::Hold,
        -2..=-1 => AgentVerdict::Sell,
        _ => AgentVerdict::StrongSell,
    };

    let headline = match verdict {
        AgentVerdict::StrongBuy => format!("{} looks like a wonderful business at a fair price.", ctx.ticker),
        AgentVerdict::Buy => format!("{} has solid economics. I'd consider adding to my position.", ctx.ticker),
        AgentVerdict::Hold => format!("{} is decent but not compelling. I'd hold, not add.", ctx.ticker),
        AgentVerdict::Sell => format!("{} doesn't meet my quality threshold. Capital is better deployed elsewhere.", ctx.ticker),
        AgentVerdict::StrongSell => format!("{} has fundamental problems. This is outside my circle of competence.", ctx.ticker),
        AgentVerdict::InsufficientData => format!("I need more data on {} before forming an opinion.", ctx.ticker),
    };

    let analysis = if f.is_some() {
        build_buffett_narrative(ctx, score)
    } else {
        "I can't analyze what I can't see. Fetch the fundamentals first.".to_string()
    };

    AgentAnalysis {
        persona: AgentPersona::Buffett,
        headline,
        analysis,
        verdict: if f.is_some() { verdict } else { AgentVerdict::InsufficientData },
        key_metrics: metrics,
        astro_take,
    }
}

fn build_buffett_narrative(ctx: &AgentContext, score: i32) -> String {
    let f = ctx.fundamentals.as_ref().unwrap();
    let mut parts = Vec::new();

    if let Some(roe) = f.roe {
        let pct = roe * 100.0;
        if pct > 20.0 {
            parts.push(format!("ROE of {pct:.0}% suggests a durable competitive advantage."));
        } else if pct < 10.0 {
            parts.push(format!("ROE of {pct:.0}% is below my threshold. The business isn't earning its cost of capital."));
        }
    }

    if let Some(de) = f.debt_equity {
        if de > 2.0 {
            parts.push(format!("Debt-to-equity of {de:.1} concerns me. Leverage amplifies both gains and losses."));
        } else if de < 0.3 {
            parts.push("The balance sheet is a fortress. Very little debt.".to_string());
        }
    }

    if let Some(fcf) = f.fcf {
        if fcf > 10_000_000_000 {
            parts.push(format!("Free cash flow of {} means the owner is getting real money back.", format_large_number(fcf)));
        } else if fcf < 0 {
            parts.push("Negative free cash flow is a red flag. The business consumes more cash than it generates.".to_string());
        }
    }

    if score >= 5 {
        parts.push("Overall, this has the hallmarks of a wonderful business. I'd be comfortable owning it for decades.".to_string());
    } else if score <= -3 {
        parts.push("I'd pass on this one. There are better places to put capital.".to_string());
    }

    if parts.is_empty() {
        "The metrics paint a mixed picture. Not compelling enough to act.".to_string()
    } else {
        parts.join(" ")
    }
}

// ---------------------------------------------------------------------------
// Graham — "In the short run, the market is a voting machine..."
// ---------------------------------------------------------------------------

fn analyze_graham(ctx: &AgentContext) -> AgentAnalysis {
    let f = ctx.fundamentals.as_ref();
    let mut metrics = Vec::new();
    let mut score = 0i32;

    // P/E — Graham's ceiling is 15
    if let Some(pe) = f.and_then(|f| f.pe_ratio) {
        let (assessment, pts) = if pe < 0.0 {
            ("Negative earnings. Not suitable for value investing.", -3)
        } else if pe < 10.0 {
            ("Deep value. This is the kind of statistical cheapness I look for.", 3)
        } else if pe < 15.0 {
            ("Within my acceptable range. P/E under 15 is the minimum standard.", 2)
        } else if pe < 20.0 {
            ("Slightly above my threshold. Acceptable only with exceptional balance sheet.", 0)
        } else {
            ("Too expensive by my standards. Speculation, not investment.", -2)
        };
        metrics.push(("P/E Ratio".to_string(), format!("{pe:.1}"), assessment.to_string()));
        score += pts;
    }

    // P/B — Graham's ceiling is 1.5 (or P/E * P/B < 22.5)
    if let Some(pb) = f.and_then(|f| f.pb_ratio) {
        let (assessment, pts) = if pb < 1.0 {
            ("Trading below book value. This is where fortunes are made.", 3)
        } else if pb < 1.5 {
            ("Within my threshold of 1.5x book.", 2)
        } else if pb < 3.0 {
            ("Above my ideal range. The margin of safety is thinner.", 0)
        } else {
            ("Far above book value. The market is pricing in perfection.", -2)
        };
        metrics.push(("P/B Ratio".to_string(), format!("{pb:.2}"), assessment.to_string()));
        score += pts;
    }

    // Graham Number: sqrt(22.5 * EPS * Book/Share)
    if let (Some(pe), Some(pb)) = (f.and_then(|f| f.pe_ratio), f.and_then(|f| f.pb_ratio)) {
        if pe > 0.0 && pb > 0.0 {
            let graham_product = pe * pb;
            let (assessment, pts) = if graham_product < 22.5 {
                ("P/E x P/B under 22.5. Passes my combined test.", 2)
            } else {
                ("P/E x P/B exceeds 22.5. Fails my combined valuation test.", -1)
            };
            metrics.push(("P/E x P/B".to_string(), format!("{graham_product:.1}"), assessment.to_string()));
            score += pts;
        }
    }

    // Current Ratio — Graham wants > 2.0
    if let Some(cr) = f.and_then(|f| f.current_ratio) {
        let (assessment, pts) = if cr > 2.0 {
            ("Strong liquidity. Can cover short-term obligations twice over.", 2)
        } else if cr > 1.5 {
            ("Adequate liquidity.", 1)
        } else if cr > 1.0 {
            ("Tight. Barely covering current liabilities.", -1)
        } else {
            ("Below 1.0. This company may struggle to pay its bills.", -2)
        };
        metrics.push(("Current Ratio".to_string(), format!("{cr:.2}"), assessment.to_string()));
        score += pts;
    }

    // Dividend — Graham prefers dividend payers
    if let Some(dy) = f.and_then(|f| f.dividend_yield) {
        let pct = dy * 100.0;
        let (assessment, pts) = if pct > 3.0 {
            ("Strong dividend. Tangible return to shareholders.", 2)
        } else if pct > 1.0 {
            ("Modest dividend. Shows commitment to returning capital.", 1)
        } else if pct > 0.0 {
            ("Token dividend.", 0)
        } else {
            ("No dividend. I prefer companies that share profits.", -1)
        };
        metrics.push(("Dividend Yield".to_string(), format!("{pct:.2}%"), assessment.to_string()));
        score += pts;
    }

    // Astro take — Graham is politely skeptical
    let astro_take = match ctx.astro_score {
        Some(s) => format!(
            "I note the astrological score of {s:.0}, but my margin of safety analysis doesn't \
             depend on planetary positions. At the end of the day, a stock is worth the present \
             value of its future cash flows, regardless of what Jupiter is doing. I'll stick \
             to the numbers."
        ),
        None => "I have no opinion on astrology. The balance sheet is my horoscope.".to_string(),
    };

    let verdict = match score {
        7.. => AgentVerdict::StrongBuy,
        4..=6 => AgentVerdict::Buy,
        0..=3 => AgentVerdict::Hold,
        -3..=-1 => AgentVerdict::Sell,
        _ => AgentVerdict::StrongSell,
    };

    let headline = match verdict {
        AgentVerdict::StrongBuy => format!("{} is statistically cheap with a strong balance sheet. A textbook value investment.", ctx.ticker),
        AgentVerdict::Buy => format!("{} passes most of my quantitative screens. A reasonable investment.", ctx.ticker),
        AgentVerdict::Hold => format!("{} is neither cheap enough to buy nor expensive enough to sell.", ctx.ticker),
        AgentVerdict::Sell => format!("{} fails my valuation criteria. The margin of safety is inadequate.", ctx.ticker),
        AgentVerdict::StrongSell => format!("{} is speculative at this price. No margin of safety.", ctx.ticker),
        AgentVerdict::InsufficientData => format!("I need financial statements for {} before I can run my screens.", ctx.ticker),
    };

    let analysis = if f.is_some() {
        build_graham_narrative(ctx, score)
    } else {
        "Without financial data, I cannot perform my quantitative analysis.".to_string()
    };

    AgentAnalysis {
        persona: AgentPersona::Graham,
        headline,
        analysis,
        verdict: if f.is_some() { verdict } else { AgentVerdict::InsufficientData },
        key_metrics: metrics,
        astro_take,
    }
}

fn build_graham_narrative(ctx: &AgentContext, score: i32) -> String {
    let f = ctx.fundamentals.as_ref().unwrap();
    let mut parts = Vec::new();

    if let (Some(pe), Some(pb)) = (f.pe_ratio, f.pb_ratio) {
        if pe > 0.0 && pb > 0.0 && pe * pb < 22.5 {
            parts.push(format!(
                "The Graham Number test passes: P/E ({pe:.1}) x P/B ({pb:.1}) = {:.1}, under my 22.5 threshold.",
                pe * pb
            ));
        } else if pe > 0.0 && pb > 0.0 {
            parts.push(format!(
                "P/E ({pe:.1}) x P/B ({pb:.1}) = {:.1}, which exceeds my 22.5 ceiling. \
                 The stock is priced for growth, not value.",
                pe * pb
            ));
        }
    }

    if let Some(cr) = f.current_ratio {
        if cr < 1.5 {
            parts.push(format!("Current ratio of {cr:.2} is below my 2.0 minimum. Liquidity risk."));
        }
    }

    if score >= 7 {
        parts.push("This passes my full battery of quantitative tests. It has the statistical profile of a sound investment.".to_string());
    } else if score <= -3 {
        parts.push("The numbers don't lie. This fails on multiple quantitative criteria. Pass.".to_string());
    }

    if parts.is_empty() {
        "The quantitative picture is mixed. Not a clear buy or sell.".to_string()
    } else {
        parts.join(" ")
    }
}

// ---------------------------------------------------------------------------
// Lynch — "Know what you own, and know why you own it"
// ---------------------------------------------------------------------------

fn analyze_lynch(ctx: &AgentContext) -> AgentAnalysis {
    let f = ctx.fundamentals.as_ref();
    let mut metrics = Vec::new();
    let mut score = 0i32;

    // PEG Ratio — Lynch's signature metric
    if let Some(peg) = f.and_then(|f| f.peg_ratio) {
        let (assessment, pts) = if peg < 0.0 {
            ("Negative PEG means negative earnings growth. That's not growth investing.", -2)
        } else if peg < 1.0 {
            ("PEG under 1.0! The market hasn't fully priced in the growth. This is my sweet spot.", 3)
        } else if peg < 1.5 {
            ("PEG around 1.0-1.5. Fairly valued for its growth rate.", 1)
        } else if peg < 2.5 {
            ("Getting pricey relative to growth. The growth better be real.", 0)
        } else {
            ("Way too expensive for the growth you're getting.", -2)
        };
        metrics.push(("PEG Ratio".to_string(), format!("{peg:.2}"), assessment.to_string()));
        score += pts;
    }

    // P/E — Lynch will pay up for growth, but within reason
    if let Some(pe) = f.and_then(|f| f.pe_ratio) {
        let (assessment, pts) = if pe < 0.0 {
            ("No earnings. Can't be a GARP investment without the 'E'.", -2)
        } else if pe < 20.0 {
            ("Reasonable P/E. If this company is growing, it could be a bargain.", 2)
        } else if pe < 35.0 {
            ("Moderate P/E. Fine if the growth rate is comparable.", 1)
        } else if pe < 50.0 {
            ("High P/E. The growth story needs to be very compelling.", 0)
        } else {
            ("Nosebleed territory. A lot of future growth is already priced in.", -1)
        };
        metrics.push(("P/E Ratio".to_string(), format!("{pe:.1}"), assessment.to_string()));
        score += pts;
    }

    // Revenue growth proxy — revenue size as indicator
    if let Some(rev) = f.and_then(|f| f.revenue) {
        let rev_str = format_large_number(rev);
        let (assessment, pts) = if rev > 100_000_000_000 {
            ("Mega-cap revenue. Growth will be harder from here, but the moat is real.", 1)
        } else if rev > 10_000_000_000 {
            ("Large business. Still room to grow if the story is right.", 1)
        } else if rev > 1_000_000_000 {
            ("Mid-size. This is where the best growth stocks live.", 2)
        } else {
            ("Small revenue base. Could be a multi-bagger if the product is right.", 1)
        };
        metrics.push(("Revenue".to_string(), rev_str, assessment.to_string()));
        score += pts;
    }

    // Net Margin — Lynch wants profitable growth
    if let Some(nm) = f.and_then(|f| f.net_margin) {
        let pct = nm * 100.0;
        let (assessment, pts) = if pct > 15.0 {
            ("Strong margins. The business is printing money.", 2)
        } else if pct > 5.0 {
            ("Decent margins. Room to expand with scale.", 1)
        } else if pct > 0.0 {
            ("Thin margins. Watch for margin compression.", 0)
        } else {
            ("Losing money. This isn't GARP, it's hope.", -2)
        };
        metrics.push(("Net Margin".to_string(), format!("{pct:.1}%"), assessment.to_string()));
        score += pts;
    }

    // Debt/Equity — Lynch checks but doesn't obsess
    if let Some(de) = f.and_then(|f| f.debt_equity) {
        let (assessment, pts) = if de < 0.5 {
            ("Clean balance sheet. Growth funded organically.", 1)
        } else if de < 1.5 {
            ("Moderate leverage. Normal for most industries.", 0)
        } else {
            ("Heavy debt. If growth stalls, this leverage becomes a problem.", -1)
        };
        metrics.push(("Debt/Equity".to_string(), format!("{de:.2}"), assessment.to_string()));
        score += pts;
    }

    // Astro take — Lynch sees it as market psychology
    let astro_take = match (ctx.astro_score, ctx.dominant_theme.as_deref()) {
        (Some(s), Some(theme)) if s > 70.0 => format!(
            "The astro reading says '{theme}' with a score of {s:.0}. I see astrology as a proxy \
             for market psychology. When the stars align favorably, sentiment tends to follow. \
             But I'd still want to walk into the store, try the product, and talk to the employees \
             before buying the stock."
        ),
        (Some(s), _) if s < 30.0 => format!(
            "Astro score of {s:.0} suggests headwinds. Markets are driven by psychology as much \
             as fundamentals. If sentiment is turning against this company, I want to understand \
             why before going against the crowd."
        ),
        _ => "I don't follow astrology closely, but I respect any signal that captures market mood. \
              What matters most is: do I understand this business?".to_string(),
    };

    let verdict = match score {
        7.. => AgentVerdict::StrongBuy,
        4..=6 => AgentVerdict::Buy,
        1..=3 => AgentVerdict::Hold,
        -2..=0 => AgentVerdict::Sell,
        _ => AgentVerdict::StrongSell,
    };

    let headline = match verdict {
        AgentVerdict::StrongBuy => format!("{} is growing fast and the market hasn't fully priced it in. Classic GARP.", ctx.ticker),
        AgentVerdict::Buy => format!("{} has a good growth story at a reasonable price.", ctx.ticker),
        AgentVerdict::Hold => format!("{} is interesting but the price-to-growth ratio isn't compelling yet.", ctx.ticker),
        AgentVerdict::Sell => format!("{} is either too expensive for its growth or the growth isn't there.", ctx.ticker),
        AgentVerdict::StrongSell => format!("{} fails my growth-at-reasonable-price test on multiple fronts.", ctx.ticker),
        AgentVerdict::InsufficientData => format!("I need to see the numbers for {} before I can form an opinion.", ctx.ticker),
    };

    let analysis = if f.is_some() {
        build_lynch_narrative(ctx, score)
    } else {
        "I can't tell you if it's a good stock without seeing the business.".to_string()
    };

    AgentAnalysis {
        persona: AgentPersona::Lynch,
        headline,
        analysis,
        verdict: if f.is_some() { verdict } else { AgentVerdict::InsufficientData },
        key_metrics: metrics,
        astro_take,
    }
}

fn build_lynch_narrative(ctx: &AgentContext, score: i32) -> String {
    let f = ctx.fundamentals.as_ref().unwrap();
    let mut parts = Vec::new();

    if let Some(peg) = f.peg_ratio {
        if peg > 0.0 && peg < 1.0 {
            parts.push(format!(
                "PEG of {peg:.2} is the standout here. The market is pricing in less growth than the company is delivering."
            ));
        } else if peg > 2.0 {
            parts.push(format!(
                "PEG of {peg:.2} means you're paying a premium for growth. Make sure you understand what's driving it."
            ));
        }
    }

    if score >= 5 {
        parts.push("This has the profile of a growth stock at a reasonable price. The kind I'd want to own.".to_string());
    } else if score <= -2 {
        parts.push("I'd pass here. There are better growth stories at better prices.".to_string());
    }

    if parts.is_empty() {
        "The growth-value picture is mixed. I'd want to dig deeper into the actual business.".to_string()
    } else {
        parts.join(" ")
    }
}

// ---------------------------------------------------------------------------
// Munger — "All I want to know is where I'm going to die, so I'll never go there"
// ---------------------------------------------------------------------------

fn analyze_munger(ctx: &AgentContext) -> AgentAnalysis {
    let f = ctx.fundamentals.as_ref();
    let mut metrics = Vec::new();
    let mut score = 0i32;

    // ROE — Munger cares deeply about quality
    if let Some(roe) = f.and_then(|f| f.roe) {
        let pct = roe * 100.0;
        let (assessment, pts) = if pct > 25.0 {
            ("Outstanding. This business has a wide moat.", 3)
        } else if pct > 15.0 {
            ("Good returns on equity. Competitive position is solid.", 2)
        } else if pct > 8.0 {
            ("Average. Not the kind of quality I gravitate toward.", 0)
        } else {
            ("Poor capital allocation. Management should return the capital to shareholders.", -2)
        };
        metrics.push(("ROE".to_string(), format!("{pct:.1}%"), assessment.to_string()));
        score += pts;
    }

    // Operating Margin — Munger wants pricing power
    if let Some(om) = f.and_then(|f| f.operating_margin) {
        let pct = om * 100.0;
        let (assessment, pts) = if pct > 30.0 {
            ("Exceptional pricing power. The moat is deep.", 3)
        } else if pct > 15.0 {
            ("Solid operations. Some competitive protection.", 1)
        } else if pct > 5.0 {
            ("Thin margins. Commoditized business.", 0)
        } else {
            ("No operating leverage. A dollar in, barely a dime out.", -2)
        };
        metrics.push(("Operating Margin".to_string(), format!("{pct:.1}%"), assessment.to_string()));
        score += pts;
    }

    // EV/EBITDA — Munger checks but prioritizes quality
    if let Some(ev) = f.and_then(|f| f.ev_ebitda) {
        let (assessment, pts) = if ev < 0.0 {
            ("Negative EBITDA. Not investable.", -2)
        } else if ev < 10.0 {
            ("Cheap by enterprise value. Is the quality here too?", 1)
        } else if ev < 20.0 {
            ("Fair enterprise value.", 0)
        } else {
            ("Premium valuation. The quality needs to be exceptional to justify this.", -1)
        };
        metrics.push(("EV/EBITDA".to_string(), format!("{ev:.1}"), assessment.to_string()));
        score += pts;
    }

    // FCF — Munger loves cash generation
    if let Some(fcf) = f.and_then(|f| f.fcf) {
        let (assessment, pts) = if fcf > 5_000_000_000 {
            ("Excellent cash generation. This business funds its own growth.", 2)
        } else if fcf > 0 {
            ("Positive cash flow. The business sustains itself.", 1)
        } else {
            ("Cash burn. Avoid.", -2)
        };
        metrics.push(("FCF".to_string(), format_large_number(fcf), assessment.to_string()));
        score += pts;
    }

    // Debt/Equity — Munger hates leverage
    if let Some(de) = f.and_then(|f| f.debt_equity) {
        let (assessment, pts) = if de < 0.3 {
            ("Almost no debt. The best kind of balance sheet.", 2)
        } else if de < 1.0 {
            ("Manageable leverage.", 1)
        } else if de < 2.0 {
            ("I'd want to understand why they need this much debt.", -1)
        } else {
            ("Excessive leverage. Three things ruin smart people: liquor, ladies, and leverage.", -2)
        };
        metrics.push(("Debt/Equity".to_string(), format!("{de:.2}"), assessment.to_string()));
        score += pts;
    }

    // Astro take — Munger views it as one data point among many
    let astro_take = match (ctx.astro_score, ctx.concordance.as_deref()) {
        (Some(s), Some(conc)) => format!(
            "The astrological alignment shows a score of {s:.0} with {conc} concordance. \
             Interesting. I'm more interested in whether the business has a durable moat. \
             The return on equity tells me more than the stars. But I never dismiss a data \
             point entirely. The world is full of things we don't understand."
        ),
        (Some(s), _) => format!(
            "Astro score of {s:.0}. I view this the way I view any unfamiliar mental model: \
             with curiosity, not conviction. Show me the business quality and I'll show you \
             my opinion."
        ),
        _ => "I don't need the stars to tell me if a business is good. I need the financial statements.".to_string(),
    };

    let verdict = match score {
        8.. => AgentVerdict::StrongBuy,
        5..=7 => AgentVerdict::Buy,
        1..=4 => AgentVerdict::Hold,
        -2..=0 => AgentVerdict::Sell,
        _ => AgentVerdict::StrongSell,
    };

    let headline = match verdict {
        AgentVerdict::StrongBuy => format!("{} is a high-quality business with a durable moat. I'd hold forever.", ctx.ticker),
        AgentVerdict::Buy => format!("{} shows good quality indicators. Worth owning.", ctx.ticker),
        AgentVerdict::Hold => format!("{} is acceptable but not the kind of quality that makes me excited.", ctx.ticker),
        AgentVerdict::Sell => format!("{} doesn't meet my quality bar. I'd rather own a wonderful business at a fair price.", ctx.ticker),
        AgentVerdict::StrongSell => format!("{} shows multiple quality red flags. Invert: what would make this fail? Too much.", ctx.ticker),
        AgentVerdict::InsufficientData => format!("I need to see the economics of {} before I can form a judgment.", ctx.ticker),
    };

    let analysis = if f.is_some() {
        build_munger_narrative(ctx, score)
    } else {
        "Show me the business economics. Then we can talk.".to_string()
    };

    AgentAnalysis {
        persona: AgentPersona::Munger,
        headline,
        analysis,
        verdict: if f.is_some() { verdict } else { AgentVerdict::InsufficientData },
        key_metrics: metrics,
        astro_take,
    }
}

fn build_munger_narrative(ctx: &AgentContext, score: i32) -> String {
    let f = ctx.fundamentals.as_ref().unwrap();
    let mut parts = Vec::new();

    if let Some(roe) = f.roe {
        let pct = roe * 100.0;
        if pct > 25.0 {
            parts.push(format!("ROE of {pct:.0}% is exceptional. When you see this consistently, there's usually a moat."));
        }
    }

    if let Some(om) = f.operating_margin {
        let pct = om * 100.0;
        if pct > 30.0 {
            parts.push(format!("Operating margin of {pct:.0}% shows real pricing power. Competitors can't touch this."));
        } else if pct < 5.0 {
            parts.push("Thin operating margin suggests a commoditized business. No pricing power.".to_string());
        }
    }

    if score >= 8 {
        parts.push("This is the kind of quality business I want to own forever. The key question is price.".to_string());
    } else if score <= -2 {
        parts.push("Invert, always invert. What would destroy this investment? Too many things. Pass.".to_string());
    }

    if parts.is_empty() {
        "The quality picture is mixed. I'd keep watching but not act.".to_string()
    } else {
        parts.join(" ")
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_large_number(n: i64) -> String {
    let f = n as f64;
    if f.abs() >= 1_000_000_000.0 { format!("${:.1}B", f / 1_000_000_000.0) }
    else if f.abs() >= 1_000_000.0 { format!("${:.1}M", f / 1_000_000.0) }
    else { format!("${f:.0}") }
}
