//! Natal chart derivation and transit-to-natal scoring.
//!
//! A "natal chart" is just a snapshot of planetary positions at a company's IPO.
//! Transit scoring compares today's positions against that snapshot.

use chrono::NaiveDate;

use super::aspects::{
    find_aspect, moon_phase_modifier, score_aspect, ActiveAspect, MERCURY_RX_CAP,
};
use super::ephemeris::{
    date_to_jdn, is_retrograde, jdn_to_t, longitude_to_sign, moon_phase_angle,
    moon_phase_name, snapshot_all, Planet, PlanetSnapshot,
};

// ---------------------------------------------------------------------------
// Natal chart
// ---------------------------------------------------------------------------

pub struct NatalChart {
    pub ticker:     String,
    pub ipo_date:   NaiveDate,
    pub positions:  Vec<PlanetSnapshot>,
}

impl NatalChart {
    /// Compute the natal chart for a company using its IPO date.
    /// Time is 09:30 EST = 14:30 UTC = 14.5 fractional hours.
    pub fn compute(ticker: &str, ipo_date: NaiveDate) -> Self {
        let jdn = date_to_jdn(
            ipo_date.year(),
            ipo_date.month(),
            ipo_date.day(),
            14.5, // 09:30 EST in UTC
        );
        NatalChart {
            ticker: ticker.to_string(),
            ipo_date,
            positions: snapshot_all(jdn),
        }
    }

    pub fn longitude_of(&self, planet: Planet) -> Option<f64> {
        self.positions.iter()
            .find(|p| p.planet == planet)
            .map(|p| p.longitude)
    }
}

// ---------------------------------------------------------------------------
// Transit score computation
// ---------------------------------------------------------------------------

pub struct TransitScore {
    pub ticker:         String,
    pub score_date:     NaiveDate,
    pub astro_score:    f32,
    pub astro_label:    String,
    pub moon_phase:     String,
    pub moon_phase_deg: f64,
    pub mercury_rx:     bool,
    pub active_aspects: Vec<ActiveAspect>,
}

/// Compute the astrological score for a ticker on a given date.
pub fn compute_transit_score(natal: &NatalChart, score_date: NaiveDate) -> TransitScore {
    let jdn = date_to_jdn(
        score_date.year(),
        score_date.month(),
        score_date.day(),
        14.5,
    );
    let t = jdn_to_t(jdn);

    // Today's planetary positions
    let transits = snapshot_all(jdn);

    // Moon phase
    let moon_phase_deg = moon_phase_angle(t);
    let moon_phase     = moon_phase_name(moon_phase_deg).to_string();
    let moon_mod       = moon_phase_modifier(moon_phase_deg);

    // Mercury retrograde?
    let mercury_rx = is_retrograde(Planet::Mercury, jdn);

    // Find all active aspects: every transit planet vs every natal planet
    let mut active_aspects: Vec<ActiveAspect> = Vec::new();
    let mut delta_sum: f32 = 0.0;

    for transit in &transits {
        // Skip Moon transits for natal comparison (moves too fast, too noisy)
        if transit.planet == Planet::Moon { continue; }

        for natal_pos in &natal.positions {
            // Skip comparing a planet to itself
            if transit.planet == natal_pos.planet { continue; }

            if let Some((aspect_type, orb)) = find_aspect(transit.longitude, natal_pos.longitude) {
                let delta = score_aspect(transit.planet, natal_pos.planet, aspect_type, orb);
                delta_sum += delta;

                let (transit_sign, _) = longitude_to_sign(transit.longitude);
                let (natal_sign, _)   = longitude_to_sign(natal_pos.longitude);

                active_aspects.push(ActiveAspect {
                    transit_planet: transit.planet,
                    transit_sign:   transit_sign.to_string(),
                    natal_planet:   natal_pos.planet,
                    natal_sign:     natal_sign.to_string(),
                    aspect:         aspect_type,
                    orb,
                    score_delta:    delta,
                });
            }
        }
    }

    // Sort aspects by magnitude (most significant first)
    active_aspects.sort_by(|a, b| {
        b.score_delta.abs().partial_cmp(&a.score_delta.abs()).unwrap()
    });

    // Composite score
    let base_score = (50.0_f32 + delta_sum + moon_mod).clamp(0.0, 100.0);
    let astro_score = if mercury_rx {
        base_score.min(MERCURY_RX_CAP)
    } else {
        base_score
    };

    let astro_label = score_label(astro_score).to_string();

    TransitScore {
        ticker: natal.ticker.clone(),
        score_date,
        astro_score,
        astro_label,
        moon_phase,
        moon_phase_deg,
        mercury_rx,
        active_aspects,
    }
}

fn score_label(score: f32) -> &'static str {
    match score as u32 {
        0..=24  => "Extreme Fear",
        25..=44 => "Fear",
        45..=55 => "Neutral",
        56..=75 => "Greed",
        _       => "Extreme Greed",
    }
}

// ---------------------------------------------------------------------------
// Serialize active_aspects → serde_json::Value for DB storage
// ---------------------------------------------------------------------------

pub fn aspects_to_json(aspects: &[ActiveAspect]) -> serde_json::Value {
    let arr: Vec<serde_json::Value> = aspects.iter().map(|a| {
        serde_json::json!({
            "transit_planet": a.transit_planet.name(),
            "transit_sign":   a.transit_sign,
            "natal_planet":   a.natal_planet.name(),
            "natal_sign":     a.natal_sign,
            "aspect":         a.aspect.name(),
            "aspect_symbol":  a.aspect.symbol(),
            "orb":            (a.orb * 10.0).round() / 10.0,
            "score_delta":    (a.score_delta * 10.0).round() / 10.0,
            "effect":         a.effect_label(),
        })
    }).collect();
    serde_json::Value::Array(arr)
}

// ---------------------------------------------------------------------------
// Deserialize active_aspects from DB JSONB → Vec<ActiveAspect>
// (used by the dashboard to display the transits table)
// ---------------------------------------------------------------------------

pub fn aspects_from_json(val: &serde_json::Value) -> Vec<ActiveAspect> {
    let arr = match val.as_array() {
        Some(a) => a,
        None => return vec![],
    };
    arr.iter().filter_map(|obj| {
        let transit_planet = Planet::from_name(obj["transit_planet"].as_str()?)?;
        let natal_planet   = Planet::from_name(obj["natal_planet"].as_str()?)?;
        let transit_sign   = obj["transit_sign"].as_str()?.to_string();
        let natal_sign     = obj["natal_sign"].as_str()?.to_string();
        let aspect = match obj["aspect"].as_str()? {
            "Conjunction" => super::aspects::AspectType::Conjunction,
            "Sextile"     => super::aspects::AspectType::Sextile,
            "Square"      => super::aspects::AspectType::Square,
            "Trine"       => super::aspects::AspectType::Trine,
            "Opposition"  => super::aspects::AspectType::Opposition,
            _             => return None,
        };
        let orb         = obj["orb"].as_f64().unwrap_or(0.0);
        let score_delta = obj["score_delta"].as_f64().unwrap_or(0.0) as f32;
        Some(ActiveAspect { transit_planet, transit_sign, natal_planet, natal_sign, aspect, orb, score_delta })
    }).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn msft_ipo() -> NaiveDate {
        NaiveDate::from_ymd_opt(1986, 3, 13).unwrap()
    }

    #[test]
    fn test_natal_chart_has_all_planets() {
        let chart = NatalChart::compute("MSFT", msft_ipo());
        assert_eq!(chart.positions.len(), 10); // all 10 planets
    }

    #[test]
    fn test_natal_sun_in_pisces() {
        // MSFT IPO: March 13, 1986 — Sun should be in Pisces (~22°)
        let chart = NatalChart::compute("MSFT", msft_ipo());
        let sun_lon = chart.longitude_of(Planet::Sun).expect("Sun position missing");
        let (sign, deg) = longitude_to_sign(sun_lon);
        assert_eq!(sign, "Pisces", "MSFT natal Sun should be in Pisces, got {sign} {deg:.1}°");
    }

    #[test]
    fn test_transit_score_in_range() {
        let chart = NatalChart::compute("MSFT", msft_ipo());
        let today = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let score = compute_transit_score(&chart, today);
        assert!(score.astro_score >= 0.0 && score.astro_score <= 100.0,
            "Score out of range: {}", score.astro_score);
        assert!(!score.moon_phase.is_empty());
    }

    #[test]
    fn test_aspects_roundtrip() {
        let chart = NatalChart::compute("TSLA", NaiveDate::from_ymd_opt(2010, 6, 29).unwrap());
        let score = compute_transit_score(&chart, NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
        let json  = aspects_to_json(&score.active_aspects);
        let back  = aspects_from_json(&json);
        assert_eq!(score.active_aspects.len(), back.len());
    }
}

// Bring NaiveDate methods into scope
use chrono::Datelike;
