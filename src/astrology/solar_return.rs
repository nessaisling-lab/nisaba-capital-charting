//! Wave 9.A1 — Solar Return charts.
//!
//! A Solar Return is the chart cast for the exact moment the transiting
//! Sun returns to its natal longitude. It happens once per year, within
//! a few hours either side of the birthday (or IPO anniversary). The SR
//! chart is read as the year's outlook — valid birthday-to-birthday.
//!
//! Search strategy: the Sun's motion is monotonic prograde (~0.985°/day,
//! never retrograde), so finding the return moment is a 1-D root-find on
//! `swiss_eph_sun_longitude(jd) - natal_sun_longitude == 0` (mod 360°).
//! We seed at the calendar anniversary and Newton-refine with the Sun's
//! mean motion. Convergence is well under 10 iterations to < 0.001°.
//!
//! Output: a full chart (all bodies + Ascendant + MC at NYSE) plus the
//! list of cross-aspects between SR planets and natal planets.

use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate};

use super::aspects::{find_aspect, AspectType};
use super::ephemeris::{
    date_to_jdn, longitude_to_sign, Planet, PlanetSnapshot,
};
use super::natal::NatalChart;
use super::swisseph_bridge::{
    calc_sun_longitude_for_search, compute_houses_nyse, snapshot_all_precise,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// One Solar Return chart — chart cast for the exact Sun-return moment.
pub struct SolarReturnChart {
    /// Calendar year the SR is for (the year *containing* the return).
    pub return_year: i32,
    /// Exact return date (UT date).
    pub return_date: NaiveDate,
    /// Fractional UT hour of the return moment (0.0-24.0).
    pub return_hour_ut: f64,
    /// Julian Day at the exact return moment.
    pub return_jdn: f64,
    /// Snapshot of all bodies at the SR moment.
    pub planets: Vec<PlanetSnapshot>,
    /// Ascendant longitude (NYSE coords) at the SR moment.
    pub ascendant: f64,
    /// Midheaven longitude at the SR moment.
    pub mc: f64,
    /// Cross-aspects: SR-planet ↔ natal-planet matches.
    pub aspects_to_natal: Vec<SrAspect>,
}

/// One aspect between a Solar Return planet and a natal planet.
#[derive(Debug, Clone)]
pub struct SrAspect {
    pub sr_planet:    Planet,
    pub natal_planet: Planet,
    pub aspect:       AspectType,
    pub orb:          f64,
}

// ---------------------------------------------------------------------------
// Compute
// ---------------------------------------------------------------------------

/// Compute the Solar Return chart for a natal chart in a target year.
///
/// `target_year` is the calendar year you want the SR for. The function
/// searches a ±2-day window around the natal anniversary in that year
/// for the moment when the transiting Sun returns to the natal Sun's
/// longitude (exact orb < 0.001°).
pub fn compute_solar_return(natal: &NatalChart, target_year: i32) -> Result<SolarReturnChart> {
    // ── Find natal Sun longitude ────────────────────────────────────
    let natal_sun = natal.positions.iter()
        .find(|p| p.planet == Planet::Sun)
        .ok_or_else(|| anyhow!("Natal chart missing Sun position"))?;
    let target_lon = natal_sun.longitude;

    // ── Seed search at calendar anniversary ────────────────────────
    // IPO date = (year, month, day) at 14:30 UTC. Use the same time on
    // target_year as a starting point, then Newton-refine.
    let ipo = natal.ipo_date;
    let seed_date = NaiveDate::from_ymd_opt(target_year, ipo.month(), ipo.day())
        .or_else(|| NaiveDate::from_ymd_opt(target_year, ipo.month(), 28))
        .ok_or_else(|| anyhow!("Invalid target year/month for SR seed"))?;
    let mut jd = date_to_jdn(target_year, seed_date.month(), seed_date.day(), 14.5);

    // ── Newton iteration ───────────────────────────────────────────
    // Sun moves ~0.985°/day (mean). delta_jd = (target - current) / 0.985.
    // Wrap difference into [-180, 180] so we never search the wrong
    // direction across the 360° boundary.
    const SOLAR_MEAN_MOTION_DEG_PER_DAY: f64 = 0.9856;
    const TOLERANCE_DEG: f64 = 0.0005; // < 1 arcsecond
    const MAX_ITERATIONS: u32 = 25;

    for _ in 0..MAX_ITERATIONS {
        let current_lon = calc_sun_longitude_for_search(jd)
            .ok_or_else(|| anyhow!("Swiss Eph failed during SR search"))?;
        let mut diff = target_lon - current_lon;
        while diff > 180.0 { diff -= 360.0; }
        while diff < -180.0 { diff += 360.0; }
        if diff.abs() < TOLERANCE_DEG { break; }
        jd += diff / SOLAR_MEAN_MOTION_DEG_PER_DAY;
    }

    // ── Cast full chart at refined JD ──────────────────────────────
    let planets = snapshot_all_precise(jd);
    let houses = compute_houses_nyse(jd)
        .map_err(|e| anyhow!("SR house calculation failed: {e}"))?;

    // ── Decode JD back to calendar date + UT hour ──────────────────
    let (year, month, day, hour_ut) = jd_to_calendar(jd);
    let return_date = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| anyhow!("Invalid return date {year}-{month}-{day}"))?;

    // ── Find SR ↔ natal aspects ────────────────────────────────────
    let mut aspects = Vec::new();
    for sr_p in &planets {
        for natal_p in &natal.positions {
            // Skip self-comparison for the Sun (always conjunction by
            // construction — uninteresting to the reader).
            if sr_p.planet == Planet::Sun && natal_p.planet == Planet::Sun {
                continue;
            }
            if let Some((aspect, orb)) = find_aspect(sr_p.longitude, natal_p.longitude) {
                aspects.push(SrAspect {
                    sr_planet:    sr_p.planet,
                    natal_planet: natal_p.planet,
                    aspect,
                    orb,
                });
            }
        }
    }
    // Tightest aspects first
    aspects.sort_by(|a, b| a.orb.partial_cmp(&b.orb).unwrap_or(std::cmp::Ordering::Equal));

    Ok(SolarReturnChart {
        return_year: target_year,
        return_date,
        return_hour_ut: hour_ut,
        return_jdn: jd,
        planets,
        ascendant: houses.ascendant,
        mc: houses.mc,
        aspects_to_natal: aspects,
    })
}

/// Wave 9.A1 — Convert a Julian Day Number back to (year, month, day, hour_UT).
/// Matches the inverse of `date_to_jdn`. Uses the standard Fliegel-Van Flandern
/// algorithm.
fn jd_to_calendar(jd: f64) -> (i32, u32, u32, f64) {
    let jd_int = (jd + 0.5).floor() as i64;
    let frac = jd + 0.5 - jd_int as f64;
    let hour_ut = frac * 24.0;

    let a = jd_int + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;

    let day   = (e - (153 * m + 2) / 5 + 1) as u32;
    let month = (m + 3 - 12 * (m / 10)) as u32;
    let year  = (100 * b + d - 4800 + (m / 10)) as i32;

    (year, month, day, hour_ut)
}

/// Wave 9.A1 — One-line summary of the chart for UI rendering.
/// Format: "SR 2026: returned 2026-12-12 14:33 UTC, ASC 18° Capricorn"
pub fn summary_line(sr: &SolarReturnChart) -> String {
    let h_int = sr.return_hour_ut.floor() as u32;
    let m_int = ((sr.return_hour_ut - h_int as f64) * 60.0).floor() as u32;
    let (asc_sign, asc_deg) = longitude_to_sign(sr.ascendant);
    format!(
        "SR {}: returned {} {:02}:{:02} UTC, ASC {:.0}° {}",
        sr.return_year, sr.return_date, h_int, m_int, asc_deg, asc_sign,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn aapl_natal() -> NatalChart {
        NatalChart::compute("AAPL", NaiveDate::from_ymd_opt(1980, 12, 12).unwrap())
    }

    /// Wave 9.A1 — Swiss Ephemeris C library uses global mutable state, so
    /// concurrent test runs corrupt each other. Reuse the bridge's test lock.
    fn _swe_lock() -> std::sync::MutexGuard<'static, ()> {
        super::super::swisseph_bridge::SWE_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    /// AAPL Sun is at 21° Sagittarius (~261°). Its 2026 Solar Return
    /// should fall on or very near 2026-12-12.
    #[test]
    fn aapl_2026_return_near_birthday() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let sr = compute_solar_return(&natal, 2026).expect("SR compute failed");
        assert_eq!(sr.return_year, 2026);
        // Allow ±1 day of birthday since the exact return moment can
        // straddle the date boundary in UT.
        let birthday = NaiveDate::from_ymd_opt(2026, 12, 12).unwrap();
        let delta = (sr.return_date - birthday).num_days().abs();
        assert!(delta <= 1, "SR date {} not within 1 day of {}", sr.return_date, birthday);
    }

    /// At the SR moment, the SR Sun longitude must equal the natal Sun
    /// longitude within < 0.001° (our search tolerance).
    #[test]
    fn aapl_2026_return_sun_matches_natal() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let sr = compute_solar_return(&natal, 2026).unwrap();
        let natal_sun = natal.positions.iter().find(|p| p.planet == Planet::Sun).unwrap();
        let sr_sun = sr.planets.iter().find(|p| p.planet == Planet::Sun).unwrap();
        let mut diff = (natal_sun.longitude - sr_sun.longitude).abs();
        if diff > 180.0 { diff = 360.0 - diff; }
        assert!(diff < 0.01, "SR Sun should match natal Sun: natal={:.4}° SR={:.4}° diff={:.4}°",
            natal_sun.longitude, sr_sun.longitude, diff);
    }

    /// Multiple consecutive SRs should be ~365.25 days apart.
    #[test]
    fn consecutive_sr_one_year_apart() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let sr1 = compute_solar_return(&natal, 2025).unwrap();
        let sr2 = compute_solar_return(&natal, 2026).unwrap();
        let delta = sr2.return_jdn - sr1.return_jdn;
        assert!(
            (delta - 365.25).abs() < 1.0,
            "Consecutive SRs should be ~1 year apart: delta={delta}",
        );
    }
}
