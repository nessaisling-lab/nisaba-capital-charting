//! Fixed-star activations (Wave 6.B3).
//!
//! Eight royal/financially-significant stars with hardcoded J2000 ecliptic
//! longitudes. Each year the constellations precess ~50.29″ ≈ 0.01397°
//! along the ecliptic, so we apply a simple linear correction from J2000
//! to the date being scored. Within astrological orbs (1°) this is
//! sufficient precision — for arcsecond accuracy, swap in `swe_fixstar2`
//! via raw FFI later.
//!
//! Activation = a transit planet conjuncts a fixed star within 1° orb.
//! Each star carries an archetypal score contribution (positive or
//! negative) added to delta_sum before sigmoid normalization.

use chrono::{Datelike, NaiveDate};
use super::ephemeris::{Planet, PlanetSnapshot};

/// Maximum orb (degrees) for a fixed-star activation.
const STAR_ORB: f64 = 1.0;

/// Annual precession rate in degrees (50.29″ / year).
const PRECESSION_DEG_PER_YEAR: f64 = 50.29 / 3600.0;

#[derive(Debug, Clone, Copy)]
pub struct FixedStar {
    pub name:      &'static str,
    /// Ecliptic longitude at J2000.0 (2000-01-01 12:00 TT).
    pub lon_j2000: f64,
    /// Visual magnitude (lower = brighter).
    pub magnitude: f32,
    /// Score contribution to delta_sum when conjuncted by a transit planet.
    pub strength:  f32,
    /// One-line archetypal meaning for UI display.
    pub archetype: &'static str,
}

/// Catalog of 8 financially-significant fixed stars.
///
/// Selection rationale: traditional "royal stars" (Regulus, Aldebaran,
/// Antares, Fomalhaut) plus stars with strong wealth/danger associations
/// in financial astrology (Spica, Algol, Sirius, Vega).
pub const FIXED_STARS: &[FixedStar] = &[
    FixedStar { name: "Regulus",   lon_j2000: 149.833, magnitude: 1.4, strength:  10.0, archetype: "Kingship, success in finance" },
    FixedStar { name: "Spica",     lon_j2000: 203.883, magnitude: 1.0, strength:  12.0, archetype: "Wealth, abundance" },
    FixedStar { name: "Antares",   lon_j2000: 249.833, magnitude: 1.0, strength:   8.0, archetype: "Leadership, passion, military/finance" },
    FixedStar { name: "Aldebaran", lon_j2000:  69.933, magnitude: 0.9, strength:   6.0, archetype: "Honors, military, recognition" },
    FixedStar { name: "Sirius",    lon_j2000: 104.283, magnitude: -1.5, strength:  8.0, archetype: "Fame, media attention" },
    FixedStar { name: "Vega",      lon_j2000: 285.383, magnitude: 0.0, strength:   5.0, archetype: "Artistry, IP, creative success" },
    FixedStar { name: "Fomalhaut", lon_j2000: 334.083, magnitude: 1.2, strength:   4.0, archetype: "Transformation, dreams" },
    FixedStar { name: "Algol",     lon_j2000:  56.350, magnitude: 2.1, strength: -14.0, archetype: "Sudden loss, danger" },
];

/// Precessed longitude of a fixed star at the given date.
pub fn star_longitude(star: &FixedStar, date: NaiveDate) -> f64 {
    let years_since_j2000 = (date.year() as f64 - 2000.0)
        + (date.ordinal() as f64 / 365.25);
    (star.lon_j2000 + PRECESSION_DEG_PER_YEAR * years_since_j2000).rem_euclid(360.0)
}

/// One detected activation: transit planet conjuncts a fixed star within orb.
#[derive(Debug, Clone)]
pub struct StarActivation {
    pub planet:     Planet,
    pub star_name:  &'static str,
    pub orb:        f64,
    pub strength:   f32,
    pub archetype:  &'static str,
}

/// Detect all fixed-star conjunctions in current transit positions.
pub fn detect_activations(
    transits: &[PlanetSnapshot],
    date: NaiveDate,
) -> Vec<StarActivation> {
    let mut out = Vec::new();
    for transit in transits {
        // Skip Moon — too fast, would activate too often
        if transit.planet == Planet::Moon { continue; }
        for star in FIXED_STARS {
            let star_lon = star_longitude(star, date);
            let mut diff = (transit.longitude - star_lon).abs() % 360.0;
            if diff > 180.0 { diff = 360.0 - diff; }
            if diff <= STAR_ORB {
                // Tightness scales strength: at 0° orb, full; at 1° orb, half
                let tightness = 1.0 - (diff / STAR_ORB) * 0.5;
                out.push(StarActivation {
                    planet: transit.planet,
                    star_name: star.name,
                    orb: diff,
                    strength: star.strength * tightness as f32,
                    archetype: star.archetype,
                });
            }
        }
    }
    out
}

/// Sum of all star activation strengths for delta_sum contribution.
pub fn activation_score_total(activations: &[StarActivation]) -> f32 {
    activations.iter().map(|a| a.strength).sum()
}

/// JSON serialization for DB storage alongside aspects/patterns.
pub fn activations_to_json(activations: &[StarActivation]) -> serde_json::Value {
    let arr: Vec<serde_json::Value> = activations.iter().map(|a| {
        serde_json::json!({
            "planet":    a.planet.name(),
            "star":      a.star_name,
            "orb":       (a.orb * 100.0).round() / 100.0,
            "strength":  (a.strength * 10.0).round() / 10.0,
            "archetype": a.archetype,
        })
    }).collect();
    serde_json::Value::Array(arr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn snap(planet: Planet, lon: f64) -> PlanetSnapshot {
        PlanetSnapshot {
            planet,
            longitude: lon,
            sign: super::super::ephemeris::longitude_to_sign(lon).0,
            degree: super::super::ephemeris::longitude_to_sign(lon).1,
            retrograde: false,
        }
    }

    #[test]
    fn precession_advances_with_year() {
        let s = &FIXED_STARS[0]; // Regulus
        let l_2000 = star_longitude(s, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let l_2026 = star_longitude(s, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let drift = l_2026 - l_2000;
        // 26 years × 0.01397°/year ≈ 0.363°
        assert!((drift - 0.363).abs() < 0.01, "Expected ~0.363° drift, got {drift}");
    }

    #[test]
    fn detects_conjunction_within_orb() {
        // Sun at 150° (Leo 0°) — close to Regulus (~150.2° at 2026)
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let transits = vec![snap(Planet::Sun, 150.2)];
        let acts = detect_activations(&transits, date);
        assert!(acts.iter().any(|a| a.star_name == "Regulus"),
            "Expected Regulus activation, got {acts:?}");
    }

    #[test]
    fn no_activation_outside_orb() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        // Sun at 152° — 2° away from Regulus, outside 1° orb
        let transits = vec![snap(Planet::Sun, 152.0)];
        let acts = detect_activations(&transits, date);
        assert!(acts.iter().find(|a| a.star_name == "Regulus").is_none());
    }

    #[test]
    fn moon_skipped() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let transits = vec![snap(Planet::Moon, 150.2)];
        let acts = detect_activations(&transits, date);
        assert!(acts.is_empty(), "Moon should be skipped, got {acts:?}");
    }

    #[test]
    fn algol_negative_strength() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        // Algol at ~56.7° in 2026
        let transits = vec![snap(Planet::Mars, 56.5)];
        let acts = detect_activations(&transits, date);
        let algol = acts.iter().find(|a| a.star_name == "Algol")
            .expect("Algol should activate");
        assert!(algol.strength < 0.0, "Algol should be negative, got {}", algol.strength);
    }
}
