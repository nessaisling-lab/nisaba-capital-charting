//! Horoscope reading engine — narrative financial interpretations.
//!
//! This is the killer feature: a written "horoscope reading" for each ticker.
//! Not just a number — a narrative interpretation that explains what the stars
//! say and what it means for the company.
//!
//! The engine is **template-based** (deterministic, free, instant, no API calls).
//! Each planet-aspect-planet combination maps to a pre-written financial
//! interpretation. The system reads active aspects, picks the top 3–5 by
//! magnitude, maps each to a financial meaning, then synthesizes an overall
//! narrative from the dominant theme.

use chrono::NaiveDate;

use super::aspects::{ActiveAspect, AspectType, DignityState};
use super::ephemeris::Planet;
use super::natal::TransitScore;

// ---------------------------------------------------------------------------
// Horoscope reading structs
// ---------------------------------------------------------------------------

/// A complete horoscope reading for a single ticker on a single date.
#[derive(Debug, Clone)]
pub struct HoroscopeReading {
    pub ticker:           String,
    pub date:             NaiveDate,
    /// 2-3 sentence narrative summary of the overall outlook.
    pub overall_outlook:  String,
    /// Dominant thematic energy: "Growth & Expansion", "Caution & Restructuring", etc.
    pub dominant_theme:   String,
    /// Top 3-5 most significant transit interpretations, ranked by magnitude.
    pub key_transits:     Vec<TransitInterpretation>,
    /// Moon phase interpretation in financial context.
    pub moon_guidance:    String,
    /// Mercury retrograde caution (None if Mercury is direct).
    pub mercury_warning:  Option<String>,
    /// Timing window: "Favorable window: next 2 weeks" or "Wait for clarity".
    pub timing_window:    String,
    /// Confidence: how many strong aspects support this reading (0-100).
    pub confidence:       f32,
}

/// A single transit aspect interpreted for financial context.
#[derive(Debug, Clone)]
pub struct TransitInterpretation {
    /// Human-readable description: "Jupiter trine natal Venus"
    pub transit_desc:          String,
    /// What it means in financial terms.
    pub meaning:               String,
    /// Specific financial implication: "Favorable for M&A, new product launches"
    pub financial_implication:  String,
    /// Strength indicator: "Strong (applying, exact in 3 days)"
    pub strength:              String,
}

// ---------------------------------------------------------------------------
// Main generation entry point
// ---------------------------------------------------------------------------

/// Generate a complete horoscope reading from a natal chart and transit score.
pub fn generate_horoscope(score: &TransitScore) -> HoroscopeReading {
    // 1. Interpret the top 3-5 aspects by magnitude
    let key_transits: Vec<TransitInterpretation> = score.active_aspects
        .iter()
        .take(5)
        .map(interpret_aspect)
        .collect();

    // 2. Classify dominant theme from all aspects
    let dominant_theme = classify_dominant_theme(&score.active_aspects, score.astro_score);

    // 3. Moon phase interpretation
    let moon_guidance = interpret_moon_phase(&score.moon_phase, score.moon_phase_deg);

    // 4. Mercury retrograde warning
    let mercury_warning = if score.mercury_rx {
        Some(interpret_mercury_rx(score.moon_phase_deg))
    } else {
        None
    };

    // 5. Timing window based on applying aspects
    let timing_window = compute_timing_window(&score.active_aspects, score.astro_score);

    // 6. Confidence: based on number of strong aspects and their agreement
    let confidence = compute_confidence(&score.active_aspects, score.astro_score);

    // 7. Synthesize overall outlook
    let overall_outlook = synthesize_outlook(
        &dominant_theme,
        &key_transits,
        &moon_guidance,
        &mercury_warning,
        score.astro_score,
    );

    HoroscopeReading {
        ticker: score.ticker.clone(),
        date: score.score_date,
        overall_outlook,
        dominant_theme,
        key_transits,
        moon_guidance,
        mercury_warning,
        timing_window,
        confidence,
    }
}

// ---------------------------------------------------------------------------
// Aspect interpretation — planet-aspect-planet templates
// ---------------------------------------------------------------------------

fn interpret_aspect(aspect: &ActiveAspect) -> TransitInterpretation {
    let transit_desc = format!(
        "{} {} natal {}",
        aspect.transit_planet.name(),
        aspect.aspect.symbol(),
        aspect.natal_planet.name(),
    );

    let (meaning, financial) = planet_aspect_meaning(
        aspect.transit_planet,
        aspect.natal_planet,
        aspect.aspect,
    );

    // Dignity enrichment
    let dignity_note = match aspect.dignity {
        DignityState::Domicile => format!(
            " {} is in domicile (strongest expression), amplifying this transit.",
            aspect.transit_planet.name(),
        ),
        DignityState::Exaltation => format!(
            " {} is exalted (elevated power), strengthening this influence.",
            aspect.transit_planet.name(),
        ),
        DignityState::Detriment => format!(
            " {} is in detriment (weakened), moderating this transit's impact.",
            aspect.transit_planet.name(),
        ),
        DignityState::Fall => format!(
            " {} is in fall (most challenged), reducing the effectiveness of this transit.",
            aspect.transit_planet.name(),
        ),
        DignityState::Peregrine => String::new(),
    };

    let full_meaning = format!("{}{}", meaning, dignity_note);

    // Strength description
    let apply_str = if aspect.applying { "applying" } else { "separating" };
    let orb_str = format!("{:.1}°", aspect.orb);
    let strength_word = if aspect.orb < 1.0 {
        "Very strong"
    } else if aspect.orb < 3.0 {
        "Strong"
    } else if aspect.orb < 5.0 {
        "Moderate"
    } else {
        "Mild"
    };
    let strength = format!("{} ({}, orb {})", strength_word, apply_str, orb_str);

    TransitInterpretation {
        transit_desc,
        meaning: full_meaning,
        financial_implication: financial,
        strength,
    }
}

/// Core interpretation table: maps (transit_planet, natal_planet, aspect_type) to
/// (meaning, financial_implication). This is the heart of the horoscope engine.
///
/// The interpretations follow traditional financial astrology principles:
/// - Jupiter = expansion, opportunity, optimism
/// - Saturn = restriction, discipline, contraction
/// - Mars = energy, competition, volatility
/// - Venus = value, harmony, partnerships
/// - Mercury = communication, commerce, information flow
/// - Sun = identity, leadership, core business
/// - Moon = sentiment, public perception, fluctuation
/// - Uranus = disruption, innovation, sudden change
/// - Neptune = illusion, confusion, visionary potential
/// - Pluto = transformation, power shifts, deep restructuring
/// - NorthNode = destiny, growth direction, karmic alignment
/// - SouthNode = past patterns, release, diminishing returns
/// - Chiron = vulnerability, healing through innovation
fn planet_aspect_meaning(transit: Planet, natal: Planet, aspect: AspectType) -> (String, String) {
    // Determine aspect quality: harmonious (+1), challenging (-1), neutral (0)
    let harmonious = matches!(
        aspect,
        AspectType::Trine | AspectType::Sextile | AspectType::SemiSextile
    );
    let challenging = matches!(
        aspect,
        AspectType::Square | AspectType::Opposition | AspectType::SemiSquare | AspectType::Sesquiquadrate | AspectType::Quincunx
    );

    // Build interpretation from transit planet + aspect quality + natal planet
    let (meaning, financial) = match (transit, harmonious, challenging) {
        // ---- JUPITER transits ----
        (Planet::Jupiter, true, _) => match natal {
            Planet::Sun => (
                "Jupiter's expansive energy flows harmoniously with the company's core identity. This is a period of natural growth, increased visibility, and leadership confidence.",
                "Revenue expansion likely. Favorable for strategic initiatives, IPOs, and market cap growth. Executive decisions trend positive.",
            ),
            Planet::Moon => (
                "Jupiter harmonizes with public sentiment. The company enjoys favorable perception, positive media coverage, and growing customer loyalty.",
                "Consumer spending increases. Retail-facing companies benefit. Favorable for product launches and marketing campaigns.",
            ),
            Planet::Venus => (
                "Jupiter amplifies Venus's themes of value and partnership. Alliances flourish, deal-making is favored, and the company attracts valuable relationships.",
                "Favorable for M&A activity, partnership announcements, and licensing deals. Revenue from partnerships likely to exceed expectations.",
            ),
            Planet::Mercury => (
                "Jupiter expands Mercury's domain of communication and commerce. Information flows freely, contracts are favorable, and intellectual property gains value.",
                "Positive for earnings calls, patent filings, product announcements, and any information-driven revenue. Analyst coverage likely favorable.",
            ),
            Planet::Mars => (
                "Jupiter empowers Mars's drive and competitive energy. The company's aggressive strategies yield outsized returns. Bold moves are rewarded.",
                "Market share gains likely. Competitive positioning strengthens. Favorable for entering new markets or launching aggressive pricing strategies.",
            ),
            Planet::Saturn => (
                "Jupiter eases Saturn's restrictions. Regulatory headwinds diminish, structural reforms bear fruit, and long-term investments start paying off.",
                "Debt restructuring succeeds. Regulatory approval likely. Long-term infrastructure investments begin generating returns.",
            ),
            Planet::Pluto => (
                "Jupiter magnifies Pluto's transformative power in a constructive direction. Deep structural changes accelerate the company's evolution.",
                "Transformative deals close favorably. Power consolidation succeeds. Major pivots gain market validation.",
            ),
            Planet::NorthNode => (
                "Jupiter aligns with the company's karmic growth direction. This is a destiny-level expansion window where the company's purpose crystallizes.",
                "Strategic direction is validated by market response. Growth initiatives align with long-term destiny. Investors recognize the vision.",
            ),
            _ => (
                "Jupiter brings expansion and optimism to this area of the company's chart. Growth energy is present and constructive.",
                "General expansion outlook is favorable. Opportunities present themselves naturally.",
            ),
        },
        (Planet::Jupiter, _, true) => match natal {
            Planet::Sun => (
                "Jupiter's expansion collides with the company's core identity, creating overextension risk. Ambition may outpace execution capability.",
                "Watch for overvaluation. Aggressive expansion may strain resources. Leadership could overcommit to growth targets.",
            ),
            Planet::Saturn => (
                "Jupiter pushes against Saturn's structural limits, creating tension between growth ambitions and fiscal discipline. Something has to give.",
                "Debt-fueled growth risks. Regulatory friction with expansion plans. Capital allocation decisions are contentious.",
            ),
            Planet::Mars => (
                "Jupiter inflates Mars's combative energy, risking overaggressive competitive moves or executive overconfidence.",
                "Antitrust scrutiny risk. Price wars may erode margins. Leadership could pick fights it can't win.",
            ),
            _ => (
                "Jupiter's expansion creates friction in this area. Growth is possible but comes with growing pains and overextension risk.",
                "Expansion plans face headwinds. Scale carefully and validate demand before committing resources.",
            ),
        },

        // ---- SATURN transits ----
        (Planet::Saturn, true, _) => match natal {
            Planet::Sun => (
                "Saturn's discipline harmonizes with core business identity. This is a period of earned credibility, mature growth, and institutional recognition.",
                "Credit ratings may improve. Institutional investors increase positions. Long-term contracts are secured.",
            ),
            Planet::Venus => (
                "Saturn brings stability to partnerships and valuations. Relationships that survive this transit are built to last.",
                "Partnership agreements solidify. Valuation metrics normalize. Dividend policies strengthen.",
            ),
            Planet::Jupiter => (
                "Saturn structures Jupiter's growth energy productively. Expansion happens in a disciplined, sustainable way.",
                "Profitable growth over rapid scaling. Margin improvement alongside revenue growth. Sustainable competitive advantages form.",
            ),
            Planet::Mercury => (
                "Saturn brings rigor to communications and commerce. Contracts are tighter, processes more efficient, and information more reliable.",
                "Favorable for compliance milestones, audit results, and process automation. Operational efficiency gains.",
            ),
            _ => (
                "Saturn provides structure and discipline to this area. Progress is slow but solid.",
                "Long-term foundations strengthen. Patience rewarded. Institutional confidence builds.",
            ),
        },
        (Planet::Saturn, _, true) => match natal {
            Planet::Sun => (
                "Saturn restricts the company's core vitality. Leadership faces tests, growth slows, and the market demands proof of value.",
                "Revenue growth stalls or declines. CEO/leadership transitions possible. Market re-rates the stock downward.",
            ),
            Planet::Moon => (
                "Saturn dampens public sentiment. The company faces reputation challenges, customer dissatisfaction, or negative media cycles.",
                "Consumer confidence in the brand weakens. Customer churn increases. PR crises more likely.",
            ),
            Planet::Mars => (
                "Saturn blocks Mars's energy, creating frustration, delayed initiatives, and competitive stagnation.",
                "Product launches delayed. Market share erosion. Supply chain bottlenecks. Execution struggles.",
            ),
            Planet::Venus => (
                "Saturn tests partnerships and value propositions. Relationships face stress, and valuations come under scrutiny.",
                "Partnership dissolution risk. Valuation compression. Dividend cuts possible if cash flow tightens.",
            ),
            Planet::Pluto => (
                "Saturn constrains Pluto's transformative drive, creating a period of painful but necessary restructuring.",
                "Forced restructuring. Layoffs possible. Legacy systems must be replaced under budget pressure.",
            ),
            _ => (
                "Saturn brings restriction and testing to this area. Challenges require patience and discipline to overcome.",
                "Headwinds in this area. Progress requires discipline. Cut what isn't working.",
            ),
        },

        // ---- MARS transits ----
        (Planet::Mars, true, _) => match natal {
            Planet::Sun => (
                "Mars energizes the company's core identity. Competitive drive intensifies productively, and bold initiatives gain momentum.",
                "Product launches succeed. Market share gains. Executive energy and decisiveness at peak. Time to act.",
            ),
            Planet::Jupiter => (
                "Mars fuels Jupiter's expansion with competitive fire. The company's growth ambitions are backed by genuine execution capability.",
                "Strong earnings momentum. Market entry succeeds. Sales teams outperform targets.",
            ),
            Planet::Venus => (
                "Mars brings passionate energy to partnerships and value creation. New alliances form quickly and with conviction.",
                "Deal-making accelerated. Acquisition targets identified and pursued. Revenue partnerships close faster.",
            ),
            _ => (
                "Mars brings energy and drive to this area. Action is favored over deliberation.",
                "Execute on existing plans. Competitive momentum is strong.",
            ),
        },
        (Planet::Mars, _, true) => match natal {
            Planet::Sun => (
                "Mars creates friction with core identity. Internal conflicts, competitive threats, or leadership clashes disrupt operations.",
                "Volatility increases. Activist investor risk. Executive departures. Product recalls or quality issues.",
            ),
            Planet::Saturn => (
                "Mars's aggressive energy smashes against Saturn's limits. Frustration peaks, safety incidents or regulatory violations possible.",
                "Workplace safety concerns. Regulatory fines. Labor disputes. Equipment failures or production accidents.",
            ),
            Planet::Pluto => (
                "Mars triggers Pluto's destructive potential. Power struggles intensify, and the company faces existential competitive threats.",
                "Hostile takeover risk. Major competitive disruption. Cybersecurity threats. Crisis-level events possible.",
            ),
            _ => (
                "Mars creates tension and conflict in this area. Volatility and impulsive decisions are risks.",
                "Manage volatility risk. Avoid impulsive strategic shifts. Competition heats up.",
            ),
        },

        // ---- VENUS transits ----
        (Planet::Venus, true, _) => match natal {
            Planet::Sun => (
                "Venus harmonizes with core identity, bringing attractiveness, favorable public perception, and smooth operations.",
                "Brand value increases. Customer acquisition costs decrease. Favorable media coverage.",
            ),
            Planet::Jupiter => (
                "Venus and Jupiter create the most auspicious combination in financial astrology. Abundance, generosity, and profitable growth.",
                "Peak revenue periods. Successful fundraising. Luxury/premium segments thrive. Stock buyback programs well-received.",
            ),
            Planet::Moon => (
                "Venus soothes public sentiment. The company enjoys a wave of goodwill, positive reviews, and customer satisfaction.",
                "NPS scores improve. Customer retention peaks. Social media sentiment strongly positive.",
            ),
            _ => (
                "Venus brings harmony, beauty, and value to this area. Relationships and aesthetics are favored.",
                "Partnership value increases. Product design wins recognition. Cost optimization succeeds.",
            ),
        },
        (Planet::Venus, _, true) => match natal {
            Planet::Saturn => (
                "Venus struggles against Saturn's austerity. Partnerships feel restrictive, and the company must choose between comfort and discipline.",
                "Valuation pressure. Cost-cutting tensions. Partnership renegotiations trend unfavorable.",
            ),
            Planet::Mars => (
                "Venus and Mars clash, creating tension between cooperation and competition, diplomacy and aggression.",
                "Internal team conflicts. Partnership disputes. Brand messaging inconsistency.",
            ),
            _ => (
                "Venus encounters friction in this area. Harmony is disrupted but adjustments restore balance.",
                "Short-term valuation headwinds. Relationship friction resolves within weeks.",
            ),
        },

        // ---- MERCURY transits ----
        (Planet::Mercury, true, _) => match natal {
            Planet::Sun => (
                "Mercury facilitates clear communication about core identity. The company's message resonates, and information flows efficiently.",
                "Earnings calls well-received. Product messaging lands. Analyst communication effective.",
            ),
            Planet::Jupiter => (
                "Mercury connects with Jupiter's expansive vision. Big ideas are communicated effectively, and intellectual property gains value.",
                "Patent approvals. Successful product announcements. Favorable analyst initiations.",
            ),
            _ => (
                "Mercury brings clarity, communication, and commercial acumen to this area.",
                "Information-driven advantages. Contract negotiations succeed. Data insights emerge.",
            ),
        },
        (Planet::Mercury, _, true) => match natal {
            Planet::Saturn => (
                "Mercury's quick thinking clashes with Saturn's slow deliberation. Communication breakdowns, contractual disputes, or regulatory paperwork delays.",
                "Filing delays. Contractual disputes. IT system issues. Miscommunication in earnings guidance.",
            ),
            _ => (
                "Mercury creates communication friction in this area. Misunderstandings and information gaps are likely.",
                "Watch for mispriced expectations. Clarify messaging. Delay major announcements if possible.",
            ),
        },

        // ---- URANUS transits ----
        (Planet::Uranus, true, _) => match natal {
            Planet::Sun => (
                "Uranus brings innovative breakthroughs to core identity. The company reinvents itself in ways the market celebrates.",
                "Breakthrough product launches. Technology pivots succeed. Market disruption in the company's favor.",
            ),
            Planet::Mercury => (
                "Uranus electrifies Mercury's domain. Breakthrough communications technology, viral moments, or paradigm-shifting information.",
                "Tech innovation announcements. Viral marketing success. AI/ML breakthroughs. Platform evolution.",
            ),
            _ => (
                "Uranus brings sudden positive innovation and unexpected opportunities to this area.",
                "Disruptive innovation opportunities. First-mover advantages. Technology leapfrogging.",
            ),
        },
        (Planet::Uranus, _, true) => match natal {
            Planet::Sun => (
                "Uranus disrupts core identity unpredictably. The company faces sudden market shifts, leadership upheaval, or forced reinvention.",
                "Unexpected earnings misses. Flash crashes. Leadership shake-ups. Industry disruption from competitors.",
            ),
            Planet::Saturn => (
                "Uranus shatters Saturn's structures. Established processes fail, regulations change abruptly, or organizational structures collapse.",
                "Regulatory surprises. Supply chain disruption. Legacy system failures. Sudden organizational restructuring.",
            ),
            _ => (
                "Uranus brings volatility and unpredictable disruption to this area. Expect the unexpected.",
                "Black swan risk elevated. Position hedging recommended. Prepare contingency plans.",
            ),
        },

        // ---- NEPTUNE transits ----
        (Planet::Neptune, true, _) => match natal {
            Planet::Sun => (
                "Neptune inspires the company's core vision. Creative breakthroughs, brand mythology, and aspirational positioning flourish.",
                "Brand equity increases. Creative campaigns resonate deeply. Vision-driven investors attracted.",
            ),
            Planet::Venus => (
                "Neptune and Venus create artistic and aesthetic brilliance. The company's aesthetic appeal and brand beauty peak.",
                "Luxury brand positioning strengthens. Design-driven products succeed. Emotional customer connections deepen.",
            ),
            _ => (
                "Neptune brings visionary inspiration and creative potential to this area.",
                "Vision-driven strategy favored. Creative problem-solving yields results. Intangible assets appreciate.",
            ),
        },
        (Planet::Neptune, _, true) => match natal {
            Planet::Sun => (
                "Neptune obscures core identity. The company's direction becomes unclear, and the market struggles to value it accurately.",
                "Valuation confusion. Accounting irregularities surface. Strategic direction unclear. Avoid entry until clarity returns.",
            ),
            Planet::Mercury => (
                "Neptune fogs Mercury's clarity. Information becomes unreliable, communication misleading, and contracts ambiguous.",
                "Earnings restatement risk. Misleading guidance. IP theft or information leaks. Due diligence failures.",
            ),
            _ => (
                "Neptune creates confusion, illusion, and unclear signals in this area. Discernment is critical.",
                "Verify all information independently. Avoid speculative positions based on rumor. Wait for clarity.",
            ),
        },

        // ---- PLUTO transits ----
        (Planet::Pluto, true, _) => match natal {
            Planet::Sun => (
                "Pluto empowers the company's core identity with transformative force. Deep reinvention succeeds and creates lasting competitive advantages.",
                "Transformative M&A succeeds. Market dominance consolidates. Power positioning pays off long-term.",
            ),
            Planet::Mercury => (
                "Pluto transforms the company's intellectual framework. Paradigm-shifting insights drive strategic evolution.",
                "Pivotal IP developments. Deep tech breakthroughs. Transformative data-driven decisions.",
            ),
            _ => (
                "Pluto brings constructive transformation and empowerment to this area. Deep change creates lasting value.",
                "Strategic transformation opportunities. Deep restructuring creates long-term value. Power consolidation.",
            ),
        },
        (Planet::Pluto, _, true) => match natal {
            Planet::Sun => (
                "Pluto's destructive aspect targets core identity. The company faces existential transformation: evolve or perish.",
                "Existential competitive threat. Hostile takeover risk. Fundamental business model under attack. Deep restructuring unavoidable.",
            ),
            Planet::Venus => (
                "Pluto destroys and rebuilds value structures. Partnerships are tested to destruction, and valuation undergoes radical revision.",
                "Major write-downs possible. Partnership dissolution. Valuation collapse and rebuilding. Phoenix-from-ashes scenario.",
            ),
            _ => (
                "Pluto brings intense pressure for transformation in this area. Resistance to change is futile.",
                "Forced transformation. Eliminate what no longer serves growth. Rebuild from foundations.",
            ),
        },

        // ---- NODE transits ----
        (Planet::NorthNode, true, _) => {
            let meaning = format!(
                "The North Node aligns harmoniously with natal {}, indicating karmic alignment with the company's growth destiny. The universe supports this direction.",
                natal.name(),
            );
            return (meaning, "Strategic direction validated. Growth initiatives align with long-term purpose. Investors see the vision.".to_string());
        },
        (Planet::NorthNode, _, true) => {
            let meaning = format!(
                "The North Node creates tension with natal {}, suggesting the company's growth direction conflicts with established patterns. Course correction needed.",
                natal.name(),
            );
            return (meaning, "Strategic misalignment risk. Growth initiatives may not serve long-term destiny. Re-evaluate direction.".to_string());
        },
        (Planet::SouthNode, true, _) => {
            let meaning = format!(
                "The South Node harmonizes with natal {}, allowing the company to leverage past strengths and established patterns constructively.",
                natal.name(),
            );
            return (meaning, "Legacy assets produce value. Established market position holds. Historical strengths remain relevant.".to_string());
        },
        (Planet::SouthNode, _, true) => {
            let meaning = format!(
                "The South Node creates friction with natal {}, indicating diminishing returns from old approaches. The past is no longer a reliable guide.",
                natal.name(),
            );
            return (meaning, "Legacy approaches losing effectiveness. Market position eroding. Innovation required to maintain relevance.".to_string());
        },

        // ---- CHIRON transits ----
        (Planet::Chiron, true, _) => {
            let meaning = format!(
                "Chiron, the wounded healer, harmonizes with natal {}. Past vulnerabilities become sources of innovation and unique competitive advantage.",
                natal.name(),
            );
            return (meaning, "Turning weaknesses into strengths. Unique market positioning from past challenges. Authenticity resonates with customers.".to_string());
        },
        (Planet::Chiron, _, true) => {
            let meaning = format!(
                "Chiron exposes natal {} vulnerabilities. Old wounds resurface, but the path to healing creates unexpected strength.",
                natal.name(),
            );
            return (meaning, "Vulnerable areas exposed. Short-term pain from addressing long-ignored issues. Healing leads to stronger foundation.".to_string());
        },

        // ---- SUN transits ----
        (Planet::Sun, true, _) => match natal {
            Planet::Jupiter => (
                "The Sun illuminates Jupiter's expansive potential. Leadership clarity amplifies growth opportunities.",
                "CEO visibility drives stock appreciation. Strategic vision becomes actionable. Market confidence peaks.",
            ),
            Planet::Saturn => (
                "The Sun brings clarity to Saturn's lessons. Leadership earns respect through disciplined execution.",
                "Institutional recognition. Credit upgrades possible. Leadership credibility strengthened.",
            ),
            _ => (
                "The Sun illuminates this area with clarity and leadership energy. Core identity is strengthened.",
                "Leadership visibility improves. Corporate identity sharpens. Direction becomes clear.",
            ),
        },
        (Planet::Sun, _, true) => match natal {
            Planet::Saturn => (
                "The Sun clashes with Saturn's restrictions. Leadership faces visible challenges and public scrutiny.",
                "CEO under pressure. Public accountability demands. Quarterly results closely scrutinized.",
            ),
            _ => (
                "The Sun creates friction that tests identity and leadership. Challenges are visible and demand response.",
                "Leadership tested publicly. Corporate identity questioned. Decisive action required.",
            ),
        },

        // ---- MOON transits (rare in scoring, Moon is usually skipped) ----
        (Planet::Moon, true, _) => (
            "The Moon's emotional energy flows harmoniously with this area. Public sentiment is supportive and market mood is positive.",
            "Short-term sentiment favorable. Retail investor interest increases. Consumer-facing catalysts may trigger.",
        ),
        (Planet::Moon, _, true) => (
            "The Moon's emotional energy creates friction. Market mood is volatile, and sentiment-driven selling is possible.",
            "Short-term sentiment headwinds. Avoid emotional trading decisions. Volatility normalizes within days.",
        ),

        // ---- Conjunction (neutral aspect, depends on planets) ----
        // Conjunctions handled by the harmonious/challenging branches above based on planet natures.
        // This catch-all handles the conjunction case where aspect_nature returns 0.
        (_, false, false) => match aspect {
            AspectType::Conjunction => {
                let meaning = format!(
                    "{} merges its energy with natal {}. This is a powerful fusion that amplifies both planets' themes, for better or worse.",
                    transit.name(), natal.name(),
                );
                return (meaning, "Concentrated energy in this area. Effects depend on the nature of both planets involved. Watch for intensity.".to_string());
            }
            _ => (
                "A subtle planetary interaction is present, adding nuance to the overall reading.",
                "Minor influence. Monitor but do not overweight in decision-making.",
            ),
        },
    };

    (meaning.to_string(), financial.to_string())
}

// ---------------------------------------------------------------------------
// Moon phase interpretation
// ---------------------------------------------------------------------------

fn interpret_moon_phase(phase_name: &str, phase_deg: f64) -> String {
    match phase_name {
        "New Moon" => format!(
            "New Moon ({:.0}°) — A powerful initiation window. New positions, new strategies, \
             and new product launches are cosmically supported. Plant seeds now; they will grow \
             over the coming two weeks. Energy favors bold beginnings.",
            phase_deg,
        ),
        "Waxing Crescent" => format!(
            "Waxing Crescent ({:.0}°) — Momentum is building. Early-stage initiatives gain \
             traction. This is the time to commit resources to what was started at the New Moon. \
             The market rewards forward movement.",
            phase_deg,
        ),
        "First Quarter" => format!(
            "First Quarter ({:.0}°) — A decision point. Challenges emerge that test the \
             viability of recent initiatives. Decisive action is required. Cut what isn't \
             working and double down on what is.",
            phase_deg,
        ),
        "Waxing Gibbous" => format!(
            "Waxing Gibbous ({:.0}°) — Refine and perfect. Momentum peaks soon. This is \
             the phase for optimization, fine-tuning positions, and preparing for maximum \
             visibility. Effort compounds.",
            phase_deg,
        ),
        "Full Moon" => format!(
            "Full Moon ({:.0}°) — Maximum illumination and maximum volatility. Culmination \
             of trends. Reversals are common. Take profits on positions that have matured. \
             Emotions run high in the market.",
            phase_deg,
        ),
        "Waning Gibbous" | "Disseminating" => format!(
            "Disseminating Moon ({:.0}°) — Share results, distribute gains. The peak has \
             passed; now is the time to harvest. Communicate value to stakeholders. \
             Reduce exposure to momentum plays.",
            phase_deg,
        ),
        "Last Quarter" | "Third Quarter" => format!(
            "Last Quarter ({:.0}°) — Re-evaluate and release. The cycle is winding down. \
             Let go of positions that have run their course. Prepare for the next cycle. \
             Defensive positioning favored.",
            phase_deg,
        ),
        "Waning Crescent" | "Balsamic" => format!(
            "Balsamic Moon ({:.0}°) — Endings and clearing. The darkest phase before renewal. \
             Close out losing positions, clear debt, and prepare clean balance sheets for the \
             next New Moon initiation window.",
            phase_deg,
        ),
        _ => format!(
            "Moon at {:.0}° — Lunar energy influences market sentiment and public perception. \
             Monitor emotional reactions in the market.",
            phase_deg,
        ),
    }
}

// ---------------------------------------------------------------------------
// Mercury retrograde interpretation
// ---------------------------------------------------------------------------

fn interpret_mercury_rx(moon_phase_deg: f64) -> String {
    // Mercury retrograde has three phases: pre-shadow, retrograde proper, post-shadow.
    // We don't track shadow periods yet, so we give the main retrograde interpretation.
    let base = "Mercury is retrograde — the planet of communication, commerce, and \
                contracts is moving backward through the zodiac. This is NOT a time to \
                sign major contracts, launch new products, or make binding agreements. \
                Delays, miscommunications, and technology failures are elevated. \
                Revisit, revise, and review existing strategies instead of launching new ones.";

    let timing_note = if moon_phase_deg < 45.0 {
        " Combined with the New/early Waxing Moon, new initiatives are doubly cautioned against."
    } else if moon_phase_deg > 165.0 && moon_phase_deg < 195.0 {
        " Combined with the Full Moon, volatility and confusion peak. Maximum caution advised."
    } else {
        " Use this period for due diligence, internal review, and strategy refinement."
    };

    format!("{}{}", base, timing_note)
}

// ---------------------------------------------------------------------------
// Dominant theme classifier
// ---------------------------------------------------------------------------

fn classify_dominant_theme(aspects: &[ActiveAspect], score: f32) -> String {
    if aspects.is_empty() {
        return "Quiet Period".to_string();
    }

    // Count benefic vs malefic energy
    let mut growth_score: f32 = 0.0;
    let mut caution_score: f32 = 0.0;
    let mut transformation_score: f32 = 0.0;
    let mut innovation_score: f32 = 0.0;

    for a in aspects {
        let delta = a.score_delta;

        // Theme classification by transit planet
        match a.transit_planet {
            Planet::Jupiter => {
                if delta > 0.0 { growth_score += delta; }
                else { caution_score += delta.abs(); }
            }
            Planet::Saturn => {
                if delta < 0.0 { caution_score += delta.abs(); }
                else { growth_score += delta * 0.5; } // Saturn harmonious = stable growth
            }
            Planet::Pluto => {
                transformation_score += delta.abs();
            }
            Planet::Uranus => {
                innovation_score += delta.abs();
            }
            Planet::Neptune => {
                if delta > 0.0 { innovation_score += delta * 0.5; }
                else { caution_score += delta.abs() * 0.5; }
            }
            Planet::Mars => {
                if delta > 0.0 { growth_score += delta * 0.7; }
                else { caution_score += delta.abs() * 0.7; }
            }
            Planet::Venus => {
                if delta > 0.0 { growth_score += delta * 0.8; }
                else { caution_score += delta.abs() * 0.3; }
            }
            Planet::NorthNode => {
                growth_score += delta.abs() * 0.6;
            }
            Planet::SouthNode => {
                caution_score += delta.abs() * 0.4;
            }
            _ => {
                if delta > 0.0 { growth_score += delta * 0.3; }
                else { caution_score += delta.abs() * 0.3; }
            }
        }
    }

    // Pick dominant theme
    let max_theme = [
        (growth_score, "Growth & Expansion"),
        (caution_score, "Caution & Restructuring"),
        (transformation_score, "Deep Transformation"),
        (innovation_score, "Innovation & Disruption"),
    ];

    let dominant = max_theme.iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(_, name)| *name)
        .unwrap_or("Neutral");

    // Reconcile theme with numeric score.
    //
    // The theme classifier counts aspect weights by planet type, while the
    // score sums signed deltas. They can disagree: e.g. many small Saturn
    // aspects pile up caution weight, but a few strong Jupiter trines push
    // the net score positive. When they conflict, the score is the source
    // of truth for polarity (bullish vs bearish), while the theme provides
    // the narrative flavor.
    //
    // Rule: score determines polarity. If the score is bullish (> 55) the
    // theme must be positive. If bearish (< 45) it must be negative.
    // The 45-55 neutral zone allows the theme classifier to decide freely.
    let reconciled = match (dominant, score as u32) {
        // Score bullish but theme says caution → override to growth
        ("Caution & Restructuring", 56..) => "Growth & Expansion",
        // Score bearish but theme says growth → override to caution
        ("Growth & Expansion", 0..=44) => "Caution & Restructuring",
        // Transformation/Innovation: let score set the polarity prefix
        // (handled in the intensity modifiers below)
        _ => dominant,
    };

    // Refine with intensity modifiers based on score magnitude
    match (reconciled, score as u32) {
        ("Growth & Expansion", 76..) => "Optimal Growth & Expansion".to_string(),
        ("Growth & Expansion", 56..=75) => "Growth & Expansion".to_string(),
        ("Growth & Expansion", _) => "Mild Growth & Expansion".to_string(),
        ("Caution & Restructuring", 0..=20) => "Extreme Caution & Restructuring".to_string(),
        ("Caution & Restructuring", 21..=35) => "Caution & Restructuring".to_string(),
        ("Caution & Restructuring", _) => "Mild Caution & Restructuring".to_string(),
        ("Deep Transformation", _) if score > 50.0 => "Constructive Transformation".to_string(),
        ("Deep Transformation", _) => "Destructive Transformation".to_string(),
        ("Innovation & Disruption", _) if score > 50.0 => "Positive Disruption".to_string(),
        ("Innovation & Disruption", _) => "Volatile Disruption".to_string(),
        _ => reconciled.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Timing window
// ---------------------------------------------------------------------------

fn compute_timing_window(aspects: &[ActiveAspect], score: f32) -> String {
    if aspects.is_empty() {
        return "No active transits — neutral window. Monitor for emerging aspects.".to_string();
    }

    let applying_count = aspects.iter().filter(|a| a.applying).count();
    let separating_count = aspects.len() - applying_count;

    // Tight orb aspects (< 2°) are near-exact — peak energy
    let tight_applying = aspects.iter()
        .filter(|a| a.applying && a.orb < 2.0)
        .count();
    let tight_favorable = aspects.iter()
        .filter(|a| a.applying && a.orb < 2.0 && a.score_delta > 0.0)
        .count();

    if score >= 70.0 && tight_favorable >= 2 {
        format!(
            "Strong favorable window active NOW. {} applying aspects near exact. \
             Act on opportunities within the next 1-2 weeks while energy peaks.",
            tight_applying,
        )
    } else if score >= 56.0 && applying_count > separating_count {
        format!(
            "Favorable energy building. {} of {} aspects are applying (approaching exact). \
             Conditions improve over the coming days. Good window for strategic action.",
            applying_count, aspects.len(),
        )
    } else if score >= 45.0 && score <= 55.0 {
        "Neutral window. Neither strongly favorable nor unfavorable. \
         Wait for directional clarity before making major moves.".to_string()
    } else if score < 45.0 && applying_count > separating_count {
        format!(
            "Challenging energy intensifying. {} applying aspects create increasing pressure. \
             Defensive positioning recommended. Wait for separating phase before acting.",
            applying_count,
        )
    } else if score < 45.0 && separating_count > applying_count {
        "Challenging period is fading. The worst pressure is passing. \
         Recovery window opens as aspects separate. Patience rewarded.".to_string()
    } else if separating_count > applying_count {
        "Current energy is dissipating. Effects of recent transits fading. \
         New cycle approaches. Begin planning for next transit window.".to_string()
    } else {
        "Mixed signals. Some energy building, some fading. \
         Selective action on high-conviction opportunities.".to_string()
    }
}

// ---------------------------------------------------------------------------
// Confidence computation
// ---------------------------------------------------------------------------

fn compute_confidence(aspects: &[ActiveAspect], score: f32) -> f32 {
    if aspects.is_empty() {
        return 20.0; // Low confidence when no aspects present
    }

    // Confidence factors:
    // 1. Number of strong aspects (|delta| > 5.0) — more = higher confidence
    let strong_count = aspects.iter().filter(|a| a.score_delta.abs() > 5.0).count();
    let strong_factor = (strong_count as f32 * 12.0).min(50.0);

    // 2. Agreement: do most aspects point the same direction?
    let positive = aspects.iter().filter(|a| a.score_delta > 0.0).count();
    let negative = aspects.iter().filter(|a| a.score_delta < 0.0).count();
    let total = (positive + negative).max(1) as f32;
    let agreement = ((positive.max(negative) as f32) / total) * 30.0;

    // 3. Score extremity: scores near 0 or 100 are higher confidence
    let extremity = ((score - 50.0).abs() / 50.0) * 20.0;

    (strong_factor + agreement + extremity).clamp(10.0, 100.0)
}

// ---------------------------------------------------------------------------
// Overall outlook synthesis
// ---------------------------------------------------------------------------

fn synthesize_outlook(
    theme: &str,
    key_transits: &[TransitInterpretation],
    moon_guidance: &str,
    mercury_warning: &Option<String>,
    score: f32,
) -> String {
    let score_desc = match score as u32 {
        0..=24  => "deeply unfavorable",
        25..=44 => "cautionary",
        45..=55 => "neutral",
        56..=75 => "favorable",
        _       => "strongly favorable",
    };

    let transit_summary = if key_transits.is_empty() {
        "No significant transits are active".to_string()
    } else if key_transits.len() == 1 {
        format!("The dominant transit is {}", key_transits[0].transit_desc)
    } else {
        format!(
            "The dominant transits are {} and {}",
            key_transits[0].transit_desc,
            key_transits[1].transit_desc,
        )
    };

    let mercury_note = if mercury_warning.is_some() {
        " Mercury retrograde adds a layer of uncertainty to all communications and agreements."
    } else {
        ""
    };

    // Extract first sentence of moon guidance
    let moon_short = moon_guidance.split('.').next().unwrap_or("Moon energy is present");

    format!(
        "The astrological outlook is {} with a dominant theme of {}. {}. \
         {}.{}",
        score_desc,
        theme,
        transit_summary,
        moon_short,
        mercury_note,
    )
}

// ---------------------------------------------------------------------------
// JSON serialization for DB storage
// ---------------------------------------------------------------------------

pub fn horoscope_to_json(reading: &HoroscopeReading) -> serde_json::Value {
    let transits: Vec<serde_json::Value> = reading.key_transits.iter().map(|t| {
        serde_json::json!({
            "transit_desc": t.transit_desc,
            "meaning": t.meaning,
            "financial_implication": t.financial_implication,
            "strength": t.strength,
        })
    }).collect();

    serde_json::json!({
        "overall_outlook": reading.overall_outlook,
        "dominant_theme": reading.dominant_theme,
        "key_transits": transits,
        "moon_guidance": reading.moon_guidance,
        "mercury_warning": reading.mercury_warning,
        "timing_window": reading.timing_window,
        "confidence": (reading.confidence * 10.0).round() / 10.0,
    })
}

pub fn horoscope_from_json(ticker: &str, date: NaiveDate, val: &serde_json::Value) -> Option<HoroscopeReading> {
    let overall_outlook = val["overall_outlook"].as_str()?.to_string();
    let dominant_theme = val["dominant_theme"].as_str()?.to_string();
    let moon_guidance = val["moon_guidance"].as_str()?.to_string();
    let mercury_warning = val["mercury_warning"].as_str().map(|s| s.to_string());
    let timing_window = val["timing_window"].as_str()?.to_string();
    let confidence = val["confidence"].as_f64().unwrap_or(50.0) as f32;

    let key_transits = val["key_transits"].as_array()
        .map(|arr| arr.iter().filter_map(|t| {
            Some(TransitInterpretation {
                transit_desc: t["transit_desc"].as_str()?.to_string(),
                meaning: t["meaning"].as_str()?.to_string(),
                financial_implication: t["financial_implication"].as_str()?.to_string(),
                strength: t["strength"].as_str()?.to_string(),
            })
        }).collect())
        .unwrap_or_default();

    Some(HoroscopeReading {
        ticker: ticker.to_string(),
        date,
        overall_outlook,
        dominant_theme,
        key_transits,
        moon_guidance,
        mercury_warning,
        timing_window,
        confidence,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astrology::natal::{compute_transit_score, NatalChart};

    fn msft_ipo() -> NaiveDate {
        NaiveDate::from_ymd_opt(1986, 3, 13).unwrap()
    }

    #[test]
    fn test_horoscope_has_all_fields() {
        let chart = NatalChart::compute("MSFT", msft_ipo());
        let score = compute_transit_score(&chart, NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
        let reading = generate_horoscope(&score);

        assert_eq!(reading.ticker, "MSFT");
        assert!(!reading.overall_outlook.is_empty(), "overall_outlook should not be empty");
        assert!(!reading.dominant_theme.is_empty(), "dominant_theme should not be empty");
        assert!(!reading.moon_guidance.is_empty(), "moon_guidance should not be empty");
        assert!(!reading.timing_window.is_empty(), "timing_window should not be empty");
        assert!(reading.confidence >= 10.0 && reading.confidence <= 100.0,
            "confidence {} out of range", reading.confidence);
    }

    #[test]
    fn test_horoscope_key_transits() {
        let chart = NatalChart::compute("AAPL", NaiveDate::from_ymd_opt(1980, 12, 12).unwrap());
        let score = compute_transit_score(&chart, NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
        let reading = generate_horoscope(&score);

        // Should have key transits if aspects exist
        if !score.active_aspects.is_empty() {
            assert!(!reading.key_transits.is_empty(), "should have key transits when aspects exist");
            for t in &reading.key_transits {
                assert!(!t.transit_desc.is_empty());
                assert!(!t.meaning.is_empty());
                assert!(!t.financial_implication.is_empty());
                assert!(!t.strength.is_empty());
            }
        }
    }

    #[test]
    fn test_horoscope_json_roundtrip() {
        let chart = NatalChart::compute("TSLA", NaiveDate::from_ymd_opt(2010, 6, 29).unwrap());
        let score = compute_transit_score(&chart, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        let reading = generate_horoscope(&score);

        let json = horoscope_to_json(&reading);
        let back = horoscope_from_json("TSLA", reading.date, &json);
        assert!(back.is_some(), "should deserialize from JSON");

        let back = back.unwrap();
        assert_eq!(back.overall_outlook, reading.overall_outlook);
        assert_eq!(back.dominant_theme, reading.dominant_theme);
        assert_eq!(back.key_transits.len(), reading.key_transits.len());
        assert_eq!(back.timing_window, reading.timing_window);
    }

    #[test]
    fn test_moon_phase_interpretation() {
        let full = interpret_moon_phase("Full Moon", 180.0);
        assert!(full.contains("Full Moon"), "should reference the phase name");
        assert!(full.contains("volatility") || full.contains("Reversals"),
            "Full Moon should mention volatility");

        let new = interpret_moon_phase("New Moon", 5.0);
        assert!(new.contains("initiation") || new.contains("New"),
            "New Moon should mention initiation");
    }

    #[test]
    fn test_mercury_rx_interpretation() {
        let warning = interpret_mercury_rx(180.0);
        assert!(warning.contains("retrograde"), "should mention retrograde");
        assert!(warning.contains("Full Moon"), "should note Full Moon combination");
    }

    #[test]
    fn test_dominant_theme_classifier() {
        // Empty aspects = Quiet Period
        let theme = classify_dominant_theme(&[], 50.0);
        assert_eq!(theme, "Quiet Period");
    }

    #[test]
    fn test_confidence_no_aspects() {
        let conf = compute_confidence(&[], 50.0);
        assert_eq!(conf, 20.0, "no aspects = low confidence");
    }

    #[test]
    fn test_timing_window_no_aspects() {
        let tw = compute_timing_window(&[], 50.0);
        assert!(tw.contains("No active transits"));
    }
}
