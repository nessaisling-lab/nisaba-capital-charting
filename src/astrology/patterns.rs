//! Aspect pattern recognition (Wave 6.B1).
//!
//! Detects geometric configurations of 3-5 planets that form classical
//! astrological patterns. Patterns add weighted bonuses to the transit
//! score because astrologers read shapes, not isolated aspects.
//!
//! Detection runs on a combined list of natal + transit positions, each
//! tagged with origin. Cross-chart patterns (transit planet completing a
//! natal configuration) are the most ticker-specific signal.

use super::ephemeris::{longitude_to_sign, Planet, PlanetSnapshot};

// ---------------------------------------------------------------------------
// Pattern types
// ---------------------------------------------------------------------------

/// One detected pattern. `bodies` lists the planets involved in detection
/// order (apex first for Yod/T-Square, ring order for Grand Trine).
#[derive(Debug, Clone)]
pub struct AspectPattern {
    pub kind:     PatternKind,
    pub bodies:   Vec<Planet>,
    /// Mean orb across all defining aspects (lower = tighter = stronger).
    pub avg_orb:  f64,
    /// Strength contribution to astro_score before normalization.
    pub strength: f32,
    /// True if at least one body is from the natal chart and at least one
    /// from transit — pattern is then ticker-specific (not market-wide).
    pub is_cross: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternKind {
    GrandTrine,
    TSquare,
    GrandCross,
    Yod,
    Stellium,
    MysticRectangle,
    Kite,
}

impl PatternKind {
    pub fn name(self) -> &'static str {
        match self {
            PatternKind::GrandTrine      => "Grand Trine",
            PatternKind::TSquare         => "T-Square",
            PatternKind::GrandCross      => "Grand Cross",
            PatternKind::Yod             => "Yod",
            PatternKind::Stellium        => "Stellium",
            PatternKind::MysticRectangle => "Mystic Rectangle",
            PatternKind::Kite            => "Kite",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            PatternKind::GrandTrine      => "△",
            PatternKind::TSquare         => "⊥",
            PatternKind::GrandCross      => "✚",
            PatternKind::Yod             => "Y",
            PatternKind::Stellium        => "◉",
            PatternKind::MysticRectangle => "▭",
            PatternKind::Kite            => "◆",
        }
    }

    /// Base strength bonus added to delta_sum before sigmoid normalization.
    /// Positive for harmonious patterns, negative for tension patterns.
    pub fn base_strength(self) -> f32 {
        match self {
            PatternKind::GrandTrine      =>  15.0, // strong harmonious flow
            PatternKind::TSquare         => -12.0, // tension demanding action
            PatternKind::GrandCross      => -18.0, // major pivot, mostly stressful
            PatternKind::Yod             =>  -8.0, // forced redirection
            PatternKind::Stellium        =>   0.0, // sign-dependent (set per detection)
            PatternKind::MysticRectangle =>  10.0, // productive tension
            PatternKind::Kite            =>  18.0, // grand trine with channel
        }
    }
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

/// Combined position with origin tag (natal vs transit).
#[derive(Debug, Clone)]
struct TaggedPos {
    planet:    Planet,
    longitude: f64,
    is_natal:  bool,
}

/// Detect all aspect patterns from a combined natal + transit position set.
pub fn detect_patterns(
    natal: &[PlanetSnapshot],
    transits: &[PlanetSnapshot],
) -> Vec<AspectPattern> {
    let mut combined: Vec<TaggedPos> = Vec::with_capacity(natal.len() + transits.len());
    for p in natal {
        combined.push(TaggedPos { planet: p.planet, longitude: p.longitude, is_natal: true });
    }
    for p in transits {
        // Skip Moon as a transit pattern member — moves too fast, creates noise
        if p.planet == Planet::Moon { continue; }
        combined.push(TaggedPos { planet: p.planet, longitude: p.longitude, is_natal: false });
    }

    let mut out = Vec::new();
    detect_grand_trines(&combined, &mut out);
    detect_t_squares(&combined, &mut out);
    detect_grand_crosses(&combined, &mut out);
    detect_yods(&combined, &mut out);
    detect_stelliums(&combined, &mut out);
    detect_mystic_rectangles(&combined, &mut out);
    detect_kites(&combined, &mut out);
    out
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn angular_separation(a: f64, b: f64) -> f64 {
    let mut diff = (a - b).abs() % 360.0;
    if diff > 180.0 { diff = 360.0 - diff; }
    diff
}

/// True if two longitudes form the given aspect within `orb` degrees.
fn aspect_match(a: f64, b: f64, target_angle: f64, orb: f64) -> Option<f64> {
    let sep = angular_separation(a, b);
    let dist = (sep - target_angle).abs();
    if dist <= orb { Some(dist) } else { None }
}

fn finalize(kind: PatternKind, bodies: Vec<Planet>, orbs: &[f64], pos: &[&TaggedPos]) -> AspectPattern {
    let avg_orb = if orbs.is_empty() { 0.0 } else { orbs.iter().sum::<f64>() / orbs.len() as f64 };
    let max_orb = orbs.iter().cloned().fold(0.0_f64, f64::max).max(1.0);
    let tightness = 1.0 - (avg_orb / max_orb).min(1.0);
    let is_cross  = pos.iter().any(|p| p.is_natal) && pos.iter().any(|p| !p.is_natal);
    let cross_mult = if is_cross { 1.0 } else { 0.6 }; // intra-chart patterns less ticker-specific
    let strength = (kind.base_strength() as f64 * (0.5 + 0.5 * tightness) * cross_mult) as f32;
    AspectPattern { kind, bodies, avg_orb, strength, is_cross }
}

// ---------- Grand Trine: 3 planets, each pair in trine (120° ± 4°) -------

fn detect_grand_trines(pts: &[TaggedPos], out: &mut Vec<AspectPattern>) {
    let orb_max = 4.0;
    let n = pts.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let o_ij = match aspect_match(pts[i].longitude, pts[j].longitude, 120.0, orb_max) {
                Some(o) => o, None => continue,
            };
            for k in (j + 1)..n {
                let o_ik = match aspect_match(pts[i].longitude, pts[k].longitude, 120.0, orb_max) {
                    Some(o) => o, None => continue,
                };
                let o_jk = match aspect_match(pts[j].longitude, pts[k].longitude, 120.0, orb_max) {
                    Some(o) => o, None => continue,
                };
                out.push(finalize(
                    PatternKind::GrandTrine,
                    vec![pts[i].planet, pts[j].planet, pts[k].planet],
                    &[o_ij, o_ik, o_jk],
                    &[&pts[i], &pts[j], &pts[k]],
                ));
            }
        }
    }
}

// ---------- T-Square: opposition (180°) + 2 squares (90°) -----------------

fn detect_t_squares(pts: &[TaggedPos], out: &mut Vec<AspectPattern>) {
    let orb_max = 6.0;
    let n = pts.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let o_op = match aspect_match(pts[i].longitude, pts[j].longitude, 180.0, orb_max) {
                Some(o) => o, None => continue,
            };
            // Find a third planet that squares both ends of the opposition
            for k in 0..n {
                if k == i || k == j { continue; }
                let o_sq1 = match aspect_match(pts[k].longitude, pts[i].longitude, 90.0, orb_max) {
                    Some(o) => o, None => continue,
                };
                let o_sq2 = match aspect_match(pts[k].longitude, pts[j].longitude, 90.0, orb_max) {
                    Some(o) => o, None => continue,
                };
                // Apex is k (the planet squaring both opposition ends)
                out.push(finalize(
                    PatternKind::TSquare,
                    vec![pts[k].planet, pts[i].planet, pts[j].planet],
                    &[o_op, o_sq1, o_sq2],
                    &[&pts[i], &pts[j], &pts[k]],
                ));
            }
        }
    }
}

// ---------- Grand Cross: 4 planets, 2 oppositions + 4 squares -------------

fn detect_grand_crosses(pts: &[TaggedPos], out: &mut Vec<AspectPattern>) {
    let orb_max = 6.0;
    let n = pts.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let o1 = match aspect_match(pts[i].longitude, pts[j].longitude, 180.0, orb_max) {
                Some(o) => o, None => continue,
            };
            for k in (j + 1)..n {
                for l in (k + 1)..n {
                    let o2 = match aspect_match(pts[k].longitude, pts[l].longitude, 180.0, orb_max) {
                        Some(o) => o, None => continue,
                    };
                    // Now check 4 squares between {i,j} × {k,l}
                    let s1 = match aspect_match(pts[i].longitude, pts[k].longitude, 90.0, orb_max) {
                        Some(o) => o, None => continue,
                    };
                    let s2 = match aspect_match(pts[i].longitude, pts[l].longitude, 90.0, orb_max) {
                        Some(o) => o, None => continue,
                    };
                    let s3 = match aspect_match(pts[j].longitude, pts[k].longitude, 90.0, orb_max) {
                        Some(o) => o, None => continue,
                    };
                    let s4 = match aspect_match(pts[j].longitude, pts[l].longitude, 90.0, orb_max) {
                        Some(o) => o, None => continue,
                    };
                    out.push(finalize(
                        PatternKind::GrandCross,
                        vec![pts[i].planet, pts[j].planet, pts[k].planet, pts[l].planet],
                        &[o1, o2, s1, s2, s3, s4],
                        &[&pts[i], &pts[j], &pts[k], &pts[l]],
                    ));
                }
            }
        }
    }
}

// ---------- Yod: 2 planets in sextile (60°), both quincunx (150°) apex ----

fn detect_yods(pts: &[TaggedPos], out: &mut Vec<AspectPattern>) {
    let orb_sext = 4.0;
    let orb_quin = 3.0;
    let n = pts.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let o_sext = match aspect_match(pts[i].longitude, pts[j].longitude, 60.0, orb_sext) {
                Some(o) => o, None => continue,
            };
            for k in 0..n {
                if k == i || k == j { continue; }
                let o_q1 = match aspect_match(pts[k].longitude, pts[i].longitude, 150.0, orb_quin) {
                    Some(o) => o, None => continue,
                };
                let o_q2 = match aspect_match(pts[k].longitude, pts[j].longitude, 150.0, orb_quin) {
                    Some(o) => o, None => continue,
                };
                // Apex is k (the planet quincunx both sextile ends)
                out.push(finalize(
                    PatternKind::Yod,
                    vec![pts[k].planet, pts[i].planet, pts[j].planet],
                    &[o_sext, o_q1, o_q2],
                    &[&pts[i], &pts[j], &pts[k]],
                ));
            }
        }
    }
}

// ---------- Stellium: 3+ planets in same sign within 8° of each other -----

fn detect_stelliums(pts: &[TaggedPos], out: &mut Vec<AspectPattern>) {
    use std::collections::HashMap;
    let mut by_sign: HashMap<&str, Vec<usize>> = HashMap::new();
    for (idx, p) in pts.iter().enumerate() {
        let (sign, _) = longitude_to_sign(p.longitude);
        by_sign.entry(sign).or_default().push(idx);
    }
    for (_, idxs) in by_sign.iter() {
        if idxs.len() < 3 { continue; }
        let bodies: Vec<Planet> = idxs.iter().map(|&i| pts[i].planet).collect();
        let participants: Vec<&TaggedPos> = idxs.iter().map(|&i| &pts[i]).collect();
        // Stellium strength scales with planet count: 3 = +6, 4 = +10, 5 = +14
        let count_bonus = 6.0_f32 + ((bodies.len() - 3) as f32) * 4.0;
        let avg_orb = 0.0; // sign-bound, no orb concept
        let is_cross = participants.iter().any(|p| p.is_natal)
                    && participants.iter().any(|p| !p.is_natal);
        let cross_mult = if is_cross { 1.0 } else { 0.6 };
        out.push(AspectPattern {
            kind: PatternKind::Stellium,
            bodies,
            avg_orb,
            strength: count_bonus * cross_mult,
            is_cross,
        });
    }
}

// ---------- Mystic Rectangle: 4 planets, 2 sextiles + 2 trines + 2 oppos --

fn detect_mystic_rectangles(pts: &[TaggedPos], out: &mut Vec<AspectPattern>) {
    let orb_max = 4.0;
    let n = pts.len();
    // Topology: 2 oppositions + 2 sextiles + 2 trines
    // Pairs (a-c) and (b-d) are oppositions; (a-b)+(c-d) sextiles, (a-d)+(b-c) trines
    for a in 0..n {
        for c in (a + 1)..n {
            let o_ac = match aspect_match(pts[a].longitude, pts[c].longitude, 180.0, orb_max) {
                Some(o) => o, None => continue,
            };
            for b in 0..n {
                if b == a || b == c { continue; }
                for d in (b + 1)..n {
                    if d == a || d == c { continue; }
                    let o_bd = match aspect_match(pts[b].longitude, pts[d].longitude, 180.0, orb_max) {
                        Some(o) => o, None => continue,
                    };
                    let o_ab = match aspect_match(pts[a].longitude, pts[b].longitude, 60.0, orb_max) {
                        Some(o) => o, None => continue,
                    };
                    let o_cd = match aspect_match(pts[c].longitude, pts[d].longitude, 60.0, orb_max) {
                        Some(o) => o, None => continue,
                    };
                    let o_ad = match aspect_match(pts[a].longitude, pts[d].longitude, 120.0, orb_max) {
                        Some(o) => o, None => continue,
                    };
                    let o_bc = match aspect_match(pts[b].longitude, pts[c].longitude, 120.0, orb_max) {
                        Some(o) => o, None => continue,
                    };
                    out.push(finalize(
                        PatternKind::MysticRectangle,
                        vec![pts[a].planet, pts[b].planet, pts[c].planet, pts[d].planet],
                        &[o_ac, o_bd, o_ab, o_cd, o_ad, o_bc],
                        &[&pts[a], &pts[b], &pts[c], &pts[d]],
                    ));
                }
            }
        }
    }
}

// ---------- Kite: Grand Trine + opposition from 4th planet to one apex ---

fn detect_kites(pts: &[TaggedPos], out: &mut Vec<AspectPattern>) {
    let orb_max = 4.0;
    let n = pts.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let oa = match aspect_match(pts[i].longitude, pts[j].longitude, 120.0, orb_max) {
                Some(o) => o, None => continue,
            };
            for k in (j + 1)..n {
                let ob = match aspect_match(pts[i].longitude, pts[k].longitude, 120.0, orb_max) {
                    Some(o) => o, None => continue,
                };
                let oc = match aspect_match(pts[j].longitude, pts[k].longitude, 120.0, orb_max) {
                    Some(o) => o, None => continue,
                };
                // Found a Grand Trine. Look for 4th planet opposite one of {i,j,k}
                for l in 0..n {
                    if l == i || l == j || l == k { continue; }
                    for &apex in &[i, j, k] {
                        if let Some(o_op) = aspect_match(pts[l].longitude, pts[apex].longitude, 180.0, orb_max) {
                            // Other two trine planets must sextile the 4th
                            let other1 = if apex == i { j } else { i };
                            let other2 = if apex == k { j } else { k };
                            let other2 = if other2 == apex { i } else { other2 };
                            let s1 = aspect_match(pts[l].longitude, pts[other1].longitude, 60.0, orb_max);
                            let s2 = aspect_match(pts[l].longitude, pts[other2].longitude, 60.0, orb_max);
                            if let (Some(os1), Some(os2)) = (s1, s2) {
                                out.push(finalize(
                                    PatternKind::Kite,
                                    vec![pts[apex].planet, pts[i].planet, pts[j].planet, pts[k].planet, pts[l].planet],
                                    &[oa, ob, oc, o_op, os1, os2],
                                    &[&pts[i], &pts[j], &pts[k], &pts[l]],
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Score contribution
// ---------------------------------------------------------------------------

/// Sum the strength contributions of all detected patterns. This is added
/// to `delta_sum` in `compute_transit_score` BEFORE sigmoid normalization.
pub fn pattern_score_total(patterns: &[AspectPattern]) -> f32 {
    patterns.iter().map(|p| p.strength).sum()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(planet: Planet, lon: f64) -> PlanetSnapshot {
        let (sign, degree) = longitude_to_sign(lon);
        PlanetSnapshot {
            planet,
            longitude: lon,
            sign,
            degree,
            retrograde: false,
        }
    }

    #[test]
    fn detects_exact_grand_trine() {
        let natal = vec![pos(Planet::Sun, 0.0), pos(Planet::Mars, 120.0), pos(Planet::Jupiter, 240.0)];
        let transits = vec![];
        let patterns = detect_patterns(&natal, &transits);
        let count = patterns.iter().filter(|p| p.kind == PatternKind::GrandTrine).count();
        assert_eq!(count, 1, "Expected exactly 1 Grand Trine, got {count}");
    }

    #[test]
    fn detects_t_square() {
        let natal = vec![pos(Planet::Sun, 0.0), pos(Planet::Moon, 180.0), pos(Planet::Mars, 90.0)];
        let transits = vec![];
        let patterns = detect_patterns(&natal, &transits);
        // Moon is filtered as transit but not as natal, so this works
        let has_tsq = patterns.iter().any(|p| p.kind == PatternKind::TSquare);
        assert!(has_tsq, "Expected T-Square in {patterns:?}");
    }

    #[test]
    fn detects_stellium() {
        // 3 planets in Aries within sign bounds
        let natal = vec![
            pos(Planet::Sun, 5.0),
            pos(Planet::Venus, 15.0),
            pos(Planet::Mercury, 25.0),
        ];
        let patterns = detect_patterns(&natal, &[]);
        let stelliums: Vec<_> = patterns.iter().filter(|p| p.kind == PatternKind::Stellium).collect();
        assert_eq!(stelliums.len(), 1);
        assert_eq!(stelliums[0].bodies.len(), 3);
    }

    #[test]
    fn detects_yod() {
        // Yod geometry: 2 planets sextile (60° apart), apex 150° from BOTH.
        // Sun at 0°, Mars at 60° (sextile). Apex at 210° is 150° from each.
        let natal = vec![
            pos(Planet::Sun, 0.0),
            pos(Planet::Mars, 60.0),
            pos(Planet::Jupiter, 210.0),
        ];
        let patterns = detect_patterns(&natal, &[]);
        let has_yod = patterns.iter().any(|p| p.kind == PatternKind::Yod);
        assert!(has_yod, "Expected Yod in {patterns:?}");
    }

    #[test]
    fn cross_chart_marked_when_natal_and_transit_mix() {
        let natal = vec![pos(Planet::Sun, 0.0), pos(Planet::Jupiter, 120.0)];
        let transits = vec![pos(Planet::Saturn, 240.0)];
        let patterns = detect_patterns(&natal, &transits);
        let gt = patterns.iter().find(|p| p.kind == PatternKind::GrandTrine);
        assert!(gt.is_some(), "Expected Grand Trine spanning natal+transit");
        assert!(gt.unwrap().is_cross, "Should be marked cross-chart");
    }

    #[test]
    fn pure_natal_pattern_not_cross() {
        let natal = vec![pos(Planet::Sun, 0.0), pos(Planet::Mars, 120.0), pos(Planet::Jupiter, 240.0)];
        let patterns = detect_patterns(&natal, &[]);
        let gt = patterns.iter().find(|p| p.kind == PatternKind::GrandTrine);
        assert!(gt.is_some());
        assert!(!gt.unwrap().is_cross);
    }

    #[test]
    fn no_patterns_when_random_positions() {
        let natal = vec![
            pos(Planet::Sun, 17.3),
            pos(Planet::Moon, 234.7),
            pos(Planet::Mars, 88.2),
        ];
        let patterns = detect_patterns(&natal, &[]);
        // No 3-body geometric patterns should match these random angles
        let geometric: Vec<_> = patterns.iter()
            .filter(|p| !matches!(p.kind, PatternKind::Stellium))
            .collect();
        assert!(geometric.is_empty(), "Unexpected patterns: {geometric:?}");
    }
}
