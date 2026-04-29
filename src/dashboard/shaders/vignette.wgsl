// Grimoire atmospheric vignette shader (v7.4)
// GPU-rendered background: radial vignette, noise grain, dust motes, gold glow

struct Uniforms {
    resolution: vec2<f32>,       // widget dimensions in pixels
    time: f32,                   // elapsed shader time (advances during animations)
    vignette_strength: f32,      // how dark edges get (0.0 = none, 1.0 = full)
    bg_color: vec4<f32>,         // grimoire_outer_bg() rgba
    gold_color: vec4<f32>,       // palette().gold rgba
    page_alpha: f32,             // page transition progress (0→1)
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Full-screen triangle: 3 vertices cover entire viewport, no vertex buffer needed
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOutput {
    var out: VertexOutput;
    // Triangle that covers clip space [-1,1]:
    // vid 0: (-1, -1), vid 1: (3, -1), vid 2: (-1, 3)
    let x = f32(i32(vid & 1u) * 4 - 1);
    let y = f32(i32(vid >> 1u) * 4 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// ── Hash functions for procedural noise ──────────────────────────

fn hash11(p: f32) -> f32 {
    return fract(sin(p * 127.1) * 43758.5453);
}

fn hash21(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

// ── Fragment shader ──────────────────────────────────────────────

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // 1. Base color — grimoire_outer_bg() used directly as center brightness.
    //    bg_color is grimoire_outer_bg() (25% of palette bg as of v7.5).
    //    Parchment: warm medium-dark brown — visible desk surface
    //    Leather:   dark warm brown — atmospheric but not black
    let desk_center = u.bg_color.rgb * 1.5;   // brighter center (was 1.2)
    let desk_edge   = u.bg_color.rgb * 0.15;  // very dark edges

    // 2. Radial vignette — lighter center, dark edges
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(uv, center) * 1.3;
    let vignette = smoothstep(0.15, 0.95, dist) * u.vignette_strength;
    var color = mix(desk_center, desk_edge, vignette);

    // 3. Static noise grain — position-based, no time dependency
    //    Scale relative to brightness so grain doesn't overwhelm dark mode
    let lum = dot(u.bg_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let grain_uv = floor(uv * u.resolution * 0.5);
    let grain_strength = 0.01 + lum * 0.04;  // 0.01 in dark, 0.03 in light
    let grain = (hash21(grain_uv) - 0.5) * grain_strength;
    color += vec3<f32>(grain);

    // 4. Dust motes — 12 procedural golden particles
    //    Positions drift with time; frozen when shader_time stops advancing
    var dust = 0.0;
    for (var i = 0u; i < 12u; i = i + 1u) {
        let seed = f32(i) * 127.1 + 31.7;
        let speed_x = 0.01 + hash11(seed * 1.3) * 0.015;
        let speed_y = 0.02 + hash11(seed * 2.7) * 0.025;

        // Lissajous-like drift pattern
        let px = fract(hash11(seed) + sin(u.time * speed_x + seed) * 0.3 + u.time * 0.005);
        let py = fract(hash11(seed * 3.1) - u.time * speed_y);

        let mote_pos = vec2<f32>(px, py);
        let d = distance(uv, mote_pos);

        // Soft circular glow, radius varies per particle
        let radius = 0.004 + hash11(seed * 5.3) * 0.004;
        let brightness = smoothstep(radius * 3.0, 0.0, d) * 0.25;

        // Fade motes near edges (don't want them in the book area)
        let edge_fade = smoothstep(0.0, 0.15, min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y)));
        dust += brightness * edge_fade;
    }
    let dust_color = u.gold_color.rgb * dust;
    color += dust_color;

    // 5. Gold edge glow during page transitions
    //    Visible when page_alpha < 1.0 (tab switch in progress)
    let glow_intensity = (1.0 - u.page_alpha) * 0.15;
    if glow_intensity > 0.001 {
        // Glow strongest at edges of the "book" area (center rectangle)
        let book_dist = max(abs(uv.x - 0.5) - 0.25, abs(uv.y - 0.5) - 0.3);
        let glow = smoothstep(0.08, 0.0, book_dist) * glow_intensity;
        color += u.gold_color.rgb * glow;
    }

    return vec4<f32>(color, 1.0);
}
