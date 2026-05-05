//! Natal chart derivation and transit-to-natal scoring.
//!
//! A "natal chart" is just a snapshot of planetary positions at a company's IPO.
//! Transit scoring compares today's positions against that snapshot.

use chrono::NaiveDate;

use std::sync::Mutex;

use super::aspects::{
    find_aspect, is_applying, moon_phase_modifier, planetary_dignity, score_aspect_v2,
    ActiveAspect, DignityState, APPLYING_MULTIPLIER, MERCURY_RX_CAP, SEPARATING_MULTIPLIER,
};
use super::ephemeris::{
    date_to_jdn, jdn_to_t, longitude_to_sign, moon_phase_angle,
    moon_phase_name, snapshot_all, Planet, PlanetSnapshot,
};
use super::swisseph_bridge;

/// Global mutex for Swiss Ephemeris calls (C library is not thread-safe).
static SWE_LOCK: Mutex<()> = Mutex::new(());

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
    ///
    /// Uses Swiss Ephemeris for sub-arcsecond accuracy (all 13 bodies).
    /// Falls back to Meeus (10 classical planets only) if Swiss Eph fails.
    pub fn compute(ticker: &str, ipo_date: NaiveDate) -> Self {
        let jdn = date_to_jdn(
            ipo_date.year(),
            ipo_date.month(),
            ipo_date.day(),
            14.5, // 09:30 EST in UTC
        );
        let positions = {
            let _lock = SWE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let precise = swisseph_bridge::snapshot_all_precise(jdn);
            if precise.len() >= 10 { precise } else { snapshot_all(jdn) }
        };
        NatalChart {
            ticker: ticker.to_string(),
            ipo_date,
            positions,
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
    /// v11.4 (Wave 6.B1) — geometric patterns detected from natal+transit positions.
    pub patterns:       Vec<super::patterns::AspectPattern>,
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

    // Today's planetary positions (Swiss Ephemeris, all 13 bodies)
    let transits = {
        let _lock = SWE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let precise = swisseph_bridge::snapshot_all_precise(jdn);
        if precise.len() >= 10 { precise } else { snapshot_all(jdn) }
    };

    // Moon phase (use Swiss Eph Moon position for accuracy)
    let moon_lon = transits.iter()
        .find(|s| s.planet == Planet::Moon)
        .map(|s| s.longitude);
    let sun_lon = transits.iter()
        .find(|s| s.planet == Planet::Sun)
        .map(|s| s.longitude);
    let moon_phase_deg = match (moon_lon, sun_lon) {
        (Some(m), Some(s)) => super::ephemeris::norm360(m - s),
        _ => moon_phase_angle(t), // fallback to Meeus
    };
    let moon_phase     = moon_phase_name(moon_phase_deg).to_string();
    let moon_mod       = moon_phase_modifier(moon_phase_deg);

    // Mercury retrograde? Check from snapshot (speed-based via Swiss Eph)
    let mercury_rx = transits.iter()
        .find(|s| s.planet == Planet::Mercury)
        .map(|s| s.retrograde)
        .unwrap_or(false);

    // Find all active aspects: every transit planet vs every natal planet
    let mut active_aspects: Vec<ActiveAspect> = Vec::new();
    let mut delta_sum: f32 = 0.0;

    // Get longitude speeds for applying/separating detection
    let _lock = SWE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let speeds: std::collections::HashMap<Planet, f64> = transits.iter()
        .filter_map(|s| swisseph_bridge::longitude_speed(s.planet, jdn).map(|spd| (s.planet, spd)))
        .collect();
    drop(_lock);

    for transit in &transits {
        // Skip Moon transits for natal comparison (moves too fast, too noisy)
        if transit.planet == Planet::Moon { continue; }

        let transit_speed = speeds.get(&transit.planet).copied();
        let (transit_sign, _) = longitude_to_sign(transit.longitude);

        // Compute transit planet's dignity in its current sign
        let dignity = planetary_dignity(transit.planet, transit_sign);

        for natal_pos in &natal.positions {
            // Skip comparing a planet to itself
            if transit.planet == natal_pos.planet { continue; }

            if let Some((aspect_type, orb)) = find_aspect(transit.longitude, natal_pos.longitude) {
                // Determine applying vs separating
                let applying = is_applying(
                    transit.longitude,
                    natal_pos.longitude,
                    transit_speed,
                    aspect_type.angle(),
                );

                // Score with v6.B2 full modifier stack: dignity, minor reduction,
                // body weighting, out-of-sign penalty, mutual reception bonus.
                let (natal_sign_str, _) = longitude_to_sign(natal_pos.longitude);
                let mut delta = score_aspect_v2(
                    transit.planet,
                    natal_pos.planet,
                    aspect_type,
                    orb,
                    transit_speed,
                    Some(transit_sign),
                    Some(transit.longitude),
                    Some(natal_pos.longitude),
                    Some(natal_sign_str),
                );

                // Apply applying/separating multiplier on top
                let apply_mult = if applying { APPLYING_MULTIPLIER } else { SEPARATING_MULTIPLIER };
                delta = (delta as f64 * apply_mult) as f32;

                delta_sum += delta;

                let (natal_sign, _) = longitude_to_sign(natal_pos.longitude);

                active_aspects.push(ActiveAspect {
                    transit_planet: transit.planet,
                    transit_sign:   transit_sign.to_string(),
                    natal_planet:   natal_pos.planet,
                    natal_sign:     natal_sign.to_string(),
                    aspect:         aspect_type,
                    orb,
                    score_delta:    delta,
                    applying,
                    dignity,
                });
            }
        }
    }

    // Sort aspects by magnitude (most significant first)
    active_aspects.sort_by(|a, b| {
        b.score_delta.abs().partial_cmp(&a.score_delta.abs()).unwrap_or(std::cmp::Ordering::Equal)
    });

    // v11.4 (Wave 6.B1) — aspect pattern bonus added to delta_sum.
    // Patterns contribute pre-normalization so they meaningfully shift the
    // sigmoid input without being washed out by aspect_count division.
    let patterns = super::patterns::detect_patterns(&natal.positions, &transits);
    let pattern_delta = super::patterns::pattern_score_total(&patterns);
    delta_sum += pattern_delta;

    // Composite score — normalized sigmoid
    //
    // The raw delta_sum can swing to ±300 with 50+ aspects per ticker.
    // Feeding raw values into a sigmoid causes bimodal clustering at 0/100.
    //
    // Fix: normalize delta_sum by the number of contributing aspects to get
    // the average delta per aspect. This means a ticker with 70 weak aspects
    // doesn't score more extremely than one with 20 strong aspects.
    //
    // The normalized value (typically ±2 to ±8) is then fed into a sigmoid
    // with k=0.30, mapping to (0, 100):
    //   score = 100 / (1 + e^(-k * normalized_x))
    //
    // At normalized_x=0 → 50, ±3 → ~71/29, ±6 → ~86/14, ±10 → ~95/5.
    // This produces a bell-shaped distribution centered around 50.
    let aspect_count = active_aspects.len().max(1) as f64;
    let raw_x = (delta_sum + moon_mod) as f64;
    let normalized_x = raw_x / aspect_count.sqrt();
    const SIGMOID_K: f64 = 0.10;
    let base_score = (100.0 / (1.0 + (-SIGMOID_K * normalized_x).exp())) as f32;
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
        patterns,
    }
}

/// v11.4 (Wave 6.B1) — serialize patterns to JSON for DB storage.
pub fn patterns_to_json(patterns: &[super::patterns::AspectPattern]) -> serde_json::Value {
    let arr: Vec<serde_json::Value> = patterns.iter().map(|p| {
        let bodies: Vec<String> = p.bodies.iter().map(|b| b.name().to_string()).collect();
        serde_json::json!({
            "kind":     p.kind.name(),
            "symbol":   p.kind.symbol(),
            "bodies":   bodies,
            "avg_orb":  (p.avg_orb * 10.0).round() / 10.0,
            "strength": (p.strength * 10.0).round() / 10.0,
            "is_cross": p.is_cross,
        })
    }).collect();
    serde_json::Value::Array(arr)
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
            "applying":       a.applying,
            "dignity":        a.dignity.name(),
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
            "Conjunction"    => super::aspects::AspectType::Conjunction,
            "SemiSextile"    => super::aspects::AspectType::SemiSextile,
            "SemiSquare"     => super::aspects::AspectType::SemiSquare,
            "Sextile"        => super::aspects::AspectType::Sextile,
            "Square"         => super::aspects::AspectType::Square,
            "Trine"          => super::aspects::AspectType::Trine,
            "Sesquiquadrate" => super::aspects::AspectType::Sesquiquadrate,
            "Quincunx"       => super::aspects::AspectType::Quincunx,
            "Opposition"     => super::aspects::AspectType::Opposition,
            _                => return None,
        };
        let orb         = obj["orb"].as_f64().unwrap_or(0.0);
        let score_delta = obj["score_delta"].as_f64().unwrap_or(0.0) as f32;
        let applying    = obj["applying"].as_bool().unwrap_or(true);
        let dignity = match obj["dignity"].as_str() {
            Some("Domicile")  => DignityState::Domicile,
            Some("Exalted")   => DignityState::Exaltation,
            Some("Detriment") => DignityState::Detriment,
            Some("Fall")      => DignityState::Fall,
            _                 => DignityState::Peregrine,
        };
        Some(ActiveAspect {
            transit_planet, transit_sign, natal_planet, natal_sign,
            aspect, orb, score_delta, applying, dignity,
        })
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
        let _guard = super::swisseph_bridge::SWE_TEST_LOCK.lock().unwrap();
        let chart = NatalChart::compute("MSFT", msft_ipo());
        // 12-13 with Swiss Eph (10 classical + nodes + maybe Chiron),
        // or 10 with Meeus fallback
        assert!(
            chart.positions.len() >= 10,
            "Expected at least 10 planets, got {}",
            chart.positions.len(),
        );
    }

    #[test]
    fn test_natal_sun_in_pisces() {
        let _guard = super::swisseph_bridge::SWE_TEST_LOCK.lock().unwrap();
        // MSFT IPO: March 13, 1986 — Sun should be in Pisces (~22°)
        let chart = NatalChart::compute("MSFT", msft_ipo());
        let sun_lon = chart.longitude_of(Planet::Sun).expect("Sun position missing");
        let (sign, deg) = longitude_to_sign(sun_lon);
        assert_eq!(sign, "Pisces", "MSFT natal Sun should be in Pisces, got {sign} {deg:.1}°");
    }

    #[test]
    fn test_transit_score_in_range() {
        let _guard = super::swisseph_bridge::SWE_TEST_LOCK.lock().unwrap();
        let chart = NatalChart::compute("MSFT", msft_ipo());
        let today = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let score = compute_transit_score(&chart, today);
        assert!(score.astro_score >= 0.0 && score.astro_score <= 100.0,
            "Score out of range: {}", score.astro_score);
        assert!(!score.moon_phase.is_empty());
    }

    #[test]
    fn test_aspects_roundtrip() {
        let _guard = super::swisseph_bridge::SWE_TEST_LOCK.lock().unwrap();
        let chart = NatalChart::compute("TSLA", NaiveDate::from_ymd_opt(2010, 6, 29).unwrap());
        let score = compute_transit_score(&chart, NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
        let json  = aspects_to_json(&score.active_aspects);
        let back  = aspects_from_json(&json);
        assert_eq!(score.active_aspects.len(), back.len());
    }
}

// Bring NaiveDate methods into scope
use chrono::Datelike;
