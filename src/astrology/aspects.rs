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
    // Major aspects (classical)
    Conjunction,     //   0°
    Sextile,         //  60°
    Square,          //  90°
    Trine,           // 120°
    Opposition,      // 180°
    // Minor aspects (v2.0.2)
    SemiSextile,     //  30°  — mild opportunity, slight growth
    SemiSquare,      //  45°  — internal friction, tension
    Quincunx,        // 150°  — adjustment required, uncomfortable growth
    Sesquiquadrate,  // 135°  — agitation, external pressure
}

impl AspectType {
    pub fn angle(self) -> f64 {
        match self {
            AspectType::Conjunction    =>   0.0,
            AspectType::SemiSextile    =>  30.0,
            AspectType::SemiSquare     =>  45.0,
            AspectType::Sextile        =>  60.0,
            AspectType::Square         =>  90.0,
            AspectType::Trine          => 120.0,
            AspectType::Sesquiquadrate => 135.0,
            AspectType::Quincunx       => 150.0,
            AspectType::Opposition     => 180.0,
        }
    }

    pub fn orb(self) -> f64 {
        match self {
            // Major aspects: wider orbs (well-established in tradition)
            AspectType::Conjunction => 8.0,
            AspectType::Sextile     => 6.0,
            AspectType::Square      => 8.0,
            AspectType::Trine       => 8.0,
            AspectType::Opposition  => 8.0,
            // Minor aspects: tighter orbs (subtler energies, need precision)
            AspectType::SemiSextile    => 2.0,
            AspectType::SemiSquare     => 2.0,
            AspectType::Quincunx       => 3.0,
            AspectType::Sesquiquadrate => 2.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            AspectType::Conjunction    => "Conjunction",
            AspectType::SemiSextile    => "SemiSextile",
            AspectType::SemiSquare     => "SemiSquare",
            AspectType::Sextile        => "Sextile",
            AspectType::Square         => "Square",
            AspectType::Trine          => "Trine",
            AspectType::Sesquiquadrate => "Sesquiquadrate",
            AspectType::Quincunx       => "Quincunx",
            AspectType::Opposition     => "Opposition",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            AspectType::Conjunction    => "☌",
            AspectType::SemiSextile    => "⚺",
            AspectType::SemiSquare     => "∠",
            AspectType::Sextile        => "⚹",
            AspectType::Square         => "□",
            AspectType::Trine          => "△",
            AspectType::Sesquiquadrate => "⚼",
            AspectType::Quincunx       => "⚻",
            AspectType::Opposition     => "☍",
        }
    }

    /// Whether this is a major (classical) or minor aspect.
    pub fn is_major(self) -> bool {
        matches!(self,
            AspectType::Conjunction | AspectType::Sextile | AspectType::Square |
            AspectType::Trine | AspectType::Opposition
        )
    }

    /// All 9 aspect types in angular order for detection.
    pub fn all() -> &'static [AspectType] {
        &[
            AspectType::Conjunction,
            AspectType::SemiSextile,
            AspectType::SemiSquare,
            AspectType::Sextile,
            AspectType::Square,
            AspectType::Trine,
            AspectType::Sesquiquadrate,
            AspectType::Quincunx,
            AspectType::Opposition,
        ]
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
        Planet::Jupiter | Planet::Venus                       => PlanetNature::Benefic,
        Planet::Saturn  | Planet::Pluto | Planet::Mars        => PlanetNature::Malefic,
        // Nodes and Chiron are neutral: their effect depends on what they aspect
        Planet::NorthNode | Planet::SouthNode | Planet::Chiron => PlanetNature::Neutral,
        _                                                      => PlanetNature::Neutral,
    }
}

/// Aspect nature: harmonious (+) vs challenging (-)
pub fn aspect_nature(a: AspectType) -> i8 {
    match a {
        // Harmonious: flow, ease, opportunity
        AspectType::Trine | AspectType::Sextile | AspectType::SemiSextile =>  1,
        // Challenging: tension, pressure, forced growth
        AspectType::Square | AspectType::Opposition |
        AspectType::SemiSquare | AspectType::Sesquiquadrate              => -1,
        // Neutral: depends on participating planets
        AspectType::Conjunction                                          =>  0,
        // Quincunx: stressful adjustment (coded as mildly challenging)
        AspectType::Quincunx                                             => -1,
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
/// Checks all 9 aspect types. If multiple aspects match (overlapping orbs),
/// returns the one with the tightest orb (most exact).
pub fn find_aspect(transit_lon: f64, natal_lon: f64) -> Option<(AspectType, f64)> {
    let sep = angular_separation(transit_lon, natal_lon);
    let mut best: Option<(AspectType, f64)> = None;

    for &aspect in AspectType::all() {
        let orb = (sep - aspect.angle()).abs();
        if orb <= aspect.orb() {
            match best {
                None => best = Some((aspect, orb)),
                Some((_, prev_orb)) if orb < prev_orb => best = Some((aspect, orb)),
                _ => {}
            }
        }
    }

    best
}

// ---------------------------------------------------------------------------
// Applying vs Separating
// ---------------------------------------------------------------------------

/// Determine if a transit aspect is applying (moving toward exact) or separating
/// (moving away from exact).
///
/// This is one of the most important distinctions in astrology. An applying aspect
/// is building energy (like a wave approaching shore), while a separating aspect
/// is dissipating (wave receding). Financial astrology gives applying aspects
/// significantly more weight.
///
/// The detection uses the transit planet's `longitude_speed` (degrees/day) from
/// Swiss Ephemeris, eliminating the need to compute yesterday's positions.
///
/// Returns `true` if applying, `false` if separating. If speed is unknown, defaults
/// to applying (conservative: don't discount the aspect).
pub fn is_applying(transit_lon: f64, natal_lon: f64, transit_speed: Option<f64>, aspect_angle: f64) -> bool {
    let speed = match transit_speed {
        Some(s) => s,
        None => return true, // unknown speed, assume applying (conservative)
    };

    // Current angular separation
    let sep = angular_separation(transit_lon, natal_lon);
    // Distance from exact aspect
    let current_distance = (sep - aspect_angle).abs();

    // Predict where the transit planet will be in a small time step
    let future_lon = transit_lon + speed * 0.1; // 0.1 day forward
    let future_sep = angular_separation(future_lon, natal_lon);
    let future_distance = (future_sep - aspect_angle).abs();

    // If the distance is decreasing, the aspect is applying
    future_distance < current_distance
}

/// Score multiplier for applying vs separating aspects.
/// Applying = 1.5x (building energy), Separating = 0.7x (fading energy).
pub const APPLYING_MULTIPLIER: f64 = 1.5;
pub const SEPARATING_MULTIPLIER: f64 = 0.7;

// ---------------------------------------------------------------------------
// Planetary Dignity
// ---------------------------------------------------------------------------

/// A planet's essential dignity in a given zodiac sign.
/// This ancient system describes how "comfortable" a planet is in each sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DignityState {
    /// Planet rules this sign. Maximum strength. (+20% score)
    Domicile,
    /// Planet is exalted here. Strong and elevated. (+20% score)
    Exaltation,
    /// Planet is in the sign opposite its domicile. Weakened. (-20% score)
    Detriment,
    /// Planet is in the sign opposite its exaltation. Most challenged. (-20% score)
    Fall,
    /// Planet has no special relationship with this sign. No modifier.
    Peregrine,
}

/// Determine a planet's essential dignity in a given zodiac sign.
pub fn planetary_dignity(planet: Planet, sign: &str) -> DignityState {
    match (planet, sign) {
        // Sun
        (Planet::Sun, "Leo")         => DignityState::Domicile,
        (Planet::Sun, "Aries")       => DignityState::Exaltation,
        (Planet::Sun, "Aquarius")    => DignityState::Detriment,
        (Planet::Sun, "Libra")       => DignityState::Fall,
        // Moon
        (Planet::Moon, "Cancer")     => DignityState::Domicile,
        (Planet::Moon, "Taurus")     => DignityState::Exaltation,
        (Planet::Moon, "Capricorn")  => DignityState::Detriment,
        (Planet::Moon, "Scorpio")    => DignityState::Fall,
        // Mercury
        (Planet::Mercury, "Gemini")  => DignityState::Domicile,
        (Planet::Mercury, "Virgo")   => DignityState::Domicile,  // Mercury rules both
        (Planet::Mercury, "Sagittarius") => DignityState::Detriment,
        (Planet::Mercury, "Pisces")  => DignityState::Fall,
        // Venus
        (Planet::Venus, "Taurus")    => DignityState::Domicile,
        (Planet::Venus, "Libra")     => DignityState::Domicile,  // Venus rules both
        (Planet::Venus, "Pisces")    => DignityState::Exaltation,
        (Planet::Venus, "Aries")     => DignityState::Detriment,
        (Planet::Venus, "Scorpio")   => DignityState::Detriment,
        (Planet::Venus, "Virgo")     => DignityState::Fall,
        // Mars
        (Planet::Mars, "Aries")      => DignityState::Domicile,
        (Planet::Mars, "Scorpio")    => DignityState::Domicile,  // Mars rules both
        (Planet::Mars, "Capricorn")  => DignityState::Exaltation,
        (Planet::Mars, "Taurus")     => DignityState::Detriment,
        (Planet::Mars, "Libra")      => DignityState::Detriment,
        (Planet::Mars, "Cancer")     => DignityState::Fall,
        // Jupiter
        (Planet::Jupiter, "Sagittarius") => DignityState::Domicile,
        (Planet::Jupiter, "Cancer")      => DignityState::Exaltation,
        (Planet::Jupiter, "Gemini")      => DignityState::Detriment,
        (Planet::Jupiter, "Capricorn")   => DignityState::Fall,
        // Saturn
        (Planet::Saturn, "Capricorn")    => DignityState::Domicile,
        (Planet::Saturn, "Aquarius")     => DignityState::Domicile,  // traditional co-ruler
        (Planet::Saturn, "Libra")        => DignityState::Exaltation,
        (Planet::Saturn, "Cancer")       => DignityState::Detriment,
        (Planet::Saturn, "Aries")        => DignityState::Fall,
        // Outer planets and nodes: no traditional dignity (Peregrine everywhere)
        _ => DignityState::Peregrine,
    }
}

/// Score modifier for dignity state: +20% for dignified, -20% for debilitated.
pub fn dignity_modifier(state: DignityState) -> f64 {
    match state {
        DignityState::Domicile | DignityState::Exaltation => 1.20,
        DignityState::Detriment | DignityState::Fall      => 0.80,
        DignityState::Peregrine                            => 1.00,
    }
}

impl DignityState {
    pub fn name(self) -> &'static str {
        match self {
            DignityState::Domicile   => "Domicile",
            DignityState::Exaltation => "Exalted",
            DignityState::Detriment  => "Detriment",
            DignityState::Fall       => "Fall",
            DignityState::Peregrine  => "Peregrine",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            DignityState::Domicile   => "+",
            DignityState::Exaltation => "+",
            DignityState::Detriment  => "-",
            DignityState::Fall       => "-",
            DignityState::Peregrine  => "",
        }
    }
}

// ---------------------------------------------------------------------------
// Scoring
// ---------------------------------------------------------------------------

/// Compute the score delta for a single aspect.
///
/// The scoring pipeline applies three layers of modifiers:
/// 1. **Base magnitude** from planet natures (benefic/malefic/neutral interactions)
/// 2. **Orb modifier** — exact aspects score full, wide orbs score 25%
/// 3. **Applying/separating** — applying 1.5x, separating 0.7x
/// 4. **Dignity modifier** — dignified transit planet +20%, debilitated -20%
///
/// Minor aspects use a 0.5x base reduction (subtler energy than major aspects).
pub fn score_aspect(transit: Planet, natal: Planet, aspect: AspectType, orb: f64) -> f32 {
    score_aspect_full(transit, natal, aspect, orb, None, None)
}

/// Full scoring with applying/separating and dignity support.
pub fn score_aspect_full(
    transit: Planet,
    natal: Planet,
    aspect: AspectType,
    orb: f64,
    transit_speed: Option<f64>,
    transit_sign: Option<&str>,
) -> f32 {
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

    // Minor aspect reduction: minor aspects carry ~50% the weight of major ones
    let minor_mod = if aspect.is_major() { 1.0 } else { 0.5 };

    // Applying/separating multiplier
    let apply_mod = match transit_speed {
        Some(_) => {
            // We can't call is_applying here without transit_lon and natal_lon,
            // so this is handled by the caller who passes the final multiplier.
            // For score_aspect_full, the caller should pre-compute and pass via
            // transit_speed = Some(APPLYING_MULTIPLIER or SEPARATING_MULTIPLIER).
            // When transit_speed is the raw speed, we default to 1.0 here.
            // The actual applying/separating is applied in compute_transit_score().
            1.0
        }
        None => 1.0,
    };

    // Dignity modifier for the transit planet
    let dig_mod = match transit_sign {
        Some(sign) => dignity_modifier(planetary_dignity(transit, sign)),
        None => 1.0,
    };

    (base * direction * orb_mod * minor_mod * apply_mod * dig_mod) as f32
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
    /// Whether the transit is applying (moving toward exact) or separating.
    pub applying:       bool,
    /// Transit planet's essential dignity in its current sign.
    pub dignity:        DignityState,
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

    // --- v2.0.2 new tests ---

    #[test]
    fn test_find_minor_aspect_quincunx() {
        // 0° and 150° = quincunx (orb 3°)
        let result = find_aspect(0.0, 150.0);
        assert!(result.is_some());
        let (asp, orb) = result.unwrap();
        assert_eq!(asp, AspectType::Quincunx);
        assert!(orb < 0.01, "Expected near-exact quincunx, got orb={orb}");
    }

    #[test]
    fn test_find_minor_aspect_semisextile() {
        // 100° and 130° = semi-sextile (30°, orb 2°)
        let result = find_aspect(100.0, 130.0);
        assert!(result.is_some());
        let (asp, _) = result.unwrap();
        assert_eq!(asp, AspectType::SemiSextile);
    }

    #[test]
    fn test_minor_aspect_scores_less_than_major() {
        // Same planets, same orb — minor aspect should score ~50% of major
        let major = score_aspect(Planet::Saturn, Planet::Sun, AspectType::Square, 1.0);
        let minor = score_aspect(Planet::Saturn, Planet::Sun, AspectType::SemiSquare, 1.0);
        assert!(
            minor.abs() < major.abs(),
            "Minor aspect ({minor:.2}) should be weaker than major ({major:.2})",
        );
    }

    #[test]
    fn test_applying_detection() {
        // Transit at 118° moving toward natal at 120° (trine with 0°) at +1°/day
        // The transit is approaching 120° (exact trine) so it's applying
        let applying = is_applying(118.0, 0.0, Some(1.0), 120.0);
        assert!(applying, "Transit approaching exact aspect should be applying");

        // Transit at 122° moving away from 120° at +1°/day = separating
        let separating = is_applying(122.0, 0.0, Some(1.0), 120.0);
        assert!(!separating, "Transit moving away from exact should be separating");
    }

    #[test]
    fn test_dignity_venus_pisces_exalted() {
        let d = planetary_dignity(Planet::Venus, "Pisces");
        assert_eq!(d, DignityState::Exaltation);
        assert!((dignity_modifier(d) - 1.2).abs() < 0.01);
    }

    #[test]
    fn test_dignity_venus_virgo_fall() {
        let d = planetary_dignity(Planet::Venus, "Virgo");
        assert_eq!(d, DignityState::Fall);
        assert!((dignity_modifier(d) - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_dignity_affects_score() {
        // Venus in Pisces (exalted, +20%) vs Venus in Aries (detriment, -20%)
        let exalted = score_aspect_full(
            Planet::Venus, Planet::Jupiter, AspectType::Trine, 0.0, None, Some("Pisces"),
        );
        let debilitated = score_aspect_full(
            Planet::Venus, Planet::Jupiter, AspectType::Trine, 0.0, None, Some("Aries"),
        );
        assert!(
            exalted.abs() > debilitated.abs(),
            "Exalted Venus ({exalted:.2}) should score higher than debilitated ({debilitated:.2})",
        );
    }

    #[test]
    fn test_all_9_aspects_detectable() {
        // Each aspect should be detectable at its exact angle
        for &aspect in AspectType::all() {
            let natal_lon = 0.0;
            let transit_lon = aspect.angle();
            let result = find_aspect(transit_lon, natal_lon);
            assert!(
                result.is_some(),
                "{:?} at exact angle {:.0}° should be detected",
                aspect, aspect.angle(),
            );
            let (found, _) = result.unwrap();
            assert_eq!(found, aspect, "Expected {:?}, found {:?}", aspect, found);
        }
    }
}
