//! Arabic Parts / Lots (Wave 6.B3).
//!
//! Calculated points derived from natal positions. Pure math, no
//! ephemeris lookup. Each Part is a "sensitive degree" that takes on
//! the meaning of its formula — Part of Fortune is the synthesis of
//! Sun (consciousness) + Moon (instinct) projected from the Ascendant
//! (incarnate self), historically used for "money timing."
//!
//! IPO charts are always day charts (09:30 EST market open), so we
//! always use the day formula for Fortune/Spirit.

use super::ephemeris::Planet;

#[derive(Debug, Clone)]
pub struct ArabicPart {
    pub name:      &'static str,
    pub longitude: f64,
    pub meaning:   &'static str,
}

/// Compute the four most financially-relevant Arabic Parts.
/// Returns empty vec if Ascendant or required bodies are missing.
pub fn compute_parts(
    ascendant: Option<f64>,
    natal_positions: &[super::ephemeris::PlanetSnapshot],
) -> Vec<ArabicPart> {
    let asc = match ascendant { Some(a) => a, None => return Vec::new() };
    let lon_of = |p: Planet| -> Option<f64> {
        natal_positions.iter().find(|x| x.planet == p).map(|x| x.longitude)
    };
    let (sun, moon, mercury) = (lon_of(Planet::Sun), lon_of(Planet::Moon), lon_of(Planet::Mercury));

    let mut parts = Vec::new();

    if let (Some(s), Some(m)) = (sun, moon) {
        parts.push(ArabicPart {
            name: "Part of Fortune",
            longitude: (asc + m - s).rem_euclid(360.0),
            meaning: "Material wealth, body, vital force",
        });
        parts.push(ArabicPart {
            name: "Part of Spirit",
            longitude: (asc + s - m).rem_euclid(360.0),
            meaning: "Soul purpose, vocation, calling",
        });
    }

    if let (Some(s), Some(m)) = (sun, mercury) {
        parts.push(ArabicPart {
            name: "Part of Commerce",
            longitude: (asc + m - s).rem_euclid(360.0),
            meaning: "Trade, negotiation, profit timing",
        });
    }

    parts.push(ArabicPart {
        name: "Part of Substance",
        longitude: (asc + 30.0).rem_euclid(360.0),
        meaning: "Resources, possessions, income flow",
    });

    parts
}

#[derive(Debug, Clone)]
pub struct PartActivation {
    pub planet:    Planet,
    pub part_name: &'static str,
    pub aspect:    &'static str,
    pub orb:       f64,
    pub strength:  f32,
}

pub fn detect_part_aspects(
    parts: &[ArabicPart],
    transits: &[super::ephemeris::PlanetSnapshot],
) -> Vec<PartActivation> {
    const ORB: f64 = 3.0;
    let aspects: &[(f64, &'static str, f32)] = &[
        (0.0,   "Conjunction",  4.0),
        (60.0,  "Sextile",      2.0),
        (90.0,  "Square",      -3.0),
        (120.0, "Trine",        3.5),
        (180.0, "Opposition",  -3.5),
    ];

    let mut out = Vec::new();
    for transit in transits {
        if transit.planet == Planet::Moon { continue; }
        for part in parts {
            let mut sep = (transit.longitude - part.longitude).abs() % 360.0;
            if sep > 180.0 { sep = 360.0 - sep; }
            for (angle, name, base_strength) in aspects {
                let diff = (sep - angle).abs();
                if diff <= ORB {
                    let tightness = 1.0 - (diff / ORB) * 0.5;
                    let part_weight: f32 = if part.name == "Part of Fortune" { 1.0 } else { 0.5 };
                    out.push(PartActivation {
                        planet: transit.planet,
                        part_name: part.name,
                        aspect: name,
                        orb: diff,
                        strength: base_strength * tightness as f32 * part_weight,
                    });
                }
            }
        }
    }
    out
}

pub fn part_activation_score_total(acts: &[PartActivation]) -> f32 {
    acts.iter().map(|a| a.strength).sum()
}

pub fn parts_to_json(parts: &[ArabicPart], activations: &[PartActivation]) -> serde_json::Value {
    let parts_arr: Vec<serde_json::Value> = parts.iter().map(|p| {
        let (sign, deg) = super::ephemeris::longitude_to_sign(p.longitude);
        serde_json::json!({
            "name":      p.name,
            "longitude": (p.longitude * 100.0).round() / 100.0,
            "sign":      sign,
            "degree":    (deg * 10.0).round() / 10.0,
            "meaning":   p.meaning,
        })
    }).collect();
    let acts_arr: Vec<serde_json::Value> = activations.iter().map(|a| {
        serde_json::json!({
            "planet":   a.planet.name(),
            "part":     a.part_name,
            "aspect":   a.aspect,
            "orb":      (a.orb * 100.0).round() / 100.0,
            "strength": (a.strength * 10.0).round() / 10.0,
        })
    }).collect();
    serde_json::json!({
        "parts": parts_arr,
        "activations": acts_arr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astrology::ephemeris::PlanetSnapshot;

    fn snap(planet: Planet, lon: f64) -> PlanetSnapshot {
        let (sign, degree) = super::super::ephemeris::longitude_to_sign(lon);
        PlanetSnapshot { planet, longitude: lon, sign, degree, retrograde: false }
    }

    #[test]
    fn fortune_formula_correct() {
        let natal = vec![snap(Planet::Sun, 10.0), snap(Planet::Moon, 100.0)];
        let parts = compute_parts(Some(0.0), &natal);
        let fortune = parts.iter().find(|p| p.name == "Part of Fortune").unwrap();
        assert!((fortune.longitude - 90.0).abs() < 0.001);
    }

    #[test]
    fn returns_empty_without_ascendant() {
        let natal = vec![snap(Planet::Sun, 10.0), snap(Planet::Moon, 100.0)];
        assert!(compute_parts(None, &natal).is_empty());
    }

    #[test]
    fn detects_transit_conjunct_fortune() {
        let natal = vec![snap(Planet::Sun, 10.0), snap(Planet::Moon, 100.0)];
        let parts = compute_parts(Some(0.0), &natal);
        let transits = vec![snap(Planet::Jupiter, 91.5)];
        let acts = detect_part_aspects(&parts, &transits);
        let conj = acts.iter().find(|a| a.part_name == "Part of Fortune"
                                     && a.aspect == "Conjunction");
        assert!(conj.is_some(), "Expected Jupiter conjunct Fortune, got {acts:?}");
    }
}
