//! Aspect detection and scoring.
//!
//! An aspect is a significant angular relationship between two planets.
//! We check transit planets against natal planets and score the result.

use super::ephemeris::Planet;

// ---------------------------------------------------------------------------
// Aspect types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectType {
    Conjunction,  //   0°
    Sextile,      //  60°
    Square,       //  90°
    Trine,        // 120°
    Opposition,   // 180°
}

impl AspectType {
    pub fn angle(self) -> f64 {
        match self {
            AspectType::Conjunction => 0.0,
            AspectType::Sextile     => 60.0,
            AspectType::Square      => 90.0,
            AspectType::Trine       => 120.0,
            AspectType::Opposition  => 180.0,
        }
    }

    pub fn orb(self) -> f64 {
        match self {
            AspectType::Conjunction => 8.0,
            AspectType::Sextile     => 6.0,
            AspectType::Square      => 8.0,
            AspectType::Trine       => 8.0,
            AspectType::Opposition  => 8.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            AspectType::Conjunction => "Conjunction",
            AspectType::Sextile     => "Sextile",
            AspectType::Square      => "Square",
            AspectType::Trine       => "Trine",
            AspectType::Opposition  => "Opposition",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            AspectType::Conjunction => "☌",
            AspectType::Sextile     => "⚹",
            AspectType::Square      => "□",
            AspectType::Trine       => "△",
            AspectType::Opposition  => "☍",
        }
    }
}

// ---------------------------------------------------------------------------
// Planet nature
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanetNature {
    Benefic,   // Jupiter, Venus — naturally positive
    Malefic,   // Saturn, Pluto, Mars — naturally challenging
    Neutral,   // everything else — context-dependent
}

pub fn planet_nature(p: Planet) -> PlanetNature {
    match p {
        Planet::Jupiter | Planet::Venus                    => PlanetNature::Benefic,
        Planet::Saturn  | Planet::Pluto | Planet::Mars     => PlanetNature::Malefic,
        _                                                   => PlanetNature::Neutral,
    }
}

/// Aspect nature: harmonious (+) vs challenging (-)
pub fn aspect_nature(a: AspectType) -> i8 {
    match a {
        AspectType::Trine | AspectType::Sextile  =>  1,  // harmonious
        AspectType::Square | AspectType::Opposition => -1, // challenging
        AspectType::Conjunction                    =>  0,  // depends on planets
    }
}

// ---------------------------------------------------------------------------
// Aspect detection
// ---------------------------------------------------------------------------

/// Angular separation between two longitudes, normalized to [0, 180].
fn angular_separation(lon_a: f64, lon_b: f64) -> f64 {
    let mut diff = (lon_a - lon_b).abs() % 360.0;
    if diff > 180.0 { diff = 360.0 - diff; }
    diff
}

/// Find the aspect type (if any) between two ecliptic longitudes.
pub fn find_aspect(transit_lon: f64, natal_lon: f64) -> Option<(AspectType, f64)> {
    let sep = angular_separation(transit_lon, natal_lon);
    for &aspect in &[
        AspectType::Conjunction,
        AspectType::Sextile,
        AspectType::Square,
        AspectType::Trine,
        AspectType::Opposition,
    ] {
        let orb = (sep - aspect.angle()).abs();
        if orb <= aspect.orb() {
            return Some((aspect, orb));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Scoring
// ---------------------------------------------------------------------------

/// Compute the score delta [-20, +20] for a single aspect.
///
/// Logic:
/// - Benefic planet + harmonious aspect → strong positive
/// - Benefic planet + challenging aspect → mild negative (not as bad as malefic)
/// - Malefic planet + challenging aspect → strong negative
/// - Malefic planet + harmonious aspect → mild positive (mitigated)
/// - Neutral planet: small modifiers
/// - Orb modifier: linear from 1.0 (exact) to 0.25 (at max orb)
pub fn score_aspect(transit: Planet, natal: Planet, aspect: AspectType, orb: f64) -> f32 {
    let transit_nature = planet_nature(transit);
    let natal_nature   = planet_nature(natal);
    let aspect_dir     = aspect_nature(aspect);

    // Base magnitude before direction
    let base: f64 = match (transit_nature, natal_nature) {
        (PlanetNature::Benefic, PlanetNature::Benefic) => 15.0,
        (PlanetNature::Benefic, PlanetNature::Neutral)
        | (PlanetNature::Neutral, PlanetNature::Benefic) => 10.0,
        (PlanetNature::Malefic, PlanetNature::Malefic) => 14.0,
        (PlanetNature::Malefic, PlanetNature::Neutral)
        | (PlanetNature::Neutral, PlanetNature::Malefic) => 9.0,
        (PlanetNature::Benefic, PlanetNature::Malefic)
        | (PlanetNature::Malefic, PlanetNature::Benefic) => 7.0,
        (PlanetNature::Neutral, PlanetNature::Neutral) => 5.0,
    };

    // Direction: conjunction depends on planet natures
    let direction: f64 = match aspect {
        AspectType::Conjunction => {
            match (transit_nature, natal_nature) {
                (PlanetNature::Benefic, _) | (_, PlanetNature::Benefic) =>  1.0,
                (PlanetNature::Malefic, _) | (_, PlanetNature::Malefic) => -1.0,
                _ => 0.5,
            }
        }
        _ => aspect_dir as f64,
    };

    // Orb modifier: exact = 1.0, max orb = 0.25 (linear)
    let max_orb = aspect.orb();
    let orb_mod = 1.0 - 0.75 * (orb / max_orb);

    (base * direction * orb_mod) as f32
}

// ---------------------------------------------------------------------------
// Active aspect — what we store and display
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ActiveAspect {
    pub transit_planet: Planet,
    pub transit_sign:   String,
    pub natal_planet:   Planet,
    pub natal_sign:     String,
    pub aspect:         AspectType,
    pub orb:            f64,
    pub score_delta:    f32,
}

impl ActiveAspect {
    pub fn effect_label(&self) -> &'static str {
        if self.score_delta > 4.0      { "Favorable" }
        else if self.score_delta < -4.0 { "Challenging" }
        else                            { "Minor" }
    }
}

// ---------------------------------------------------------------------------
// Moon phase modifier
// ---------------------------------------------------------------------------

/// Score modifier from current moon phase. Range: -8 to +8.
pub fn moon_phase_modifier(phase_angle: f64) -> f32 {
    match phase_angle as u32 {
        0..=29    =>  5.0,  // New Moon — initiation, fresh start
        30..=149  =>  8.0,  // Waxing — building, growth energy
        150..=209 => -5.0,  // Full Moon — peak, reversal risk
        210..=329 => -8.0,  // Waning — declining, releasing
        330..=359 => -3.0,  // Balsamic — endings, clearing
        _         =>  0.0,
    }
}

// ---------------------------------------------------------------------------
// Mercury retrograde cap
// ---------------------------------------------------------------------------

/// When Mercury is retrograde, cap score at 65 to reflect uncertainty/disruption.
pub const MERCURY_RX_CAP: f32 = 65.0;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angular_separation() {
        // 0° and 120° → 120° apart
        assert!((angular_separation(0.0, 120.0) - 120.0).abs() < 0.001);
        // 10° and 350° → 20° apart (across 0°)
        assert!((angular_separation(10.0, 350.0) - 20.0).abs() < 0.001);
        // 0° and 181° → 179° (not 181°)
        assert!((angular_separation(0.0, 181.0) - 179.0).abs() < 0.001);
    }

    #[test]
    fn test_find_aspect_trine() {
        // 0° and 120° = exact trine
        let result = find_aspect(0.0, 120.0);
        assert!(result.is_some());
        let (asp, orb) = result.unwrap();
        assert_eq!(asp, AspectType::Trine);
        assert!(orb < 0.01);
    }

    #[test]
    fn test_find_aspect_out_of_orb() {
        // 0° and 50° = not a recognized aspect (nearest is sextile at 60°, orb=10° > 6°)
        let result = find_aspect(0.0, 50.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_score_positive() {
        // Jupiter trine natal Sun = strongly positive
        let delta = score_aspect(Planet::Jupiter, Planet::Sun, AspectType::Trine, 0.0);
        assert!(delta > 0.0, "Jupiter trine should be positive: {delta}");
    }

    #[test]
    fn test_score_negative() {
        // Saturn square natal Sun = strongly negative
        let delta = score_aspect(Planet::Saturn, Planet::Sun, AspectType::Square, 0.0);
        assert!(delta < 0.0, "Saturn square should be negative: {delta}");
    }

    #[test]
    fn test_orb_diminishes_score() {
        let exact   = score_aspect(Planet::Jupiter, Planet::Sun, AspectType::Trine, 0.0);
        let wide    = score_aspect(Planet::Jupiter, Planet::Sun, AspectType::Trine, 7.9);
        assert!(exact.abs() > wide.abs(), "Wider orb should have smaller magnitude");
    }
}
