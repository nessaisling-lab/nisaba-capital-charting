//! Wave 9.A4 — Secondary Progressions ("1 day = 1 year").
//!
//! Secondary progressions are a Hellenistic-era timing technique:
//! to get the chart for year N of a person's (or company's) life,
//! advance the natal chart forward by N days. The relationship between
//! the original natal moment and the progressed moment is symbolic, not
//! literal — but the rhythm tracks remarkably well to lifecycle events.
//!
//! Key signals:
//! - **Progressed Sun** moves ~1° per year. Every ~30 years it ingresses
//!   into a new sign — a *major* personality / corporate-character shift.
//! - **Progressed Moon** moves ~12° per year, completing a full cycle
//!   every ~28 years. It ingresses into a new sign every ~2.3 years and
//!   sets the emotional/cyclic tone.
//! - **Progressed Mercury / Venus** can briefly retrograde and progressed
//!   stations are dramatic when they occur.
//! - Outer planets barely move in progression — for company timing they
//!   are essentially fixed.
//!
//! Reference: standard medieval/modern progression theory. The "1 day = 1
//! year" mapping is from Ptolemy via medieval Latin transmission (cf.
//! Bonatti's *Liber Astronomiae* tract 9).

use anyhow::Result;
use chrono::{Datelike, NaiveDate};

use super::aspects::{find_aspect, AspectType};
use super::ephemeris::{date_to_jdn, Planet, PlanetSnapshot};
use super::natal::NatalChart;
use super::swisseph_bridge::snapshot_all_precise;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// One progressed chart — natal chart advanced by (years since natal) days.
pub struct ProgressedChart {
    /// Target date the progression is for.
    pub target_date:    NaiveDate,
    /// Years (with fractional part) since natal date.
    pub years_elapsed:  f64,
    /// Equivalent calendar date that the progression "looks at" — the
    /// natal date + `years_elapsed` *days*. For AAPL 2026-05-06 this is
    /// ~1981-01-27 (45.4 days after natal).
    pub equivalent_date: NaiveDate,
    /// Snapshot of all bodies at the progressed moment.
    pub planets:        Vec<PlanetSnapshot>,
    /// Aspects between progressed planets and natal planets.
    pub aspects_to_natal: Vec<ProgressedAspect>,
}

/// One aspect between a progressed planet and a natal planet.
#[derive(Debug, Clone)]
pub struct ProgressedAspect {
    pub progressed_planet: Planet,
    pub natal_planet:      Planet,
    pub aspect:            AspectType,
    pub orb:               f64,
}

/// Marker indicating a progressed Sun or Moon has crossed a sign cusp.
/// Used by the dashboard to surface "Progressed Sun entering Aquarius
/// next month — major character shift" type signals.
#[derive(Debug, Clone)]
pub struct SignIngress {
    pub planet:        Planet,
    pub from_sign:     &'static str,
    pub to_sign:       &'static str,
    /// Date of the sign-cusp crossing. May be in the past (recent) or
    /// future (upcoming).
    pub ingress_date:  NaiveDate,
    /// Whether this is in the past (relative to target_date) or future.
    pub days_offset:   i64,
}

// ---------------------------------------------------------------------------
// Compute
// ---------------------------------------------------------------------------

/// Wave 9.A4 — Compute the progressed chart for `target_date` from `natal`.
///
/// Returns `Ok` even when years_elapsed is fractional or 0; the chart is
/// always defined. Aspects use the same orb table as transit detection.
pub fn compute_progressed_chart(natal: &NatalChart, target_date: NaiveDate) -> Result<ProgressedChart> {
    // ── Years elapsed (fractional) ────────────────────────────────
    let natal_date = natal.ipo_date;
    let days_elapsed = (target_date - natal_date).num_days();
    let years_elapsed = days_elapsed as f64 / 365.25;

    // ── Compute progressed JD = natal_jd + years_elapsed days ─────
    let natal_jd = date_to_jdn(
        natal_date.year(), natal_date.month(), natal_date.day(),
        14.5, // Match natal compute time (09:30 EST = 14:30 UT)
    );
    let progressed_jd = natal_jd + years_elapsed;

    // ── Equivalent calendar date (for UI) ─────────────────────────
    let equivalent_date = jd_to_naive_date(progressed_jd);

    // ── Cast progressed chart ──────────────────────────────────────
    let planets = snapshot_all_precise(progressed_jd);

    // ── Find progressed-to-natal aspects ──────────────────────────
    let mut aspects = Vec::new();
    for prog_p in &planets {
        for natal_p in &natal.positions {
            // Same-planet conjunction is uninteresting (the planet hasn't
            // moved much by definition), so skip Sun-Sun, Moon-Moon, etc.
            if prog_p.planet == natal_p.planet { continue; }
            if let Some((aspect, orb)) = find_aspect(prog_p.longitude, natal_p.longitude) {
                aspects.push(ProgressedAspect {
                    progressed_planet: prog_p.planet,
                    natal_planet:      natal_p.planet,
                    aspect,
                    orb,
                });
            }
        }
    }
    aspects.sort_by(|a, b| a.orb.partial_cmp(&b.orb).unwrap_or(std::cmp::Ordering::Equal));

    Ok(ProgressedChart {
        target_date,
        years_elapsed,
        equivalent_date,
        planets,
        aspects_to_natal: aspects,
    })
}

/// Wave 9.A4 — Find upcoming progressed-Sun and progressed-Moon sign
/// ingresses within `window_years` of `target_date`. These are slow
/// signals: progressed Sun ingress every ~30 years, progressed Moon
/// every ~2.3 years.
pub fn upcoming_sign_ingresses(
    natal: &NatalChart,
    target_date: NaiveDate,
    window_years: i32,
) -> Result<Vec<SignIngress>> {
    let mut ingresses = Vec::new();

    // Compute the "current" progressed Sun + Moon signs.
    let current = compute_progressed_chart(natal, target_date)?;
    let current_sun_sign = current.planets.iter()
        .find(|p| p.planet == Planet::Sun)
        .map(|p| p.sign);
    let current_moon_sign = current.planets.iter()
        .find(|p| p.planet == Planet::Moon)
        .map(|p| p.sign);

    // Step day-by-day looking for sign changes. 1 day = 1 year for
    // progressions, so window_years calendar years = window_years actual
    // calendar days of natal-time advancement.  Use 30-day step (about
    // 1 progressed month).
    let mut probe = target_date;
    let mut last_sun = current_sun_sign;
    let mut last_moon = current_moon_sign;
    for _ in 0..(window_years * 12) {
        probe += chrono::Duration::days(30);
        let chart = match compute_progressed_chart(natal, probe) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let sun_sign = chart.planets.iter().find(|p| p.planet == Planet::Sun).map(|p| p.sign);
        let moon_sign = chart.planets.iter().find(|p| p.planet == Planet::Moon).map(|p| p.sign);

        if let (Some(prev), Some(now)) = (last_sun, sun_sign) {
            if prev != now {
                let days_offset = (probe - target_date).num_days();
                ingresses.push(SignIngress {
                    planet: Planet::Sun,
                    from_sign: prev,
                    to_sign: now,
                    ingress_date: probe,
                    days_offset,
                });
                last_sun = sun_sign;
            }
        }
        if let (Some(prev), Some(now)) = (last_moon, moon_sign) {
            if prev != now {
                let days_offset = (probe - target_date).num_days();
                ingresses.push(SignIngress {
                    planet: Planet::Moon,
                    from_sign: prev,
                    to_sign: now,
                    ingress_date: probe,
                    days_offset,
                });
                last_moon = moon_sign;
            }
        }
    }

    Ok(ingresses)
}

/// Wave 9.A4 — Format the progressed chart's most-impactful state into
/// one UI line. Lead with progressed Sun sign + degree, then strongest
/// progressed-natal aspect.
pub fn summary_line(prog: &ProgressedChart) -> String {
    let prog_sun = prog.planets.iter().find(|p| p.planet == Planet::Sun);
    let lead_aspect = prog.aspects_to_natal.first();

    let sun_part = match prog_sun {
        Some(p) => format!("Prog. Sun {:.1}° {}", p.degree, p.sign),
        None => "Prog. Sun unknown".to_string(),
    };
    let aspect_part = match lead_aspect {
        Some(a) => format!("Prog. {} {} natal {} ({:.1}°)",
            a.progressed_planet.name(),
            a.aspect.name(),
            a.natal_planet.name(),
            a.orb,
        ),
        None => "no tight prog. aspects".to_string(),
    };
    format!("{} · {}", sun_part, aspect_part)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
    NaiveDate::from_ymd_opt(year, month, day)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aapl_natal() -> NatalChart {
        NatalChart::compute("AAPL", NaiveDate::from_ymd_opt(1980, 12, 12).unwrap())
    }

    fn _swe_lock() -> std::sync::MutexGuard<'static, ()> {
        super::super::swisseph_bridge::SWE_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    /// AAPL 2026-05-06: ~45.4 years elapsed → progressed chart looks at
    /// natal_date + 45.4 days = ~1981-01-26.
    #[test]
    fn aapl_2026_progressed_equiv_date() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let target = NaiveDate::from_ymd_opt(2026, 5, 6).unwrap();
        let prog = compute_progressed_chart(&natal, target).unwrap();
        assert!(
            (prog.years_elapsed - 45.4).abs() < 0.1,
            "Expected ~45.4 years elapsed, got {}",
            prog.years_elapsed,
        );
        // Equivalent date should be ~Jan 26, 1981
        let expected = NaiveDate::from_ymd_opt(1981, 1, 26).unwrap();
        let delta = (prog.equivalent_date - expected).num_days().abs();
        assert!(delta <= 2, "Equivalent date {} too far from {}", prog.equivalent_date, expected);
    }

    /// Progressed Sun should have moved ~45° from natal Sun (1°/yr × 45y).
    /// Natal AAPL Sun ~261° → Progressed Sun ~306° (~6° Aquarius).
    #[test]
    fn aapl_2026_progressed_sun_moved_45deg() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let target = NaiveDate::from_ymd_opt(2026, 5, 6).unwrap();
        let prog = compute_progressed_chart(&natal, target).unwrap();
        let prog_sun = prog.planets.iter().find(|p| p.planet == Planet::Sun).unwrap();
        let natal_sun = natal.positions.iter().find(|p| p.planet == Planet::Sun).unwrap();
        let mut diff = prog_sun.longitude - natal_sun.longitude;
        if diff < 0.0 { diff += 360.0; }
        // Sun moves ~0.985°/day → 45.4 days × 0.985 ≈ 44.7°
        assert!(
            (diff - 44.7).abs() < 2.0,
            "Progressed Sun should be ~44.7° past natal Sun, got delta={diff:.1}°",
        );
    }

    /// Progressed outer planets should be nearly stationary.
    /// Pluto's daily motion is ~0.04° at apparent peak, so 45 days ≈
    /// max ~1.5°. Verify Pluto moves *less* than the Sun (Sun moves
    /// ~45° in 45 days).
    #[test]
    fn aapl_progressed_pluto_slower_than_sun() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let target = NaiveDate::from_ymd_opt(2026, 5, 6).unwrap();
        let prog = compute_progressed_chart(&natal, target).unwrap();
        let prog_pluto = prog.planets.iter().find(|p| p.planet == Planet::Pluto).unwrap();
        let natal_pluto = natal.positions.iter().find(|p| p.planet == Planet::Pluto).unwrap();
        let prog_sun = prog.planets.iter().find(|p| p.planet == Planet::Sun).unwrap();
        let natal_sun = natal.positions.iter().find(|p| p.planet == Planet::Sun).unwrap();

        let pluto_diff = {
            let mut d = (prog_pluto.longitude - natal_pluto.longitude).abs();
            if d > 180.0 { d = 360.0 - d; }
            d
        };
        let sun_diff = {
            let mut d = (prog_sun.longitude - natal_sun.longitude).abs();
            if d > 180.0 { d = 360.0 - d; }
            d
        };
        assert!(
            pluto_diff < sun_diff,
            "Pluto ({pluto_diff:.3}°) should move slower than Sun ({sun_diff:.3}°)",
        );
        // And Pluto motion should be < 2° in 45 days as a sanity bound.
        assert!(pluto_diff < 2.0, "Pluto progressed motion unrealistic: {pluto_diff:.3}°");
    }

    /// Aspects between progressed and natal planets must NOT include
    /// same-planet pairings (Sun-Sun is by construction conjunction, not
    /// useful information).
    #[test]
    fn no_same_planet_aspects() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let target = NaiveDate::from_ymd_opt(2026, 5, 6).unwrap();
        let prog = compute_progressed_chart(&natal, target).unwrap();
        for asp in &prog.aspects_to_natal {
            assert_ne!(asp.progressed_planet, asp.natal_planet,
                "Same-planet aspect should be filtered out: {:?}", asp);
        }
    }

    /// Natal date as target → 0 years elapsed → progressed chart equals
    /// natal chart for all bodies.
    #[test]
    fn natal_date_zero_progression() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let prog = compute_progressed_chart(&natal, natal.ipo_date).unwrap();
        assert!(prog.years_elapsed.abs() < 0.001);
        assert_eq!(prog.equivalent_date, natal.ipo_date);
    }

    /// Summary line renders cleanly.
    #[test]
    fn summary_line_renders() {
        let _g = _swe_lock();
        let natal = aapl_natal();
        let target = NaiveDate::from_ymd_opt(2026, 5, 6).unwrap();
        let prog = compute_progressed_chart(&natal, target).unwrap();
        let s = summary_line(&prog);
        assert!(s.contains("Prog. Sun"));
    }
}
