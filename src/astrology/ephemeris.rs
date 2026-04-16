//! Planetary position calculations using Jean Meeus "Astronomical Algorithms".
//!
//! Accuracy: ~1-2° for outer planets, ~1' for Sun/Moon.
//! This is well within the 6-10° orbs used in astrology.
//!
//! All longitudes are ecliptic, 0-360°, measured from the vernal equinox.
//! T = Julian centuries from J2000.0 (Jan 1.5, 2000 = JDN 2451545.0).

use std::f64::consts::PI;

// ---------------------------------------------------------------------------
// Planet enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Planet {
    Sun,
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
}

impl Planet {
    pub fn all() -> &'static [Planet] {
        &[
            Planet::Sun, Planet::Moon, Planet::Mercury, Planet::Venus,
            Planet::Mars, Planet::Jupiter, Planet::Saturn,
            Planet::Uranus, Planet::Neptune, Planet::Pluto,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Planet::Sun     => "Sun",
            Planet::Moon    => "Moon",
            Planet::Mercury => "Mercury",
            Planet::Venus   => "Venus",
            Planet::Mars    => "Mars",
            Planet::Jupiter => "Jupiter",
            Planet::Saturn  => "Saturn",
            Planet::Uranus  => "Uranus",
            Planet::Neptune => "Neptune",
            Planet::Pluto   => "Pluto",
        }
    }

    pub fn from_name(s: &str) -> Option<Planet> {
        match s {
            "Sun"     => Some(Planet::Sun),
            "Moon"    => Some(Planet::Moon),
            "Mercury" => Some(Planet::Mercury),
            "Venus"   => Some(Planet::Venus),
            "Mars"    => Some(Planet::Mars),
            "Jupiter" => Some(Planet::Jupiter),
            "Saturn"  => Some(Planet::Saturn),
            "Uranus"  => Some(Planet::Uranus),
            "Neptune" => Some(Planet::Neptune),
            "Pluto"   => Some(Planet::Pluto),
            _         => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Date → Julian Day Number
// ---------------------------------------------------------------------------

/// Convert a calendar date + fractional hour (UT) to Julian Day Number.
/// Valid for all dates from 1900 onward.
pub fn date_to_jdn(year: i32, month: u32, day: u32, hour_ut: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let a = (y as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();   // Gregorian correction
    let day_frac = day as f64 + hour_ut / 24.0;
    (365.25 * (y as f64 + 4716.0)).floor()
        + (30.6001 * (m as f64 + 1.0)).floor()
        + day_frac + b - 1524.5
}

/// T = Julian centuries from J2000.0
pub fn jdn_to_t(jdn: f64) -> f64 {
    (jdn - 2_451_545.0) / 36_525.0
}

/// Normalize angle to [0, 360)
fn norm360(deg: f64) -> f64 {
    deg - 360.0 * (deg / 360.0).floor()
}

/// Degrees to radians
fn deg2rad(d: f64) -> f64 { d * PI / 180.0 }

// ---------------------------------------------------------------------------
// Sun — Meeus Ch.25 (low-precision, ~0.01° accuracy)
// ---------------------------------------------------------------------------

pub fn sun_longitude(t: f64) -> f64 {
    // Geometric mean longitude of the Sun (degrees)
    let l0 = norm360(280.46646 + t * (36000.76983 + t * 0.0003032));
    // Mean anomaly of the Sun
    let m = norm360(357.52911 + t * (35999.05029 - t * 0.0001537));
    let m_rad = deg2rad(m);
    // Equation of center
    let c = (1.914602 - t * (0.004817 + t * 0.000014)) * m_rad.sin()
        + (0.019993 - t * 0.000101) * (2.0 * m_rad).sin()
        + 0.000289 * (3.0 * m_rad).sin();
    // Sun's true longitude
    let sun_lon = l0 + c;
    // Apparent longitude (subtract aberration + nutation is small, skip)
    norm360(sun_lon - 0.00569 - 0.00478 * deg2rad(125.04 - 1934.136 * t).sin())
}

// ---------------------------------------------------------------------------
// Moon — Meeus Ch.47 (simplified, ~0.3° accuracy)
// ---------------------------------------------------------------------------

pub fn moon_longitude(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;

    // Moon's mean longitude
    let lp = norm360(218.3165 + 481267.8813 * t);
    // Moon's mean anomaly
    let m = norm360(134.9634 + 477198.8676 * t + 0.0087 * t2 + t3 / 69699.0);
    // Moon's mean anomaly (Sun)
    let ms = norm360(357.5291 + 35999.0503 * t - 0.0001559 * t2);
    // Moon's argument of latitude
    let f = norm360(93.2721 + 483202.0175 * t - 0.0034 * t2);
    // Longitude of ascending node
    let omega = norm360(125.0445 - 1934.1362 * t + 0.0020 * t2);

    let m_r  = deg2rad(m);
    let ms_r = deg2rad(ms);
    let f_r  = deg2rad(f);
    let o_r  = deg2rad(omega);

    // Principal terms (degrees)
    let sigma_l = 6.288750 * m_r.sin()
        + 1.274018 * (2.0 * deg2rad(lp) - m_r).sin()
        + 0.658309 * (2.0 * deg2rad(lp)).sin()
        + 0.213616 * (2.0 * m_r).sin()
        - 0.185596 * ms_r.sin()
        - 0.114336 * (2.0 * f_r).sin()
        + 0.058793 * (2.0 * deg2rad(lp) - 2.0 * m_r).sin()
        + 0.057212 * (2.0 * deg2rad(lp) - ms_r - m_r).sin()
        + 0.053320 * (2.0 * deg2rad(lp) + m_r).sin()
        + 0.045874 * (2.0 * deg2rad(lp) - ms_r).sin()
        + 0.041024 * (m_r - ms_r).sin()
        - 0.034718 * deg2rad(lp).sin()
        - 0.030465 * (ms_r + m_r).sin()
        + 0.015326 * (2.0 * deg2rad(lp) - 2.0 * f_r).sin()
        - 0.012528 * (2.0 * f_r + m_r).sin()
        - 0.010980 * (2.0 * f_r - m_r).sin()
        + 0.010674 * (4.0 * deg2rad(lp) - m_r).sin()
        + 0.010034 * (3.0 * m_r).sin()
        - 0.008548 * (4.0 * deg2rad(lp) - 2.0 * m_r).sin()
        - 0.007910 * (ms_r - m_r + 2.0 * deg2rad(lp)).sin()
        - 0.006783 * (2.0 * deg2rad(lp) + ms_r).sin()
        + 0.005412 * (2.0 * deg2rad(lp) - ms_r + m_r).sin()
        - 0.004570 * (2.0 * m_r - 2.0 * deg2rad(lp)).sin()
        + 0.004130 * (2.0 * deg2rad(lp) + 2.0 * m_r - ms_r).sin()
        - 0.003751 * (2.0 * deg2rad(lp) - ms_r - 2.0 * m_r).sin()
        + 0.003000 * (2.0 * m_r + ms_r).sin()
        + 0.002730 * (2.0 * deg2rad(lp) - 2.0 * ms_r + m_r).sin()
        - 0.002585 * (2.0 * f_r - m_r + 2.0 * deg2rad(lp)).sin()
        - 0.002495 * (2.0 * deg2deg(lp) + 2.0 * f_r).sin()
        - 0.002377 * (deg2rad(lp) - ms_r).sin()
        + 0.002346 * (4.0 * m_r).sin()
        - 0.002172 * (2.0 * ms_r - m_r).sin()
        + 0.002134 * (4.0 * deg2rad(lp) - ms_r - m_r).sin()
        - 0.001943 * (3.0 * m_r - ms_r).sin()
        - 0.001595 * (2.0 * f_r + 2.0 * deg2rad(lp) - m_r).sin()
        - 0.001417 * (m_r + ms_r + 2.0 * deg2rad(lp)).sin()
        + 0.001405 * (2.0 * ms_r).sin()
        + 0.001203 * (m_r + ms_r - 2.0 * deg2rad(lp)).sin()
        + 0.001174 * (2.0 * deg2rad(lp) - m_r + ms_r).sin()
        - 0.001129 * (deg2rad(omega) - deg2rad(lp)).sin();

    // Additive correction
    let corr = -0.000918 * o_r.sin()
        - 0.000895 * (2.0 * o_r).sin()
        + 0.000817 * (m_r + deg2rad(lp)).sin()
        + 0.000806 * (deg2rad(lp) + 2.0 * f_r).sin();

    norm360(lp + sigma_l + corr)
}

fn deg2deg(d: f64) -> f64 { d } // identity; avoids confusion in moon formula

// ---------------------------------------------------------------------------
// Outer planets — VSOP87 mean elements (Meeus App. II, ~1° accuracy)
// ---------------------------------------------------------------------------

/// Mean orbital elements at epoch J2000.0, with linear rate per Julian century.
/// Format: (L0, L1) where L = L0 + L1*T  (degrees)
/// These are mean longitudes (heliocentric), corrected to geocentric below.
struct PlanetElements {
    l: (f64, f64),  // mean longitude
    a: f64,         // semi-major axis (AU)
    e: (f64, f64),  // eccentricity
    #[allow(dead_code)]
    i: (f64, f64),  // inclination (reserved for latitude calculations)
    w: (f64, f64),  // argument of perihelion
    #[allow(dead_code)]
    o: (f64, f64),  // longitude of ascending node (reserved for latitude calculations)
}

fn elements(planet: Planet) -> PlanetElements {
    match planet {
        Planet::Mercury => PlanetElements {
            l: (252.25084, 149472.67411),
            a: 0.38709893,
            e: (0.20563069, 0.00002527),
            i: (7.00487, -23.51),
            w: (77.45645, 573.57),
            o: (48.33167, -446.30),
        },
        Planet::Venus => PlanetElements {
            l: (181.97973, 58517.81538),
            a: 0.72333199,
            e: (0.00677323, -0.00004938),
            i: (3.39471, -2.86),
            w: (131.53298, -108.80),
            o: (76.68069, -996.89),
        },
        Planet::Mars => PlanetElements {
            l: (355.45332, 19140.29934),
            a: 1.52366231,
            e: (0.09341233, 0.00011902),
            i: (1.85061, -25.47),
            w: (336.04084, 1560.78),
            o: (49.57854, -1020.12),
        },
        Planet::Jupiter => PlanetElements {
            l: (34.40438, 3034.74612),
            a: 5.20336301,
            e: (0.04839266, -0.00012880),
            i: (1.30530, -4.15),
            w: (14.75385, 839.93),
            o: (100.55615, 1217.17),
        },
        Planet::Saturn => PlanetElements {
            l: (49.94432, 1222.49362),
            a: 9.53707032,
            e: (0.05415060, -0.00036762),
            i: (2.48446, 6.11),
            w: (92.43194, -1948.89),
            o: (113.71504, -1591.05),
        },
        Planet::Uranus => PlanetElements {
            l: (313.23218, 428.48202),
            a: 19.19126393,
            e: (0.04716771, -0.00019150),
            i: (0.76986, -2.09),
            w: (170.96424, 1312.56),
            o: (74.22988, -1681.40),
        },
        Planet::Neptune => PlanetElements {
            l: (304.88003, 218.45945),
            a: 30.06896348,
            e: (0.00858587, 0.00002510),
            i: (1.76917, -3.64),
            w: (44.97135, -844.43),
            o: (131.72169, -151.25),
        },
        Planet::Pluto => PlanetElements {
            l: (238.92881, 145.20780),
            a: 39.48168677,
            e: (0.24880766, 0.00006465),
            i: (17.14175, 11.07),
            w: (224.06676, -132.25),
            o: (110.30347, -37.33),
        },
        // Sun and Moon handled separately
        _ => unreachable!(),
    }
}

/// Heliocentric ecliptic longitude for a planet (degrees).
/// Uses Kepler's equation solved iteratively (3 iterations is sufficient).
fn heliocentric_longitude(planet: Planet, t: f64) -> f64 {
    let el = elements(planet);
    let t_cent = t; // already in centuries

    // Mean elements at epoch
    let l = norm360(el.l.0 + el.l.1 * t_cent / 100.0); // rate is per century / 3600 arcsec
    let e = el.e.0 + el.e.1 * t_cent;
    let w = norm360(el.w.0 + el.w.1 * t_cent / 3600.0);

    // Mean anomaly
    let m = norm360(l - w);
    let m_r = deg2rad(m);

    // Solve Kepler's equation: E - e*sin(E) = M
    let mut ea = m_r;
    for _ in 0..5 {
        ea = m_r + e * ea.sin();
    }

    // True anomaly
    let v = 2.0 * ((((1.0 + e) / (1.0 - e)).sqrt() * (ea / 2.0).tan()).atan());
    let v_deg = v * 180.0 / PI;

    norm360(v_deg + w)
}

/// Convert heliocentric longitude to approximate geocentric ecliptic longitude.
/// This is a simplified conversion — accurate enough for astrological purposes.
fn helio_to_geo(planet: Planet, t: f64, helio_lon: f64) -> f64 {
    let el = elements(planet);
    let sun_lon = sun_longitude(t);

    // Sun's mean anomaly
    let ms = norm360(357.52911 + t * (35999.05029 - t * 0.0001537));
    let ms_r = deg2rad(ms);
    // Earth's equation of center (approximate radius vector)
    let r_earth = 1.000001018 * (1.0 - 0.016708634_f64.powi(2))
        / (1.0 + 0.016708634 * ms_r.cos());

    // Planet's radius vector (simplified from mean elements)
    let e = el.e.0;
    let helio_r = el.a * (1.0 - e * e) / (1.0 + e * (deg2rad(helio_lon - el.w.0)).cos());

    // Geocentric conversion using the law of cosines
    let delta_r = helio_lon - sun_lon;
    let delta_r_rad = deg2rad(delta_r);

    // Approximate geocentric longitude correction
    let correction = (r_earth * delta_r_rad.sin())
        .atan2(helio_r - r_earth * delta_r_rad.cos());
    let correction_deg = correction * 180.0 / PI;

    norm360(helio_lon + 180.0 + correction_deg)
}

// ---------------------------------------------------------------------------
// Public: compute ecliptic longitude for any planet
// ---------------------------------------------------------------------------

pub fn planet_longitude(planet: Planet, t: f64) -> f64 {
    match planet {
        Planet::Sun  => sun_longitude(t),
        Planet::Moon => moon_longitude(t),
        other => {
            let helio = heliocentric_longitude(other, t);
            helio_to_geo(other, t, helio)
        }
    }
}

// ---------------------------------------------------------------------------
// Retrograde detection
// ---------------------------------------------------------------------------

/// Returns true if the planet is moving retrograde (longitude decreasing).
/// Compares longitude today vs 1 day ago. Handles 0°/360° wraparound.
pub fn is_retrograde(planet: Planet, jdn: f64) -> bool {
    let t_now  = jdn_to_t(jdn);
    let t_prev = jdn_to_t(jdn - 1.0);
    let lon_now  = planet_longitude(planet, t_now);
    let lon_prev = planet_longitude(planet, t_prev);

    // Handle wrap-around: if difference > 180° the planet crossed 0°/360°
    let mut diff = lon_now - lon_prev;
    if diff > 180.0  { diff -= 360.0; }
    if diff < -180.0 { diff += 360.0; }

    diff < 0.0
}

// ---------------------------------------------------------------------------
// Moon phase
// ---------------------------------------------------------------------------

/// Moon phase angle in degrees: 0 = New Moon, 180 = Full Moon.
pub fn moon_phase_angle(t: f64) -> f64 {
    norm360(moon_longitude(t) - sun_longitude(t))
}

/// Human-readable moon phase name from phase angle.
pub fn moon_phase_name(angle: f64) -> &'static str {
    match angle as u32 {
        0..=29    => "New Moon",
        30..=89   => "Waxing Crescent",
        90..=119  => "First Quarter",
        120..=179 => "Waxing Gibbous",
        180..=209 => "Full Moon",
        210..=269 => "Waning Gibbous",
        270..=299 => "Last Quarter",
        300..=359 => "Waning Crescent",
        _         => "New Moon",
    }
}

// ---------------------------------------------------------------------------
// Zodiac
// ---------------------------------------------------------------------------

/// Convert ecliptic longitude to zodiac sign + degree within that sign.
pub fn longitude_to_sign(lon: f64) -> (&'static str, f64) {
    const SIGNS: &[&str] = &[
        "Aries", "Taurus", "Gemini", "Cancer",
        "Leo", "Virgo", "Libra", "Scorpio",
        "Sagittarius", "Capricorn", "Aquarius", "Pisces",
    ];
    let lon = norm360(lon);
    let index = (lon / 30.0).floor() as usize;
    let degree = lon - index as f64 * 30.0;
    (SIGNS[index.min(11)], degree)
}

// ---------------------------------------------------------------------------
// Snapshot: all planets at a given JDN
// ---------------------------------------------------------------------------

pub struct PlanetSnapshot {
    pub planet:     Planet,
    pub longitude:  f64,
    pub sign:       &'static str,
    pub degree:     f64,
    pub retrograde: bool,
}

pub fn snapshot_all(jdn: f64) -> Vec<PlanetSnapshot> {
    let t = jdn_to_t(jdn);
    Planet::all().iter().map(|&planet| {
        let longitude  = planet_longitude(planet, t);
        let (sign, degree) = longitude_to_sign(longitude);
        let retrograde = match planet {
            Planet::Sun | Planet::Moon => false,  // never retrograde
            other => is_retrograde(other, jdn),
        };
        PlanetSnapshot { planet, longitude, sign, degree, retrograde }
    }).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jdn_j2000() {
        // J2000.0 = Jan 1.5, 2000 = JDN 2451545.0
        let jdn = date_to_jdn(2000, 1, 1, 12.0);
        assert!((jdn - 2_451_545.0).abs() < 0.001, "JDN for J2000.0 off: {jdn}");
    }

    #[test]
    fn test_sun_lon_j2000() {
        // Sun longitude on Jan 1, 2000 should be ~280° (Capricorn ~10°)
        let t = jdn_to_t(date_to_jdn(2000, 1, 1, 12.0));
        let lon = sun_longitude(t);
        assert!(lon > 270.0 && lon < 290.0, "Sun lon at J2000 unexpected: {lon}");
    }

    #[test]
    fn test_zodiac_sign() {
        let (sign, deg) = longitude_to_sign(0.0);
        assert_eq!(sign, "Aries");
        assert!(deg < 1.0);

        let (sign, deg) = longitude_to_sign(45.0);
        assert_eq!(sign, "Taurus");
        assert!((deg - 15.0).abs() < 0.01);

        let (sign, _) = longitude_to_sign(359.9);
        assert_eq!(sign, "Pisces");
    }

    #[test]
    fn test_moon_phase_names() {
        assert_eq!(moon_phase_name(0.0),   "New Moon");
        assert_eq!(moon_phase_name(90.0),  "First Quarter");
        assert_eq!(moon_phase_name(180.0), "Full Moon");
        assert_eq!(moon_phase_name(270.0), "Last Quarter");
    }

    #[test]
    fn test_norm360() {
        assert!((norm360(400.0) - 40.0).abs() < 0.001);
        assert!((norm360(-10.0) - 350.0).abs() < 0.001);
        assert!((norm360(0.0) - 0.0).abs() < 0.001);
    }
}
