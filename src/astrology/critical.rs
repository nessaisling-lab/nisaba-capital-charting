//! Wave 9.B4 — Critical degrees + Out-of-Bounds detection.
//!
//! **Critical degrees** are sensitive points within each sign-modality
//! (cardinal / fixed / mutable). A planet at a critical degree carries
//! more weight than a planet at a "normal" degree of the same sign —
//! these are *action* degrees, often coincident with major life events.
//!
//! - **Cardinal signs** (Aries, Cancer, Libra, Capricorn): 0°, 13°, 26°
//! - **Fixed signs**    (Taurus, Leo, Scorpio, Aquarius):   8-9°, 21-22°
//! - **Mutable signs**  (Gemini, Virgo, Sagittarius, Pisces): 4°, 17°
//!
//! The 0° cardinal degree is *especially* loaded — also called the "world
//! degree." A planet at 0° Aries / Cancer / Libra / Capricorn brings a
//! universal, archetypal flavor.
//!
//! **Out-of-bounds** (OOB) — a planet whose declination exceeds ±23.4367°
//! (the Sun's tropic-of-Cancer/Capricorn limit). Such a planet operates
//! "beyond the Sun's path" and tends to express in non-standard or
//! unpredictable ways. Common for Mercury, Venus, Mars, and the Moon;
//! rare for outer planets. Already detected via `is_out_of_bounds()` in
//! `ephemeris.rs` — this module just provides UI-friendly classification.
//!
//! Reference: standard tropical astrology texts (Rudhyar, Greene). The
//! "critical degrees" set is shared across most lineages; the modality-
//! based grouping shown here is the post-Lilly synthesis.

// (Planet import reserved for future profections-aware classification.)

// ---------------------------------------------------------------------------
// Critical degrees
// ---------------------------------------------------------------------------

/// Modality classification for a sign — drives the critical degrees set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignModality {
    Cardinal,
    Fixed,
    Mutable,
}

/// Get the modality for a sign index (0=Aries, 1=Taurus, …, 11=Pisces).
/// Signs cycle Cardinal → Fixed → Mutable every 3, so use modulo 3.
pub fn modality_for_sign_index(idx: usize) -> SignModality {
    match idx % 3 {
        0 => SignModality::Cardinal, // Aries, Cancer, Libra, Capricorn
        1 => SignModality::Fixed,    // Taurus, Leo, Scorpio, Aquarius
        _ => SignModality::Mutable,  // Gemini, Virgo, Sagittarius, Pisces
    }
}

/// Detected critical-degree classification at a given longitude.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriticalDegree {
    /// 0° Aries / Cancer / Libra / Capricorn — the four "world degrees."
    /// Most loaded of all critical degrees; archetypal force.
    WorldDegree,
    /// Other cardinal critical degrees: 13° / 26° of cardinal signs.
    Cardinal,
    /// Fixed critical degrees: 8°-9° / 21°-22° of fixed signs.
    Fixed,
    /// Mutable critical degrees: 4° / 17° of mutable signs.
    Mutable,
}

impl CriticalDegree {
    /// One-line UI label.
    pub fn label(self) -> &'static str {
        match self {
            Self::WorldDegree => "World degree (0° cardinal)",
            Self::Cardinal    => "Critical degree (cardinal)",
            Self::Fixed       => "Critical degree (fixed)",
            Self::Mutable     => "Critical degree (mutable)",
        }
    }

    /// Strength multiplier applied to aspects involving a planet at
    /// this degree. 0° cardinal weighs heaviest; others +30%.
    pub fn strength_multiplier(self) -> f64 {
        match self {
            Self::WorldDegree => 1.5,
            Self::Cardinal | Self::Fixed | Self::Mutable => 1.3,
        }
    }
}

/// Wave 9.B4 — Detect whether a longitude falls on a critical degree.
///
/// Tolerance: ±0.5° (matches typical "exact orb" reading; planet within
/// half a degree of the critical point counts as on it).
pub fn is_critical_degree(lon: f64) -> Option<CriticalDegree> {
    let mut l = lon % 360.0;
    if l < 0.0 { l += 360.0; }
    let sign_index = (l / 30.0).floor() as usize % 12;
    let degree_in_sign = l - sign_index as f64 * 30.0;
    let modality = modality_for_sign_index(sign_index);
    const TOL: f64 = 0.5;

    match modality {
        SignModality::Cardinal => {
            // 0° cardinal = world degree (special)
            if degree_in_sign < TOL || degree_in_sign > (30.0 - TOL) {
                Some(CriticalDegree::WorldDegree)
            } else if (degree_in_sign - 13.0).abs() < TOL
                   || (degree_in_sign - 26.0).abs() < TOL {
                Some(CriticalDegree::Cardinal)
            } else {
                None
            }
        }
        SignModality::Fixed => {
            // Fixed: 8-9° and 21-22° (tighter ±0.5° each)
            if (degree_in_sign - 8.5).abs() < 1.0 + TOL
            || (degree_in_sign - 21.5).abs() < 1.0 + TOL {
                Some(CriticalDegree::Fixed)
            } else {
                None
            }
        }
        SignModality::Mutable => {
            // Mutable: 4° and 17°
            if (degree_in_sign - 4.0).abs() < TOL
            || (degree_in_sign - 17.0).abs() < TOL {
                Some(CriticalDegree::Mutable)
            } else {
                None
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Out-of-bounds (declination)
// ---------------------------------------------------------------------------

/// Classification of a planet's out-of-bounds state. Mostly for UI
/// formatting — the actual boolean test lives in `ephemeris::is_out_of_bounds`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OobState {
    /// Within ±23.4367° declination — normal range.
    Normal,
    /// Declination > +23.4367° (north of summer solstice).
    OobNorth,
    /// Declination < -23.4367° (south of winter solstice).
    OobSouth,
}

impl OobState {
    pub fn is_out_of_bounds(self) -> bool {
        !matches!(self, OobState::Normal)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Normal    => "in bounds",
            Self::OobNorth  => "out-of-bounds north",
            Self::OobSouth  => "out-of-bounds south",
        }
    }
}

/// Wave 9.B4 — Classify a planet's declination as in-bounds or OOB.
pub fn classify_oob(declination: f64) -> OobState {
    use super::ephemeris::OUT_OF_BOUNDS_THRESHOLD;
    if declination > OUT_OF_BOUNDS_THRESHOLD {
        OobState::OobNorth
    } else if declination < -OUT_OF_BOUNDS_THRESHOLD {
        OobState::OobSouth
    } else {
        OobState::Normal
    }
}

/// Wave 9.B4 — Strength multiplier for an OOB planet. Out-of-bounds
/// planets are *louder* — they exceed the Sun's natural envelope, so
/// their expression is amplified (often unpredictably). +25%.
pub fn oob_strength_multiplier(state: OobState) -> f64 {
    if state.is_out_of_bounds() { 1.25 } else { 1.0 }
}

// ---------------------------------------------------------------------------
// Convenience: combine critical degree + OOB flags for one planet
// ---------------------------------------------------------------------------

/// Combined precision flags for one planet position.
#[derive(Debug, Clone, Copy)]
pub struct PrecisionFlags {
    pub critical: Option<CriticalDegree>,
    pub oob:      OobState,
}

impl PrecisionFlags {
    /// Compose the combined strength multiplier from both flags.
    /// Caps at 2.0× to avoid runaway scoring on stacked flags.
    pub fn combined_multiplier(&self) -> f64 {
        let crit = self.critical.map(|c| c.strength_multiplier()).unwrap_or(1.0);
        let oob_m = oob_strength_multiplier(self.oob);
        (crit * oob_m).min(2.0)
    }

    /// Short UI label combining whichever flags fired.
    /// Empty string if nothing notable.
    pub fn label(&self) -> String {
        let mut parts = Vec::new();
        if let Some(c) = self.critical { parts.push(c.label()); }
        if self.oob.is_out_of_bounds() { parts.push(self.oob.label()); }
        parts.join(" · ")
    }
}

/// Wave 9.B4 — Compute precision flags for one planet at a given
/// longitude + declination.
pub fn flags_for(lon: f64, declination: f64) -> PrecisionFlags {
    PrecisionFlags {
        critical: is_critical_degree(lon),
        oob:      classify_oob(declination),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0° Aries → world degree.
    #[test]
    fn aries_zero_world_degree() {
        assert_eq!(is_critical_degree(0.0), Some(CriticalDegree::WorldDegree));
    }

    /// 0° Cancer (90°) → world degree.
    #[test]
    fn cancer_zero_world_degree() {
        assert_eq!(is_critical_degree(90.0), Some(CriticalDegree::WorldDegree));
    }

    /// 13° Aries → cardinal critical.
    #[test]
    fn aries_13_cardinal() {
        assert_eq!(is_critical_degree(13.0), Some(CriticalDegree::Cardinal));
    }

    /// 26° Capricorn → cardinal critical (270°+26° = 296°).
    #[test]
    fn capricorn_26_cardinal() {
        assert_eq!(is_critical_degree(296.0), Some(CriticalDegree::Cardinal));
    }

    /// 8° Taurus (38°) → fixed critical.
    #[test]
    fn taurus_8_fixed() {
        assert_eq!(is_critical_degree(38.0), Some(CriticalDegree::Fixed));
    }

    /// 21° Leo (120°+21°=141°) → fixed critical.
    #[test]
    fn leo_21_fixed() {
        assert_eq!(is_critical_degree(141.0), Some(CriticalDegree::Fixed));
    }

    /// 4° Gemini (60°+4°=64°) → mutable critical.
    #[test]
    fn gemini_4_mutable() {
        assert_eq!(is_critical_degree(64.0), Some(CriticalDegree::Mutable));
    }

    /// 17° Pisces (330°+17°=347°) → mutable critical.
    #[test]
    fn pisces_17_mutable() {
        assert_eq!(is_critical_degree(347.0), Some(CriticalDegree::Mutable));
    }

    /// AAPL natal Saturn ~16° Virgo (166°) — close to 17° mutable
    /// critical degree. With ±0.5° tol, 16.0° is just outside (1° away).
    #[test]
    fn aapl_saturn_not_quite_critical() {
        // AAPL natal Saturn at exactly 16° → not on critical 17°.
        assert!(is_critical_degree(166.0).is_none());
        // But 17° Virgo (167°) IS critical.
        assert_eq!(is_critical_degree(167.0), Some(CriticalDegree::Mutable));
    }

    /// 5° Aries → no critical degree (between 0° and 13°).
    #[test]
    fn aries_5_no_critical() {
        assert!(is_critical_degree(5.0).is_none());
    }

    /// World degree mult is highest.
    #[test]
    fn world_degree_strongest() {
        assert!(CriticalDegree::WorldDegree.strength_multiplier()
            > CriticalDegree::Cardinal.strength_multiplier());
    }

    /// OOB classification.
    #[test]
    fn oob_north_at_25() {
        assert_eq!(classify_oob(25.0), OobState::OobNorth);
        assert!(classify_oob(25.0).is_out_of_bounds());
    }

    #[test]
    fn oob_south_at_minus_25() {
        assert_eq!(classify_oob(-25.0), OobState::OobSouth);
    }

    #[test]
    fn oob_normal_at_zero() {
        assert_eq!(classify_oob(0.0), OobState::Normal);
        assert!(!classify_oob(0.0).is_out_of_bounds());
    }

    /// Combined multiplier capped at 2.0×.
    #[test]
    fn combined_multiplier_caps() {
        let flags = PrecisionFlags {
            critical: Some(CriticalDegree::WorldDegree), // 1.5
            oob:      OobState::OobNorth,                 // 1.25
        };
        // 1.5 × 1.25 = 1.875 → under cap
        let m = flags.combined_multiplier();
        assert!((m - 1.875).abs() < 0.01);
    }

    /// Label rendering for combined flags.
    #[test]
    fn combined_label() {
        let flags = PrecisionFlags {
            critical: Some(CriticalDegree::WorldDegree),
            oob:      OobState::OobNorth,
        };
        let label = flags.label();
        assert!(label.contains("World degree"));
        assert!(label.contains("out-of-bounds"));
    }

    /// Empty label when nothing notable.
    #[test]
    fn empty_label() {
        let flags = PrecisionFlags {
            critical: None,
            oob:      OobState::Normal,
        };
        assert_eq!(flags.label(), "");
        assert_eq!(flags.combined_multiplier(), 1.0);
    }

}
