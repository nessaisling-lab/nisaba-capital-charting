//! Wave 9.A2 — Planetary Returns.
//!
//! A "planetary return" is the moment a transiting planet returns to its
//! natal longitude. Different planets cycle at very different rates:
//!
//! | Planet  | Synodic | Returns in 50y window |
//! |---------|---------|------------------------|
//! | Mars    | ~2.0y   | ~25 (with retrograde-induced triple-passes possible) |
//! | Jupiter | ~11.86y | ~4                     |
//! | Saturn  | ~29.46y | ~1-2                   |
//!
//! Saturn return is the *big one* — once-or-twice in a corporate lifetime,
//! marks structural maturation events. Jupiter return is the 12-year
//! expansion cycle. Mars return is shorter-term but still useful for
//! cycle-aligned backtesting (~2-year price cycles).
//!
//! Search strategy:
//! Unlike the Sun, outer planets retrograde — their longitude over time
//! is a *non-monotonic* curve. So Newton's method on a single seed can
//! miss roots. We use:
//!   1. Coarse scan (planet-specific step) over the window
//!   2. For each detected sign-change in (lon - natal_lon) wrapped to
//!      [-180, 180], bisect to find the exact zero crossing
//!   3. Return all roots, ordered by date

use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate};

use super::ephemeris::{date_to_jdn, Planet};
use super::natal::NatalChart;
use super::swisseph_bridge::calc_planet_longitude_for_search;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// One planetary return event — exact moment when transit planet
/// re-occupies its natal longitude.
#[derive(Debug, Clone)]
pub struct ReturnEvent {
    pub planet:        Planet,
    pub return_date:   NaiveDate,
    pub return_jdn:    f64,
    /// Sequential index — return #1 is the first return after the natal
    /// epoch, #2 is the second, etc. For retrograde-induced triple passes,
    /// each pass gets its own number.
    pub return_number: u32,
    /// Final orb to the natal longitude after refinement (degrees).
    /// Should always be < 0.001° if the bisection converged.
    pub orb:           f64,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Wave 9.A2 — Find every return of `planet` to its natal longitude,
/// inside `window_years` after the natal epoch.
///
/// Returns are sorted by date (earliest first) and numbered sequentially.
pub fn find_returns(
    natal: &NatalChart,
    planet: Planet,
    window_years: i32,
) -> Result<Vec<ReturnEvent>> {
    let natal_pos = natal.positions.iter()
        .find(|p| p.planet == planet)
        .ok_or_else(|| anyhow!("Natal chart missing {:?} position", planet))?;
    let target_lon = natal_pos.longitude;

    // Coarse scan step — slower planets get larger steps. Mars retrograde
    // can produce three passes within ~3 months, so use 5-day step there.
    let step_days = match planet {
        Planet::Sun => 1.0,
        Planet::Moon => 0.25, // Moon returns every ~27 days
        Planet::Mercury | Planet::Venus | Planet::Mars => 5.0,
        Planet::Jupiter | Planet::Saturn | Planet::Uranus
            | Planet::Neptune | Planet::Pluto => 15.0,
        Planet::NorthNode | Planet::SouthNode | Planet::Chiron => 30.0,
    };

    // Synodic period in years — used to (a) skip the natal epoch's own
    // retrograde crossings (planet hasn't truly "left" yet) and (b) merge
    // triple-pass retrograde clusters into one return event.
    let synodic_years = match planet {
        Planet::Sun => 1.0,
        Planet::Moon => 0.0746, // ~27.3 days
        Planet::Mercury => 0.32,
        Planet::Venus => 1.6,
        Planet::Mars => 2.0,
        Planet::Jupiter => 11.86,
        Planet::Saturn => 29.46,
        Planet::Uranus => 84.0,
        Planet::Neptune => 164.8,
        Planet::Pluto => 248.0,
        Planet::NorthNode | Planet::SouthNode => 18.6,
        Planet::Chiron => 50.4,
    };

    let natal_jd = date_to_jdn(
        natal.ipo_date.year(),
        natal.ipo_date.month(),
        natal.ipo_date.day(),
        14.5,
    );
    // Skip the planet's own natal-epoch retrograde wobble. Search starts
    // 70% of one synodic in — by then the planet has truly traveled away.
    let scan_start_jd = natal_jd + 0.70 * synodic_years * 365.25;
    let end_jd = natal_jd + window_years as f64 * 365.25;

    // Scan: at each step, compute (lon - target) wrapped to [-180, 180].
    // A return = sign change between consecutive samples.
    let mut samples: Vec<(f64, f64)> = Vec::new();
    let mut jd = scan_start_jd;
    while jd < end_jd {
        if let Some(lon) = calc_planet_longitude_for_search(planet, jd) {
            let mut diff = lon - target_lon;
            while diff > 180.0 { diff -= 360.0; }
            while diff < -180.0 { diff += 360.0; }
            samples.push((jd, diff));
        }
        jd += step_days;
    }

    // Detect sign changes + bisect each. Cluster-merge returns within
    // (synodic / 3) of the previous one — that's the retrograde-pass
    // window. Astrologers usually consider triple-passes of the same
    // synodic to be one event (the central pass).
    let cluster_threshold_jd = (synodic_years / 3.0) * 365.25;
    let mut events: Vec<ReturnEvent> = Vec::new();

    for window in samples.windows(2) {
        let (jd_a, diff_a) = window[0];
        let (jd_b, diff_b) = window[1];

        // Sign change = zero crossing in [jd_a, jd_b]. Skip wraparound
        // jumps (diff shifts by > 90° in one step).
        if diff_a * diff_b >= 0.0 { continue; }
        if (diff_a - diff_b).abs() > 90.0 { continue; }

        let exact_jd = match bisect(planet, target_lon, jd_a, jd_b) {
            Some(jd) => jd,
            None => continue,
        };
        let final_diff = signed_diff(planet, target_lon, exact_jd).unwrap_or(0.0);

        // Cluster-merge: if the previous detected return is within
        // cluster_threshold, skip — this is the same retrograde cluster.
        if let Some(last) = events.last() {
            if exact_jd - last.return_jdn < cluster_threshold_jd { continue; }
        }

        let return_date = jd_to_naive_date(exact_jd);
        events.push(ReturnEvent {
            planet,
            return_date,
            return_jdn: exact_jd,
            return_number: events.len() as u32 + 1,
            orb: final_diff.abs(),
        });
    }

    Ok(events)
}

/// Wave 9.A2 — Bisect to find exact zero of (lon - target_lon) in [jd_a, jd_b].
fn bisect(planet: Planet, target_lon: f64, mut jd_a: f64, mut jd_b: f64) -> Option<f64> {
    const MAX_ITER: u32 = 40;
    const TOL: f64 = 0.001; // < 4 arcseconds

    let mut diff_a = signed_diff(planet, target_lon, jd_a)?;
    let diff_b = signed_diff(planet, target_lon, jd_b)?;
    if diff_a * diff_b > 0.0 { return None; } // No bracket

    for _ in 0..MAX_ITER {
        let mid = 0.5 * (jd_a + jd_b);
        let diff_mid = signed_diff(planet, target_lon, mid)?;
        if diff_mid.abs() < TOL { return Some(mid); }
        if diff_a * diff_mid < 0.0 {
            jd_b = mid;
        } else {
            jd_a = mid;
            diff_a = diff_mid;
        }
    }
    Some(0.5 * (jd_a + jd_b))
}

fn signed_diff(planet: Planet, target_lon: f64, jd: f64) -> Option<f64> {
    let lon = calc_planet_longitude_for_search(planet, jd)?;
    let mut diff = lon - target_lon;
    while diff > 180.0 { diff -= 360.0; }
    while diff < -180.0 { diff += 360.0; }
    Some(diff)
}

fn jd_to_naive_date(jd: f64) -> NaiveDate {
    let jd_int = (jd + 0.5).floor() as i64;
    let a = jd_int + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let day   = (e - (153 * m + 2) / 5 + 1) as u32;
    let month = (m + 3 - 12 * (m / 10)) as u32;
    let year  = (100 * b + d - 4800 + (m / 10)) as i32;
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or_else(|| {
        // Fallback to an obviously-wrong date so tests catch the bug
        NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
    })
}

// ---------------------------------------------------------------------------
// Convenience accessors
// ---------------------------------------------------------------------------

/// Wave 9.A2 — Return the next upcoming return of `planet` after `today`,
/// or `None` if none in the search window.
pub fn next_return(
    natal: &NatalChart,
    planet: Planet,
    today: NaiveDate,
    window_years: i32,
) -> Result<Option<ReturnEvent>> {
    let events = find_returns(natal, planet, window_years)?;
    Ok(events.into_iter().find(|e| e.return_date >= today))
}

/// Wave 9.A2 — Return all *Saturn* returns. Saturn return is the big
/// astrological milestone (~29.5y cycle) — for a 50-year-old company,
/// 1-2 returns total.
pub fn saturn_returns(natal: &NatalChart, window_years: i32) -> Result<Vec<ReturnEvent>> {
    find_returns(natal, Planet::Saturn, window_years)
}

/// Wave 9.A2 — Return all *Jupiter* returns. ~12-year expansion cycle.
pub fn jupiter_returns(natal: &NatalChart, window_years: i32) -> Result<Vec<ReturnEvent>> {
    find_returns(natal, Planet::Jupiter, window_years)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn aapl_natal() -> NatalChart {
        NatalChart::compute("AAPL", NaiveDate::from_ymd_opt(1980, 12, 12).unwrap())
    }

    fn _swe_lock() -> std::sync::MutexGuard<'static, ()> {
        // Use `unwrap_or_else` so a panic in a previous test doesn't
        // poison this one — the mutex serializes Swiss Eph C calls;
        // it doesn't protect data integrity.
        super::super::swisseph_bridge::SWE_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    /// Saturn return: ~29.5y cycle. AAPL IPO'd 1980 — first Saturn return
    /// should fall around 2010.
    #[test]
    fn aapl_saturn_first_return_around_2010() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let events = saturn_returns(&natal, 50).unwrap();
        assert!(!events.is_empty(), "Should have at least 1 Saturn return in 50y");
        let first = &events[0];
        assert_eq!(first.planet, Planet::Saturn);
        assert_eq!(first.return_number, 1);
        let year = first.return_date.format("%Y").to_string().parse::<i32>().unwrap();
        assert!(
            (2009..=2011).contains(&year),
            "First Saturn return should be ~2010, got {year}",
        );
    }

    /// Jupiter return: ~12y cycle. AAPL IPO'd 1980 — should see returns
    /// around 1992, 2004, 2016, 2028.
    #[test]
    fn aapl_jupiter_returns_every_12_years() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let events = jupiter_returns(&natal, 50).unwrap();
        assert!(events.len() >= 3, "Should have at least 3 Jupiter returns in 50y, got {}", events.len());
        for window in events.windows(2) {
            let delta_jd = window[1].return_jdn - window[0].return_jdn;
            let delta_years = delta_jd / 365.25;
            assert!(
                (delta_years - 11.86).abs() < 1.5,
                "Consecutive Jupiter returns should be ~11.86y apart, got {delta_years:.2}y",
            );
        }
    }

    /// Each return event must have orb < 0.01° (well within bisection
    /// tolerance) — bisection convergence sanity check.
    #[test]
    fn return_events_converge_tightly() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let events = jupiter_returns(&natal, 30).unwrap();
        for ev in &events {
            assert!(
                ev.orb < 0.01,
                "Jupiter return #{} orb should be < 0.01°, got {}",
                ev.return_number, ev.orb,
            );
        }
    }
}
