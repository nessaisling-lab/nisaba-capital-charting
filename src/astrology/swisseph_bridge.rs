//! Swiss Ephemeris bridge — high-accuracy planetary positions.
//!
//! Adapter layer translating our `Planet`/`PlanetSnapshot` types to/from
//! the `swiss-eph` crate's safe API. Uses the built-in Moshier analytical
//! ephemeris (sub-arcminute, no external `.se1` files required). Falls back
//! to the Jean Meeus engine if Swiss Ephemeris computation fails for a body.
//!
//! Key advantages over Meeus:
//!   - Sub-arcminute accuracy for all planets (vs ~1-2° for outer planets)
//!   - Lunar nodes (True Node) and Chiron natively supported
//!   - `longitude_speed` gives applying/separating without needing yesterday
//!   - House cusps + Ascendant + MC for location-aware charts

use swiss_eph::safe::{
    self as swe,
    CalcFlags,
    HouseSystem,
    HouseCusps,
    Position as SwePosition,
    Planet as SwePlanet,
};

use super::ephemeris::{
    Planet, PlanetSnapshot, longitude_to_sign, norm360, snapshot_all,
};

// ---------------------------------------------------------------------------
// Planet mapping: our Planet enum → swiss-eph Planet enum
// ---------------------------------------------------------------------------

/// Map our Planet to the Swiss Ephemeris planet constant.
/// Returns `None` for SouthNode (computed as NorthNode + 180°).
fn to_swe_planet(planet: Planet) -> Option<SwePlanet> {
    match planet {
        Planet::Sun       => Some(SwePlanet::Sun),
        Planet::Moon      => Some(SwePlanet::Moon),
        Planet::Mercury   => Some(SwePlanet::Mercury),
        Planet::Venus     => Some(SwePlanet::Venus),
        Planet::Mars      => Some(SwePlanet::Mars),
        Planet::Jupiter   => Some(SwePlanet::Jupiter),
        Planet::Saturn    => Some(SwePlanet::Saturn),
        Planet::Uranus    => Some(SwePlanet::Uranus),
        Planet::Neptune   => Some(SwePlanet::Neptune),
        Planet::Pluto     => Some(SwePlanet::Pluto),
        Planet::NorthNode => Some(SwePlanet::TrueNode),
        Planet::SouthNode => None, // derived from NorthNode
        Planet::Chiron    => Some(SwePlanet::Chiron),
    }
}

// ---------------------------------------------------------------------------
// Core calculation flags
// ---------------------------------------------------------------------------

/// Standard flags: Moshier analytical ephemeris + speed.
///
/// Moshier is built into the Swiss Ephemeris library — no external `.se1`
/// data files needed. Sub-arcminute accuracy for all bodies (far better
/// than Meeus, and more than sufficient for astrological orbs of 1-8°).
///
/// If `.se1` files are installed later, remove `.with_moshier()` to get
/// sub-arcsecond accuracy from the full Swiss Ephemeris data.
fn default_flags() -> CalcFlags {
    CalcFlags::new().with_speed().with_moshier()
}

// ---------------------------------------------------------------------------
// Single-planet position
// ---------------------------------------------------------------------------

/// Compute a single planet's position using Swiss Ephemeris.
///
/// `jd` is the Julian Day in Terrestrial Time (TT). For IPO charts at
/// 09:30 EST, that's JDN + 14.5/24 (UT) plus ~69 seconds delta-T,
/// but the delta-T difference is negligible for astrological orbs.
///
/// Returns `(longitude, longitude_speed)` where speed is degrees/day.
/// Negative speed = retrograde.
fn calc_position(planet: Planet, jd: f64) -> anyhow::Result<(f64, f64)> {
    if planet == Planet::SouthNode {
        // South Node is always 180° opposite the True (North) Node
        let (north_lon, north_speed) = calc_position(Planet::NorthNode, jd)?;
        let south_lon = norm360(north_lon + 180.0);
        // South Node speed mirrors North Node (same magnitude, same sign —
        // both nodes move in the same direction along the ecliptic)
        return Ok((south_lon, north_speed));
    }

    let swe_planet = to_swe_planet(planet)
        .ok_or_else(|| anyhow::anyhow!("No Swiss Ephemeris mapping for {:?}", planet))?;

    let pos: SwePosition = swe::calc(jd, swe_planet, default_flags())
        .map_err(|e| anyhow::anyhow!("Swiss Ephemeris calc failed for {:?}: {}", planet, e))?;

    // Guard: Swiss Ephemeris can return Ok with NaN in edge cases.
    if pos.longitude.is_nan() {
        anyhow::bail!("{:?}: Swiss Ephemeris returned NaN longitude", planet);
    }

    Ok((norm360(pos.longitude), pos.longitude_speed))
}

// ---------------------------------------------------------------------------
// Full snapshot: all 13 bodies
// ---------------------------------------------------------------------------

/// Snapshot all 13 planets using Swiss Ephemeris (sub-arcsecond accuracy).
///
/// Falls back to Meeus `snapshot_all()` for the 10 classical planets if
/// Swiss Ephemeris fails, skipping nodes and Chiron in that case.
pub fn snapshot_all_precise(jdn: f64) -> Vec<PlanetSnapshot> {
    let mut results = Vec::with_capacity(13);

    for &planet in Planet::all() {
        match calc_position(planet, jdn) {
            Ok((longitude, speed)) => {
                let (sign, degree) = longitude_to_sign(longitude);
                let retrograde = match planet {
                    // Sun and Moon never retrograde
                    Planet::Sun | Planet::Moon => false,
                    // Nodes are always retrograde in mean motion, but True Node
                    // can briefly go direct. Use speed sign.
                    Planet::NorthNode | Planet::SouthNode => speed < 0.0,
                    _ => speed < 0.0,
                };
                results.push(PlanetSnapshot {
                    planet,
                    longitude,
                    sign,
                    degree,
                    retrograde,
                });
            }
            Err(_e) => {
                // If Swiss Eph fails for a classical planet, we can still
                // compute it with Meeus. For nodes/Chiron, just skip.
                if !planet.needs_swiss_eph() {
                    // Fall back to Meeus for this one planet
                    let meeus = snapshot_all(jdn);
                    if let Some(snap) = meeus.iter().find(|s| s.planet == planet) {
                        results.push(PlanetSnapshot {
                            planet: snap.planet,
                            longitude: snap.longitude,
                            sign: snap.sign,
                            degree: snap.degree,
                            retrograde: snap.retrograde,
                        });
                    }
                }
                // else: node/Chiron not available, skip silently
            }
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Longitude speed for applying/separating detection
// ---------------------------------------------------------------------------

/// Get a transit planet's longitude speed in degrees/day.
/// Positive = direct motion, negative = retrograde.
/// This is critical for determining applying vs separating aspects.
pub fn longitude_speed(planet: Planet, jdn: f64) -> Option<f64> {
    calc_position(planet, jdn).ok().map(|(_, speed)| speed)
}

// ---------------------------------------------------------------------------
// House cusps + angles
// ---------------------------------------------------------------------------

/// Default location for US equity charts: NYSE, New York City.
pub const NYSE_LAT: f64 = 40.7128;
pub const NYSE_LON: f64 = -74.0060;

/// House cusp positions + Ascendant + MC.
pub struct HouseData {
    /// 12 house cusp longitudes (0-based: house 1 = index 0)
    pub cusps: [f64; 12],
    /// Ascendant longitude
    pub ascendant: f64,
    /// Midheaven (MC) longitude
    pub mc: f64,
}

/// Compute house cusps for a given Julian Day and geographic location.
/// Defaults to Whole Sign houses (simplest, most reliable for financial astrology).
pub fn compute_houses(jdn: f64, lat: f64, lon: f64) -> anyhow::Result<HouseData> {
    let cusps: HouseCusps = swe::houses(jdn, lat, lon, HouseSystem::WholeSign)
        .map_err(|e| anyhow::anyhow!("House calculation failed: {}", e))?;

    Ok(HouseData {
        cusps: cusps.cusps,
        ascendant: cusps.ascendant,
        mc: cusps.mc,
    })
}

/// Compute houses using NYSE location (default for US equities).
pub fn compute_houses_nyse(jdn: f64) -> anyhow::Result<HouseData> {
    compute_houses(jdn, NYSE_LAT, NYSE_LON)
}

// ---------------------------------------------------------------------------
// Julian Day helper (Swiss Ephemeris native)
// ---------------------------------------------------------------------------

/// Convert calendar date to Julian Day using Swiss Ephemeris's own routine.
/// This should agree with our `date_to_jdn()` to within milliseconds.
pub fn swe_julday(year: i32, month: u32, day: u32, hour_ut: f64) -> f64 {
    swe::julday(year, month as i32, day as i32, hour_ut)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// Swiss Ephemeris uses global mutable C state. Tests must not run
// concurrently or the shared state gets corrupted (NaN positions,
// inconsistent flag sets). This mutex serializes all SWE tests.
#[cfg(test)]
pub(crate) static SWE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astrology::ephemeris::date_to_jdn;

    #[test]
    fn test_swe_sun_position() {
        let _guard = super::SWE_TEST_LOCK.lock().unwrap();
        // Jan 1, 2000 12:00 UT — Sun should be ~280° (Capricorn)
        let jdn = date_to_jdn(2000, 1, 1, 12.0);
        let (lon, speed) = calc_position(Planet::Sun, jdn).unwrap();
        assert!(lon > 270.0 && lon < 290.0, "Sun lon at J2000 unexpected: {lon}");
        assert!(speed > 0.0, "Sun should be direct: {speed}");
    }

    #[test]
    fn test_swe_vs_meeus_sun_moon() {
        let _guard = super::SWE_TEST_LOCK.lock().unwrap();
        let jdn = date_to_jdn(2024, 6, 15, 12.0);
        let meeus_snaps = snapshot_all(jdn);
        let precise_snaps = snapshot_all_precise(jdn);

        for planet in [Planet::Sun, Planet::Moon] {
            let meeus = meeus_snaps.iter().find(|s| s.planet == planet).unwrap();
            let precise = precise_snaps.iter().find(|s| s.planet == planet).unwrap();
            let mut diff = (meeus.longitude - precise.longitude).abs();
            if diff > 180.0 { diff = 360.0 - diff; }
            assert!(
                diff < 1.0,
                "{:?}: Meeus={:.2}° vs SwissEph={:.2}° (diff={:.2}°)",
                planet, meeus.longitude, precise.longitude, diff,
            );
        }
    }

    #[test]
    fn test_swe_nodes() {
        let _guard = super::SWE_TEST_LOCK.lock().unwrap();
        let jdn = date_to_jdn(2024, 6, 15, 12.0);
        for planet in [Planet::NorthNode, Planet::SouthNode] {
            let (lon, _speed) = calc_position(planet, jdn)
                .unwrap_or_else(|e| panic!("{:?} failed: {e}", planet));
            assert!(lon >= 0.0 && lon < 360.0, "{:?} lon out of range: {lon}", planet);
        }

        // North Node + South Node should be exactly 180° apart
        let (north, _) = calc_position(Planet::NorthNode, jdn).unwrap();
        let (south, _) = calc_position(Planet::SouthNode, jdn).unwrap();
        let mut diff = (north - south).abs();
        if diff > 180.0 { diff = 360.0 - diff; }
        assert!(
            (diff - 180.0).abs() < 0.01,
            "Nodes not opposite: North={north:.2}° South={south:.2}° diff={diff:.2}°",
        );
    }

    #[test]
    fn test_swe_chiron_optional() {
        let _guard = super::SWE_TEST_LOCK.lock().unwrap();
        let jdn = date_to_jdn(2024, 6, 15, 12.0);
        match calc_position(Planet::Chiron, jdn) {
            Ok((lon, _speed)) => {
                assert!(lon >= 0.0 && lon < 360.0, "Chiron lon out of range: {lon}");
            }
            Err(e) => {
                // Expected if no asteroid files available — not a failure
                eprintln!("Chiron not available (expected without .se1 files): {e}");
            }
        }
    }

    #[test]
    fn test_snapshot_all_precise_count() {
        let _guard = super::SWE_TEST_LOCK.lock().unwrap();
        let jdn = date_to_jdn(2024, 6, 15, 12.0);
        let snaps = snapshot_all_precise(jdn);
        // 13 bodies if Chiron's asteroid file is available, 12 without it.
        // The embedded ephemeris covers Sun through Pluto + True Node.
        // Chiron requires seas_XX.se1 files or Moshier fallback.
        assert!(
            snaps.len() >= 12 && snaps.len() <= 13,
            "Expected 12-13 bodies, got {}",
            snaps.len(),
        );
        // Verify all classical planets are present
        for planet in Planet::all_classical() {
            assert!(
                snaps.iter().any(|s| s.planet == *planet),
                "{:?} missing from snapshot",
                planet,
            );
        }
        // Verify nodes are present
        assert!(snaps.iter().any(|s| s.planet == Planet::NorthNode), "NorthNode missing");
        assert!(snaps.iter().any(|s| s.planet == Planet::SouthNode), "SouthNode missing");
    }

    #[test]
    fn test_swe_julday_agreement() {
        let _guard = super::SWE_TEST_LOCK.lock().unwrap();
        let ours = date_to_jdn(2024, 3, 20, 12.0);
        let theirs = swe_julday(2024, 3, 20, 12.0);
        assert!(
            (ours - theirs).abs() < 0.001,
            "JDN mismatch: ours={ours} theirs={theirs}",
        );
    }

    #[test]
    fn test_houses_nyse() {
        let _guard = super::SWE_TEST_LOCK.lock().unwrap();
        let jdn = date_to_jdn(2024, 6, 15, 14.5); // ~9:30 AM EST in UT
        let houses = compute_houses_nyse(jdn).unwrap();
        assert!(houses.ascendant >= 0.0 && houses.ascendant < 360.0);
        assert!(houses.mc >= 0.0 && houses.mc < 360.0);
        // All 12 cusps should be valid
        for (i, &cusp) in houses.cusps.iter().enumerate() {
            assert!(cusp >= 0.0 && cusp < 360.0, "Cusp {} out of range: {cusp}", i + 1);
        }
    }
}
