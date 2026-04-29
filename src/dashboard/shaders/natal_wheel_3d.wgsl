// 3D Natal Chart shader (v8.0) — "The Observatory"
// Procedural SDF rendering of a perspective-tilted zodiac wheel with
// glowing planets, animated transit drift, and luminous aspect lines.
//
// Same full-screen-triangle approach as vignette.wgsl — no vertex buffer,
// all rendering via signed distance functions in the fragment shader.

// ── Uniform buffer (matches NatalWheel3DUniforms in mod.rs, 496 bytes) ─

struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    camera_tilt: f32,
    bg_color: vec4<f32>,
    gold_color: vec4<f32>,
    transit_color: vec4<f32>,
    natal_planets: array<vec4<f32>, 13>,
    transit_planets: array<vec4<f32>, 13>,
    natal_count: f32,
    transit_count: f32,
    retro_r: f32,
    retro_g: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// ── Vertex shader: full-screen triangle from vertex_index ──────────

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vid & 1u) * 4 - 1);
    let y = f32(i32(vid >> 1u) * 4 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// ── Constants ──────────────────────────────────────────────────────

const PI: f32 = 3.14159265;
const TAU: f32 = 6.28318530;

// Radii (normalized to [-1,1] unit space)
const R_OUTER: f32   = 0.92;    // zodiac ring outer edge
const R_NATAL: f32   = 0.644;   // natal planet track  (0.70 × 0.92)
const R_TRANSIT: f32 = 0.810;   // transit planet track (0.88 × 0.92)
const R_CENTER: f32  = 0.478;   // inner circle         (0.52 × 0.92)
const RING_W: f32    = 0.005;   // ring stroke half-width
const PLANET_R: f32  = 0.016;   // planet dot radius
const HALO_R: f32    = 0.045;   // glow halo outer radius
const ASPECT_W: f32  = 0.003;   // aspect line half-width

// ── Sign color lookup (element-based — fire/earth/air/water) ──────

fn sign_color(idx: u32) -> vec4<f32> {
    switch idx {
        case 0u:  { return vec4<f32>(0.95, 0.30, 0.20, 0.35); } // Aries       — fire
        case 1u:  { return vec4<f32>(0.45, 0.75, 0.30, 0.35); } // Taurus      — earth
        case 2u:  { return vec4<f32>(0.90, 0.85, 0.30, 0.35); } // Gemini      — air
        case 3u:  { return vec4<f32>(0.30, 0.60, 0.95, 0.35); } // Cancer      — water
        case 4u:  { return vec4<f32>(0.95, 0.55, 0.15, 0.35); } // Leo         — fire
        case 5u:  { return vec4<f32>(0.55, 0.80, 0.40, 0.35); } // Virgo       — earth
        case 6u:  { return vec4<f32>(0.85, 0.75, 0.40, 0.35); } // Libra       — air
        case 7u:  { return vec4<f32>(0.70, 0.25, 0.30, 0.35); } // Scorpio     — water
        case 8u:  { return vec4<f32>(0.80, 0.40, 0.90, 0.35); } // Sagittarius — fire
        case 9u:  { return vec4<f32>(0.40, 0.55, 0.45, 0.35); } // Capricorn   — earth
        case 10u: { return vec4<f32>(0.35, 0.70, 0.90, 0.35); } // Aquarius    — air
        case 11u: { return vec4<f32>(0.50, 0.40, 0.80, 0.35); } // Pisces      — water
        default:  { return vec4<f32>(0.50, 0.50, 0.50, 0.20); }
    }
}

// ── SDF helpers ────────────────────────────────────────────────────

// Distance to an annular ring stroke
fn sdf_ring(p: vec2<f32>, radius: f32, width: f32) -> f32 {
    return abs(length(p) - radius) - width;
}

// Distance to a line segment a→b
fn sdf_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let t = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * t);
}

// Distance to a filled circle
fn sdf_dot(p: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    return length(p - center) - radius;
}

// Ecliptic longitude (degrees) → canvas angle (radians)
// Matches astrology.rs lon_to_angle: -(lon × π / 180)
fn lon_to_angle(lon: f32) -> f32 {
    return -lon * PI / 180.0;
}

// Position on a ring at given angle
fn ring_pos(angle: f32, radius: f32) -> vec2<f32> {
    return vec2<f32>(cos(angle), sin(angle)) * radius;
}

// Hash for star field
fn hash(n: f32) -> f32 {
    return fract(sin(n * 43758.5453) * 2.0);
}

// ── Fragment shader ────────────────────────────────────────────────

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Screen UV → centered [-1, 1]
    let raw = (in.uv - 0.5) * 2.0;

    // ── Inverse perspective: screen → chart space ─────────────────
    // Undo Y foreshortening (expand Y back to circle)
    let inv_scale = 1.0 / (1.0 - u.camera_tilt);
    let expanded = vec2<f32>(raw.x, raw.y * inv_scale);

    // Undo slow rotation (chart spins inside the ellipse)
    let rot = -u.time * 0.015;
    let cr = cos(rot);
    let sr = sin(rot);
    let pc = vec2<f32>(expanded.x * cr - expanded.y * sr,
                       expanded.x * sr + expanded.y * cr);

    // Chart-space polar coords
    let r = length(pc);
    let pixel_w = 2.5 / min(u.resolution.x, u.resolution.y);

    // Background
    var color = u.bg_color.rgb;

    // ── 1. Zodiac ring segments (12 colored arcs) ─────────────────
    if r > R_NATAL - pixel_w && r < R_OUTER + pixel_w {
        // Chart angle → ecliptic longitude [0, 360)
        var lon = -atan2(pc.y, pc.x) * 180.0 / PI;
        if lon < 0.0 { lon += 360.0; }
        let idx = u32(lon / 30.0) % 12u;
        let sc = sign_color(idx);

        // Anti-aliased edges
        let inner_mask = smoothstep(R_NATAL - pixel_w, R_NATAL + pixel_w, r);
        let outer_mask = smoothstep(R_OUTER + pixel_w, R_OUTER - pixel_w, r);
        let mask = inner_mask * outer_mask;

        color = mix(color, sc.rgb, sc.a * mask);
    }

    // ── 2. Ring strokes ───────────────────────────────────────────
    let dim = vec3<f32>(0.6, 0.55, 0.4); // warm dim gold

    // Outer ring
    let d_outer = sdf_ring(pc, R_OUTER, RING_W);
    color = mix(color, dim, (1.0 - smoothstep(0.0, pixel_w * 2.0, d_outer)) * 0.55);

    // Natal track
    let d_nat_ring = sdf_ring(pc, R_NATAL, RING_W * 0.7);
    color = mix(color, dim, (1.0 - smoothstep(0.0, pixel_w * 2.0, d_nat_ring)) * 0.35);

    // Transit track (faint)
    let d_tra_ring = sdf_ring(pc, R_TRANSIT, RING_W * 0.5);
    color = mix(color, dim * 0.8, (1.0 - smoothstep(0.0, pixel_w * 2.0, d_tra_ring)) * 0.20);

    // Inner circle
    let d_inner = sdf_ring(pc, R_CENTER, RING_W * 0.7);
    color = mix(color, dim, (1.0 - smoothstep(0.0, pixel_w * 2.0, d_inner)) * 0.35);

    // ── 3. Sign divider lines (12 radial lines) ──────────────────
    for (var i = 0u; i < 12u; i = i + 1u) {
        let dv_a = lon_to_angle(f32(i) * 30.0);
        let a = ring_pos(dv_a, R_CENTER);
        let b = ring_pos(dv_a, R_OUTER);
        let d = sdf_segment(pc, a, b);
        let da = 1.0 - smoothstep(0.0, pixel_w * 2.0, d - RING_W * 0.4);
        color = mix(color, dim, da * 0.22);
    }

    // ── 4. Aspect lines (natal × transit) ─────────────────────────
    let nc = u32(u.natal_count);
    let tc = u32(u.transit_count);
    let drift = u.time * 0.5 * PI / 180.0; // 0.5°/sec transit drift

    for (var i = 0u; i < nc; i = i + 1u) {
        let n_lon = u.natal_planets[i].x;
        let n_pos = ring_pos(lon_to_angle(n_lon), R_CENTER * 0.92);

        for (var j = 0u; j < tc; j = j + 1u) {
            let t_lon = u.transit_planets[j].x;
            let t_pos = ring_pos(lon_to_angle(t_lon) + drift, R_CENTER * 0.92);

            // Angular difference
            var diff = abs(n_lon - t_lon);
            diff = diff % 360.0;
            if diff > 180.0 { diff = 360.0 - diff; }

            var asp_color = vec3<f32>(0.0);
            var asp_alpha = 0.0;
            var asp_w = ASPECT_W;

            if diff < 8.0 || diff > 352.0 {
                // Conjunction — thick gold
                asp_color = vec3<f32>(1.0, 0.9, 0.3);
                asp_alpha = 0.20;
                asp_w = ASPECT_W * 1.5;
            } else if abs(diff - 60.0) < 6.0 {
                // Sextile — green
                asp_color = vec3<f32>(0.3, 1.0, 0.5);
                asp_alpha = 0.14;
            } else if abs(diff - 90.0) < 8.0 {
                // Square — red
                asp_color = vec3<f32>(1.0, 0.3, 0.3);
                asp_alpha = 0.16;
            } else if abs(diff - 120.0) < 8.0 {
                // Trine — blue
                asp_color = vec3<f32>(0.3, 0.7, 1.0);
                asp_alpha = 0.20;
            }

            if asp_alpha > 0.0 {
                let d = sdf_segment(pc, n_pos, t_pos);
                let la = 1.0 - smoothstep(0.0, pixel_w * 3.0, d - asp_w);
                color = mix(color, asp_color, la * asp_alpha);
            }
        }
    }

    // ── 5. Natal planets (gold dots with glow halos) ──────────────
    for (var i = 0u; i < nc; i = i + 1u) {
        let lon = u.natal_planets[i].x;
        let pos = ring_pos(lon_to_angle(lon), R_NATAL);
        let dh = length(pc - pos);

        // Outer glow
        let halo = 1.0 - smoothstep(0.0, HALO_R, dh);
        color = mix(color, u.gold_color.rgb, halo * 0.20);

        // Solid dot
        let dd = sdf_dot(pc, pos, PLANET_R);
        let dot_a = 1.0 - smoothstep(-pixel_w, pixel_w, dd);
        color = mix(color, u.gold_color.rgb, dot_a * 0.95);

        // Hot center
        let core = 1.0 - smoothstep(0.0, PLANET_R * 0.35, dh);
        color = mix(color, vec3<f32>(1.0, 0.95, 0.75), core * 0.45);
    }

    // ── 6. Transit planets (blue/red, animated drift) ─────────────
    let retro_rgb = vec3<f32>(u.retro_r, u.retro_g, 0.5);

    for (var i = 0u; i < tc; i = i + 1u) {
        let lon = u.transit_planets[i].x;
        let is_retro = u.transit_planets[i].y;
        let pos = ring_pos(lon_to_angle(lon) + drift, R_TRANSIT);
        let tc_rgb = select(u.transit_color.rgb, retro_rgb, is_retro > 0.5);
        let dh = length(pc - pos);

        // Glow
        let halo = 1.0 - smoothstep(0.0, HALO_R * 0.75, dh);
        color = mix(color, tc_rgb, halo * 0.16);

        // Dot
        let dd = sdf_dot(pc, pos, PLANET_R * 0.85);
        let dot_a = 1.0 - smoothstep(-pixel_w, pixel_w, dd);
        color = mix(color, tc_rgb, dot_a * 0.90);
    }

    // ── 7. Directional lighting (top-bright, bottom-dark) ─────────
    let light_y = -raw.y;            // screen-space: top = positive
    let light = 0.82 + 0.18 * light_y;
    color *= light;

    // ── 8. Rim glow (pulsing gold shimmer on outer edge) ──────────
    let edge_d = abs(r - R_OUTER);
    let rim = 1.0 - smoothstep(0.0, 0.10, edge_d);
    let pulse = 0.6 + 0.4 * sin(u.time * 0.4);
    color += u.gold_color.rgb * rim * 0.22 * pulse;

    // ── 9. Star field (twinkling points outside the chart) ────────
    if r > R_OUTER + 0.03 {
        let sp = raw * 45.0;
        let sid = floor(sp);
        let sf = fract(sp) - 0.5;
        let sv = hash(sid.x * 127.1 + sid.y * 311.7);
        if sv > 0.95 {
            let brightness = (sv - 0.95) * 20.0;
            let sd = length(sf);
            let star_a = 1.0 - smoothstep(0.0, 0.18, sd);
            let twinkle = 0.5 + 0.5 * sin(u.time * 1.5 + sv * TAU);
            color += vec3<f32>(0.85, 0.80, 0.65) * star_a * brightness * twinkle * 0.25;
        }
    }

    // ── 10. Outer vignette (darken widget edges) ──────────────────
    let vig = 1.0 - smoothstep(0.5, 1.05, length(raw));
    color *= 0.65 + 0.35 * vig;

    return vec4<f32>(color, 1.0);
}
