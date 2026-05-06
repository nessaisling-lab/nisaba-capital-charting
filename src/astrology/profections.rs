//! Wave 9.A3 — Profections (Hellenistic annual time-lord).
//!
//! Profections are a Hellenistic timing technique. Starting from the 1st
//! house (Ascendant) at age 0, the "profected house" advances by one each
//! year of life. The traditional ruler of the sign on that profected
//! house's cusp becomes the year's *time-lord* — the planet that "governs"
//! the chapter. Twelve houses → 12-year cycle.
//!
//! Example for a stock with a Capricorn ASC:
//! - Age 0  → 1st house (Capricorn) → Lord: Saturn
//! - Age 1  → 2nd house (Aquarius)  → Lord: Saturn
//! - Age 2  → 3rd house (Pisces)    → Lord: Jupiter
//! - Age 5  → 6th house (Gemini)    → Lord: Mercury
//! - Age 12 → back to 1st house, new Saturn-year
//!
//! The time-lord receives a +50% strength multiplier on aspects in
//! `score_aspect_v2`, so the profected planet's transit network for the
//! year compounds with its time-lord status.
//!
//! Monthly profections: within the yearly house, each calendar-month
//! steps one further sign — gives a sub-lord. Daily profections (one
//! sign per ~2.5 days) exist but we don't model them here.
//!
//! Reference: Hellenistic Astrology by Chris Brennan §16-17, or
//! Antiochus's *Definitions and Foundations* (1st-c CE).

use chrono::{Datelike, NaiveDate};

use super::ephemeris::Planet;

// ---------------------------------------------------------------------------
// Sign + ruler helpers
// ---------------------------------------------------------------------------

const SIGN_NAMES: [&str; 12] = [
    "Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo",
    "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces",
];

/// Wave 9.A3 — Traditional Hellenistic sign rulers (no modern outers).
/// Aquarius → Saturn (not Uranus); Pisces → Jupiter (not Neptune);
/// Scorpio → Mars (not Pluto). Hellenistic profection convention.
pub fn traditional_ruler(sign_index: usize) -> Planet {
    match sign_index % 12 {
        0  => Planet::Mars,    // Aries
        1  => Planet::Venus,   // Taurus
        2  => Planet::Mercury, // Gemini
        3  => Planet::Moon,    // Cancer
        4  => Planet::Sun,     // Leo
        5  => Planet::Mercury, // Virgo
        6  => Planet::Venus,   // Libra
        7  => Planet::Mars,    // Scorpio (traditional, not Pluto)
        8  => Planet::Jupiter, // Sagittarius
        9  => Planet::Saturn,  // Capricorn
        10 => Planet::Saturn,  // Aquarius (traditional, not Uranus)
        _  => Planet::Jupiter, // Pisces (traditional, not Neptune)
    }
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// One profection — yearly + monthly time-lords for a target date.
#[derive(Debug, Clone)]
pub struct Profection {
    /// Age in years (floor) on the target date relative to natal date.
    pub age: u32,
    /// Profected house number (1-12).
    pub profected_house: u8,
    /// Sign on the profected house's cusp.
    pub sign: &'static str,
    /// Traditional ruler of the profected-house sign — the year's
    /// **time-lord**.
    pub lord_planet: Planet,
    /// Sub-lord (monthly profection, ruler of next sign per month).
    pub monthly_house: u8,
    pub monthly_sign: &'static str,
    pub monthly_lord: Planet,
    /// Months elapsed since last natal anniversary (0-11).
    pub months_into_year: u8,
}

/// Wave 9.A3 — Compute the profection state for a chart on `target_date`.
///
/// Requires the natal chart's Ascendant longitude (used to determine the
/// 1st-house sign). Returns `None` if no Ascendant is available — the
/// dashboard's natal_angles table must be populated.
pub fn compute_profection(
    natal_date: NaiveDate,
    ascendant_lon: f64,
    target_date: NaiveDate,
) -> Profection {
    // ── Age in completed years + months-into-year ──────────────────
    let mut age = (target_date.year() - natal_date.year()) as i32;
    // Subtract a year if we haven't passed the anniversary in target_date's year
    let anniv = NaiveDate::from_ymd_opt(target_date.year(), natal_date.month(), natal_date.day())
        .unwrap_or(natal_date);
    if target_date < anniv { age -= 1; }
    let age = age.max(0) as u32;

    // Last anniversary = anniv if past, otherwise previous year's
    let last_anniv = if target_date >= anniv {
        anniv
    } else {
        NaiveDate::from_ymd_opt(target_date.year() - 1, natal_date.month(), natal_date.day())
            .unwrap_or(natal_date)
    };
    let days_since_anniv = (target_date - last_anniv).num_days();
    let months_into_year = (days_since_anniv / 30).clamp(0, 11) as u8;

    // ── Profected house from Ascendant ─────────────────────────────
    let asc_sign_index = ((ascendant_lon % 360.0 + 360.0) % 360.0 / 30.0).floor() as usize % 12;
    let house_offset = (age % 12) as usize;
    let profected_sign_index = (asc_sign_index + house_offset) % 12;
    let profected_house = (house_offset + 1) as u8;
    let sign = SIGN_NAMES[profected_sign_index];
    let lord_planet = traditional_ruler(profected_sign_index);

    // ── Monthly sub-lord (further `months_into_year` signs forward) ─
    let monthly_sign_index = (asc_sign_index + house_offset + months_into_year as usize) % 12;
    let monthly_house = ((house_offset + months_into_year as usize) % 12 + 1) as u8;
    let monthly_sign = SIGN_NAMES[monthly_sign_index];
    let monthly_lord = traditional_ruler(monthly_sign_index);

    Profection {
        age,
        profected_house,
        sign,
        lord_planet,
        monthly_house,
        monthly_sign,
        monthly_lord,
        months_into_year,
    }
}

/// Wave 9.A3 — Strength multiplier for an aspect involving the year's
/// time-lord. Astrologically, the time-lord's transit network for the
/// year is highlighted; everything they touch gets weight. +50%.
pub const TIME_LORD_MULTIPLIER: f64 = 1.5;

/// Wave 9.A3 — Convenience: is the planet the year's time-lord?
pub fn is_time_lord(prof: &Profection, planet: Planet) -> bool {
    prof.lord_planet == planet
}

/// Wave 9.A3 — One-line UI summary.
/// Format: "Year of Mars (5th house · Scorpio)"
pub fn summary_line(prof: &Profection) -> String {
    format!(
        "Year of {} ({}{} house · {})",
        prof.lord_planet.name(),
        prof.profected_house,
        ordinal_suffix(prof.profected_house),
        prof.sign,
    )
}

fn ordinal_suffix(n: u8) -> &'static str {
    match n % 10 {
        1 if n != 11 => "st",
        2 if n != 12 => "nd",
        3 if n != 13 => "rd",
        _ => "th",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Capricorn ASC, age 0 → 1st house (Capricorn) → Saturn.
    #[test]
    fn capricorn_asc_age_0_saturn_year() {
        let natal = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let target = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let asc = 280.0; // 10° Capricorn
        let p = compute_profection(natal, asc, target);
        assert_eq!(p.age, 0);
        assert_eq!(p.profected_house, 1);
        assert_eq!(p.sign, "Capricorn");
        assert_eq!(p.lord_planet, Planet::Saturn);
    }

    /// Capricorn ASC, age 12 → cycle back to 1st house, Saturn again.
    #[test]
    fn capricorn_asc_age_12_back_to_saturn() {
        let natal = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let target = NaiveDate::from_ymd_opt(2012, 1, 1).unwrap();
        let asc = 280.0;
        let p = compute_profection(natal, asc, target);
        assert_eq!(p.age, 12);
        assert_eq!(p.profected_house, 1);
        assert_eq!(p.lord_planet, Planet::Saturn);
    }

    /// Capricorn ASC, age 5 → 6th house = Gemini → Mercury.
    #[test]
    fn capricorn_asc_age_5_mercury_year() {
        let natal = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let target = NaiveDate::from_ymd_opt(2005, 1, 2).unwrap();
        let asc = 280.0;
        let p = compute_profection(natal, asc, target);
        assert_eq!(p.age, 5);
        assert_eq!(p.profected_house, 6);
        assert_eq!(p.sign, "Gemini");
        assert_eq!(p.lord_planet, Planet::Mercury);
    }

    /// AAPL: IPO 1980-12-12 with Capricorn ASC (~14° Capricorn).
    /// On 2026-05-06, age = 45 (since pre-Dec-12 birthday).
    /// 45 mod 12 = 9 → 10th house from Capricorn = Libra → Venus.
    #[test]
    fn aapl_2026_05_06_year_of_venus() {
        let natal = NaiveDate::from_ymd_opt(1980, 12, 12).unwrap();
        let target = NaiveDate::from_ymd_opt(2026, 5, 6).unwrap();
        let asc = 284.0; // ~14° Capricorn
        let p = compute_profection(natal, asc, target);
        assert_eq!(p.age, 45);
        assert_eq!(p.profected_house, 10);
        assert_eq!(p.sign, "Libra");
        assert_eq!(p.lord_planet, Planet::Venus);
    }

    /// Pre-anniversary in same year → age decrements correctly.
    #[test]
    fn pre_anniversary_decrements_age() {
        let natal = NaiveDate::from_ymd_opt(2000, 6, 15).unwrap();
        let target = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(); // Before Jun 15 anniv
        let asc = 0.0;
        let p = compute_profection(natal, asc, target);
        assert_eq!(p.age, 24); // not yet 25
    }

    /// Time-lord recognition.
    #[test]
    fn time_lord_recognized() {
        let natal = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let target = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let asc = 280.0; // Capricorn → Saturn-year
        let p = compute_profection(natal, asc, target);
        assert!(is_time_lord(&p, Planet::Saturn));
        assert!(!is_time_lord(&p, Planet::Mars));
    }

    /// Monthly sub-lord advances one sign per ~30 days from anniversary.
    #[test]
    fn monthly_sub_lord_advances() {
        let natal = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let asc = 280.0; // Capricorn
        // Month 0 = same as yearly lord
        let p0 = compute_profection(natal, asc, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        assert_eq!(p0.months_into_year, 0);
        assert_eq!(p0.monthly_sign, "Capricorn");
        // Month 3 = 3 signs forward = Aries → Mars sub-lord
        let p3 = compute_profection(natal, asc, NaiveDate::from_ymd_opt(2000, 4, 1).unwrap());
        assert_eq!(p3.months_into_year, 3);
        assert_eq!(p3.monthly_sign, "Aries");
        assert_eq!(p3.monthly_lord, Planet::Mars);
    }

    /// Summary line formatter renders cleanly.
    #[test]
    fn summary_line_format() {
        let natal = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let asc = 280.0;
        let p = compute_profection(natal, asc, NaiveDate::from_ymd_opt(2005, 1, 2).unwrap());
        let s = summary_line(&p);
        assert!(s.contains("Year of Mercury"));
        assert!(s.contains("6th house"));
        assert!(s.contains("Gemini"));
    }

    /// Traditional rulers — Scorpio is Mars (not Pluto).
    #[test]
    fn traditional_ruler_scorpio_is_mars() {
        assert_eq!(traditional_ruler(7), Planet::Mars);
    }

    /// Traditional rulers — Aquarius is Saturn (not Uranus).
    #[test]
    fn traditional_ruler_aquarius_is_saturn() {
        assert_eq!(traditional_ruler(10), Planet::Saturn);
    }
}
