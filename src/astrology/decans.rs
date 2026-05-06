//! Wave 9.B1 — Decans (10° subdivisions of zodiac signs).
//!
//! Each 30° sign divides into three 10° decans. Egyptian system (most
//! commonly used in Hellenistic astrology) assigns each decan a primary
//! ruler (the sign's own ruler) and a *sub-ruler* drawn from a Chaldean
//! planet sequence (Saturn → Jupiter → Mars → Sun → Venus → Mercury → Moon
//! → repeat). The sub-ruler tints the decan's character.
//!
//! Use case: a planet at 5° Aries vs 25° Aries reads differently. Aries
//! decan 1 (Mars-Mars) is raw initiating force; decan 2 (Mars-Sun) adds
//! a leadership shine; decan 3 (Mars-Venus) softens with magnetism.
//!
//! Reference: standard Egyptian/Chaldean decan tables. See e.g. *Hellenistic
//! Astrology* by Chris Brennan §19, or Project Hindsight translations of
//! Vettius Valens.

use super::ephemeris::Planet;

/// One decan of a sign (10° wide).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decan {
    /// Sign name (Aries..Pisces).
    pub sign: &'static str,
    /// 0, 1, or 2 — which decan within the sign.
    pub decan_index: u8,
    /// Sign's own ruler — primary tone.
    pub ruler: Planet,
    /// Chaldean-sequence sub-ruler — secondary flavor.
    pub sub_ruler: Planet,
    /// Brief thematic descriptor (~3-6 words).
    pub theme: &'static str,
}

/// Wave 9.B1 — Look up the decan for any ecliptic longitude.
///
/// Returns the matching decan record. Longitude is normalized to [0, 360)
/// before lookup. Each decan is exactly 10° wide.
pub fn decan_for_longitude(lon: f64) -> Decan {
    let mut l = lon % 360.0;
    if l < 0.0 { l += 360.0; }
    let sign_index = (l / 30.0).floor() as usize % 12;
    let degree_in_sign = l - sign_index as f64 * 30.0;
    let decan_index = ((degree_in_sign / 10.0).floor() as u8).min(2);
    DECAN_TABLE[sign_index][decan_index as usize]
}

/// 12 signs × 3 decans = 36 entries. Order: Aries, Taurus, ..., Pisces.
/// Sub-rulers follow the Chaldean order from each sign's ruler.
const DECAN_TABLE: [[Decan; 3]; 12] = [
    // Aries (Mars)
    [
        Decan { sign: "Aries",   decan_index: 0, ruler: Planet::Mars,    sub_ruler: Planet::Mars,    theme: "raw initiating force" },
        Decan { sign: "Aries",   decan_index: 1, ruler: Planet::Mars,    sub_ruler: Planet::Sun,     theme: "leadership shine" },
        Decan { sign: "Aries",   decan_index: 2, ruler: Planet::Mars,    sub_ruler: Planet::Venus,   theme: "magnetic action" },
    ],
    // Taurus (Venus)
    [
        Decan { sign: "Taurus",  decan_index: 0, ruler: Planet::Venus,   sub_ruler: Planet::Venus,   theme: "rooted sensual ground" },
        Decan { sign: "Taurus",  decan_index: 1, ruler: Planet::Venus,   sub_ruler: Planet::Mercury, theme: "practical exchange" },
        Decan { sign: "Taurus",  decan_index: 2, ruler: Planet::Venus,   sub_ruler: Planet::Saturn,  theme: "patient accumulation" },
    ],
    // Gemini (Mercury)
    [
        Decan { sign: "Gemini",  decan_index: 0, ruler: Planet::Mercury, sub_ruler: Planet::Jupiter, theme: "broad curiosity" },
        Decan { sign: "Gemini",  decan_index: 1, ruler: Planet::Mercury, sub_ruler: Planet::Mars,    theme: "sharp wit" },
        Decan { sign: "Gemini",  decan_index: 2, ruler: Planet::Mercury, sub_ruler: Planet::Sun,     theme: "articulate confidence" },
    ],
    // Cancer (Moon)
    [
        Decan { sign: "Cancer",  decan_index: 0, ruler: Planet::Moon,    sub_ruler: Planet::Venus,   theme: "tender feeling-tone" },
        Decan { sign: "Cancer",  decan_index: 1, ruler: Planet::Moon,    sub_ruler: Planet::Mercury, theme: "intuitive communication" },
        Decan { sign: "Cancer",  decan_index: 2, ruler: Planet::Moon,    sub_ruler: Planet::Moon,    theme: "deep emotional core" },
    ],
    // Leo (Sun)
    [
        Decan { sign: "Leo",     decan_index: 0, ruler: Planet::Sun,     sub_ruler: Planet::Saturn,  theme: "disciplined sovereignty" },
        Decan { sign: "Leo",     decan_index: 1, ruler: Planet::Sun,     sub_ruler: Planet::Jupiter, theme: "expansive generosity" },
        Decan { sign: "Leo",     decan_index: 2, ruler: Planet::Sun,     sub_ruler: Planet::Mars,    theme: "courageous performance" },
    ],
    // Virgo (Mercury)
    [
        Decan { sign: "Virgo",   decan_index: 0, ruler: Planet::Mercury, sub_ruler: Planet::Sun,     theme: "skilled craft" },
        Decan { sign: "Virgo",   decan_index: 1, ruler: Planet::Mercury, sub_ruler: Planet::Venus,   theme: "refined service" },
        Decan { sign: "Virgo",   decan_index: 2, ruler: Planet::Mercury, sub_ruler: Planet::Mercury, theme: "analytical precision" },
    ],
    // Libra (Venus)
    [
        Decan { sign: "Libra",   decan_index: 0, ruler: Planet::Venus,   sub_ruler: Planet::Moon,    theme: "diplomatic warmth" },
        Decan { sign: "Libra",   decan_index: 1, ruler: Planet::Venus,   sub_ruler: Planet::Saturn,  theme: "structured fairness" },
        Decan { sign: "Libra",   decan_index: 2, ruler: Planet::Venus,   sub_ruler: Planet::Jupiter, theme: "expansive grace" },
    ],
    // Scorpio (Mars / Pluto)
    [
        Decan { sign: "Scorpio", decan_index: 0, ruler: Planet::Mars,    sub_ruler: Planet::Mars,    theme: "intense focus" },
        Decan { sign: "Scorpio", decan_index: 1, ruler: Planet::Mars,    sub_ruler: Planet::Sun,     theme: "regenerative power" },
        Decan { sign: "Scorpio", decan_index: 2, ruler: Planet::Mars,    sub_ruler: Planet::Venus,   theme: "magnetic depth" },
    ],
    // Sagittarius (Jupiter)
    [
        Decan { sign: "Sagittarius", decan_index: 0, ruler: Planet::Jupiter, sub_ruler: Planet::Mercury, theme: "philosophical curiosity" },
        Decan { sign: "Sagittarius", decan_index: 1, ruler: Planet::Jupiter, sub_ruler: Planet::Moon,    theme: "intuitive vision" },
        Decan { sign: "Sagittarius", decan_index: 2, ruler: Planet::Jupiter, sub_ruler: Planet::Saturn,  theme: "principled wisdom" },
    ],
    // Capricorn (Saturn)
    [
        Decan { sign: "Capricorn", decan_index: 0, ruler: Planet::Saturn, sub_ruler: Planet::Jupiter, theme: "ambitious build" },
        Decan { sign: "Capricorn", decan_index: 1, ruler: Planet::Saturn, sub_ruler: Planet::Mars,    theme: "executive drive" },
        Decan { sign: "Capricorn", decan_index: 2, ruler: Planet::Saturn, sub_ruler: Planet::Sun,     theme: "mature authority" },
    ],
    // Aquarius (Saturn / Uranus)
    [
        Decan { sign: "Aquarius", decan_index: 0, ruler: Planet::Saturn, sub_ruler: Planet::Venus,   theme: "innovative collective" },
        Decan { sign: "Aquarius", decan_index: 1, ruler: Planet::Saturn, sub_ruler: Planet::Mercury, theme: "intellectual reform" },
        Decan { sign: "Aquarius", decan_index: 2, ruler: Planet::Saturn, sub_ruler: Planet::Moon,    theme: "humanitarian feeling" },
    ],
    // Pisces (Jupiter / Neptune)
    [
        Decan { sign: "Pisces",  decan_index: 0, ruler: Planet::Jupiter, sub_ruler: Planet::Saturn,  theme: "structured imagination" },
        Decan { sign: "Pisces",  decan_index: 1, ruler: Planet::Jupiter, sub_ruler: Planet::Jupiter, theme: "expansive empathy" },
        Decan { sign: "Pisces",  decan_index: 2, ruler: Planet::Jupiter, sub_ruler: Planet::Mars,    theme: "passionate compassion" },
    ],
];

/// Convenience: returns the decan as a one-line label for UI rendering.
/// Format: "Aries decan 1 (Mars-Sun) — leadership shine"
pub fn decan_label(decan: Decan) -> String {
    format!(
        "{} decan {} ({}-{}) — {}",
        decan.sign,
        decan.decan_index + 1,
        decan.ruler.name(),
        decan.sub_ruler.name(),
        decan.theme,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Aries 0° → decan 1 (Mars-Mars).
    #[test]
    fn aries_first_decan() {
        let d = decan_for_longitude(0.0);
        assert_eq!(d.sign, "Aries");
        assert_eq!(d.decan_index, 0);
        assert_eq!(d.ruler, Planet::Mars);
        assert_eq!(d.sub_ruler, Planet::Mars);
    }

    /// Aries 15° → decan 2 (Mars-Sun, leadership shine).
    #[test]
    fn aries_second_decan() {
        let d = decan_for_longitude(15.0);
        assert_eq!(d.decan_index, 1);
        assert_eq!(d.sub_ruler, Planet::Sun);
    }

    /// Aries 25° → decan 3 (Mars-Venus).
    #[test]
    fn aries_third_decan() {
        let d = decan_for_longitude(25.0);
        assert_eq!(d.decan_index, 2);
        assert_eq!(d.sub_ruler, Planet::Venus);
    }

    /// 30° boundary → Taurus decan 1.
    #[test]
    fn cusp_30_taurus() {
        let d = decan_for_longitude(30.0);
        assert_eq!(d.sign, "Taurus");
        assert_eq!(d.decan_index, 0);
    }

    /// AAPL natal Sun: 21° Sagittarius → 261° absolute.
    /// 261° is in Sagittarius (240-270), specifically decan 3 (260-270°).
    #[test]
    fn aapl_natal_sun_decan() {
        let d = decan_for_longitude(261.0);
        assert_eq!(d.sign, "Sagittarius");
        assert_eq!(d.decan_index, 2);
        assert_eq!(d.ruler, Planet::Jupiter);
        assert_eq!(d.sub_ruler, Planet::Saturn);
    }

    /// AAPL natal Saturn: 16° Virgo → 166° absolute.
    /// 166° is in Virgo (150-180), specifically decan 2 (160-170°).
    #[test]
    fn aapl_natal_saturn_decan() {
        let d = decan_for_longitude(166.0);
        assert_eq!(d.sign, "Virgo");
        assert_eq!(d.decan_index, 1);
        assert_eq!(d.sub_ruler, Planet::Venus);
    }

    /// 359.5° wraps near Pisces decan 3.
    #[test]
    fn pisces_third_decan() {
        let d = decan_for_longitude(359.5);
        assert_eq!(d.sign, "Pisces");
        assert_eq!(d.decan_index, 2);
    }

    /// Negative longitude normalizes correctly.
    #[test]
    fn negative_longitude_wraps() {
        let d = decan_for_longitude(-30.0); // == 330° == Pisces 0°
        assert_eq!(d.sign, "Pisces");
        assert_eq!(d.decan_index, 0);
    }

    /// All 36 decans covered exactly — no gaps or overlaps.
    #[test]
    fn coverage_complete() {
        // Step every 1° and ensure exactly 36 unique (sign, decan_index) pairs.
        let mut seen = std::collections::HashSet::new();
        for deg in 0..360 {
            let d = decan_for_longitude(deg as f64 + 0.5); // mid-degree
            seen.insert((d.sign, d.decan_index));
        }
        assert_eq!(seen.len(), 36, "Expected 36 decans, got {}", seen.len());
    }

    /// Label formatter renders cleanly.
    #[test]
    fn label_format() {
        let d = decan_for_longitude(15.0); // Aries decan 2 Mars-Sun
        let label = decan_label(d);
        assert!(label.contains("Aries decan 2"));
        assert!(label.contains("Mars-Sun"));
        assert!(label.contains("leadership"));
    }
}
