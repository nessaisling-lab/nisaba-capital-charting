//! Eclipse activation detection (Wave 6.B4).
//!
//! When an eclipse's ecliptic longitude falls within 6° of any natal
//! planet, that planet is "activated" by the eclipse — its themes are
//! amplified for ~6 months. Solar eclipses near a natal planet drive
//! external/identity events; lunar eclipses drive emotional/relationship
//! events.
//!
//! Saros series: each eclipse belongs to a numbered family that repeats
//! every 18y 11d 8h. Tracking the Saros lets us cross-reference what
//! happened to the ticker last time this same family hit (e.g.
//! AAPL's natal Sun was hit by Saros 142 in 2008).
//!
//! Eclipse data is loaded from the `eclipses` table (seeded by migration
//! 0041 with NASA Five-Millennium Catalog entries 2025-2028).

use chrono::NaiveDate;
use super::ephemeris::{Planet, PlanetSnapshot};

const ECLIPSE_ORB: f64 = 6.0;
/// Score contribution per activation. Solar = identity-driving, larger
/// magnitude. Lunar = emotional/relational, smaller magnitude.
const SOLAR_STRENGTH: f32 = -10.0; // eclipses tend toward stress/disruption
const LUNAR_STRENGTH: f32 = -6.0;

#[derive(Debug, Clone)]
pub struct Eclipse {
    pub date:         NaiveDate,
    pub eclipse_type: String,
    pub longitude:    f64,
    pub magnitude:    Option<f64>,
    pub saros_series: Option<i32>,
    pub notes:        Option<String>,
}

/// Hardcoded eclipse catalog (NASA Five-Millennium Catalog 2025-2028).
/// Mirrors migration 0041 seed data so transit scoring works without
/// requiring a DB round-trip. Update both when adding entries.
pub fn upcoming_eclipses() -> Vec<Eclipse> {
    let entries: &[(&str, &str, f64, f64, i32, &str)] = &[
        ("2025-03-14", "lunar_total",     173.95, 1.180, 123, "Worm/Pi Day blood moon"),
        ("2025-03-29", "solar_partial",     8.87, 0.938, 149, "Atlantic + N. Europe"),
        ("2025-09-07", "lunar_total",     165.04, 1.367, 128, "Asia + Africa"),
        ("2025-09-21", "solar_partial",   178.85, 0.855, 154, "S. Pacific + Antarctica"),
        ("2026-02-17", "solar_annular",   328.72, 0.963, 121, "Antarctica"),
        ("2026-03-03", "lunar_total",     142.65, 1.151, 133, "Pacific"),
        ("2026-08-12", "solar_total",     140.05, 1.039, 126, "Greenland/Iceland/Spain"),
        ("2026-08-28", "lunar_partial",   335.20, 0.930, 138, "Americas"),
        ("2027-02-06", "solar_annular",   317.18, 0.928, 131, "S. America/Africa"),
        ("2027-02-21", "lunar_penumbral", 332.48, 0.927, 143, ""),
        ("2027-07-18", "lunar_penumbral", 295.62, 0.038, 110, ""),
        ("2027-08-02", "solar_total",     129.87, 1.079, 136, "Spain/Egypt path"),
        ("2027-08-17", "lunar_penumbral", 324.87, 0.140, 148, ""),
        ("2028-01-12", "solar_annular",   291.58, 0.920, 141, "N. America"),
        ("2028-01-26", "lunar_total",     126.10, 1.082, 153, ""),
        ("2028-07-22", "solar_total",     119.62, 1.056, 146, "Australia"),
        ("2028-08-06", "lunar_partial",   314.19, 0.392, 158, ""),
    ];
    entries.iter().map(|(d, ty, lon, mag, saros, notes)| {
        Eclipse {
            date: NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap(),
            eclipse_type: ty.to_string(),
            longitude: *lon,
            magnitude: Some(*mag),
            saros_series: Some(*saros),
            notes: if notes.is_empty() { None } else { Some(notes.to_string()) },
        }
    }).collect()
}

#[derive(Debug, Clone)]
pub struct EclipseActivation {
    pub eclipse_date:  NaiveDate,
    pub eclipse_type:  String,
    pub natal_planet:  Planet,
    pub orb:           f64,
    pub strength:      f32,
    pub saros_series:  Option<i32>,
    pub days_until:    i64, // negative if past
}

/// Detect activations: natal planets within 6° of any upcoming eclipse
/// (within 12 months). Past eclipses still in 6-month "echo" window also
/// count, with reduced strength.
pub fn detect_activations(
    eclipses: &[Eclipse],
    natal_positions: &[PlanetSnapshot],
    score_date: NaiveDate,
) -> Vec<EclipseActivation> {
    let mut out = Vec::new();
    for eclipse in eclipses {
        let days_until = (eclipse.date - score_date).num_days();
        // Window: next 12 months OR past 6 months ("echo" fade)
        if days_until > 365 || days_until < -180 { continue; }

        let is_solar = eclipse.eclipse_type.starts_with("solar");
        let base = if is_solar { SOLAR_STRENGTH } else { LUNAR_STRENGTH };

        for natal in natal_positions {
            let mut diff = (eclipse.longitude - natal.longitude).abs() % 360.0;
            if diff > 180.0 { diff = 360.0 - diff; }
            if diff > ECLIPSE_ORB { continue; }

            // Tightness scaling: 0° orb full strength, 6° orb 30%
            let tightness = 1.0 - (diff / ECLIPSE_ORB) * 0.7;
            // Fade past eclipses linearly over 180-day echo
            let time_factor: f32 = if days_until < 0 {
                (1.0 + days_until as f32 / 180.0).max(0.0)
            } else if days_until > 90 {
                // Far-future eclipses contribute less until within 90 days
                0.5 + 0.5 * (1.0 - (days_until - 90) as f32 / 275.0).max(0.0)
            } else {
                1.0
            };

            out.push(EclipseActivation {
                eclipse_date:  eclipse.date,
                eclipse_type:  eclipse.eclipse_type.clone(),
                natal_planet:  natal.planet,
                orb:           diff,
                strength:      base * tightness as f32 * time_factor,
                saros_series:  eclipse.saros_series,
                days_until,
            });
        }
    }
    out
}

pub fn activation_score_total(activations: &[EclipseActivation]) -> f32 {
    activations.iter().map(|a| a.strength).sum()
}

pub fn activations_to_json(activations: &[EclipseActivation]) -> serde_json::Value {
    let arr: Vec<serde_json::Value> = activations.iter().map(|a| {
        serde_json::json!({
            "eclipse_date":  a.eclipse_date.to_string(),
            "eclipse_type":  a.eclipse_type,
            "natal_planet":  a.natal_planet.name(),
            "orb":           (a.orb * 100.0).round() / 100.0,
            "strength":      (a.strength * 10.0).round() / 10.0,
            "saros_series":  a.saros_series,
            "days_until":    a.days_until,
        })
    }).collect();
    serde_json::Value::Array(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(planet: Planet, lon: f64) -> PlanetSnapshot {
        let (sign, degree) = super::super::ephemeris::longitude_to_sign(lon);
        PlanetSnapshot { planet, longitude: lon, sign, degree, retrograde: false }
    }

    fn eclipse(date_str: &str, ty: &str, lon: f64) -> Eclipse {
        Eclipse {
            date: NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap(),
            eclipse_type: ty.to_string(),
            longitude: lon,
            magnitude: Some(1.0),
            saros_series: Some(142),
            notes: None,
        }
    }

    #[test]
    fn detects_natal_within_orb() {
        let eclipses = vec![eclipse("2026-08-12", "solar_total", 140.0)];
        let natal = vec![snap(Planet::Sun, 138.5)]; // 1.5° from eclipse
        let score_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let acts = detect_activations(&eclipses, &natal, score_date);
        assert_eq!(acts.len(), 1);
        assert!(acts[0].strength < 0.0, "Expected negative strength for solar eclipse");
    }

    #[test]
    fn ignores_distant_natal() {
        let eclipses = vec![eclipse("2026-08-12", "solar_total", 140.0)];
        let natal = vec![snap(Planet::Sun, 200.0)]; // 60° away
        let score_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let acts = detect_activations(&eclipses, &natal, score_date);
        assert!(acts.is_empty());
    }

    #[test]
    fn ignores_far_future_eclipse() {
        // Eclipse 2 years away (outside 12-month window)
        let eclipses = vec![eclipse("2028-08-12", "solar_total", 140.0)];
        let natal = vec![snap(Planet::Sun, 140.0)];
        let score_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let acts = detect_activations(&eclipses, &natal, score_date);
        assert!(acts.is_empty());
    }

    #[test]
    fn lunar_weaker_than_solar() {
        let solar = vec![eclipse("2026-08-12", "solar_total", 140.0)];
        let lunar = vec![eclipse("2026-08-12", "lunar_total", 140.0)];
        let natal = vec![snap(Planet::Sun, 140.0)];
        let score_date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let solar_act = detect_activations(&solar, &natal, score_date);
        let lunar_act = detect_activations(&lunar, &natal, score_date);
        assert!(solar_act[0].strength.abs() > lunar_act[0].strength.abs());
    }

    #[test]
    fn past_eclipse_in_echo_window() {
        // Eclipse 60 days past — still in echo window, reduced strength
        let eclipses = vec![eclipse("2026-03-03", "lunar_total", 142.0)];
        let natal = vec![snap(Planet::Sun, 141.0)];
        let score_date = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
        let acts = detect_activations(&eclipses, &natal, score_date);
        assert_eq!(acts.len(), 1);
        // Should be active but weakened by time_factor
        assert!(acts[0].days_until < 0);
    }
}
