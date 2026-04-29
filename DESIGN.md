# Financial Dashboard — Design Document

**Project:** Pursuit NYC Week 4 Fellowship — Native Rust Desktop Financial Dashboard
**Stack:** Rust, Iced 0.13, SQLx, PostgreSQL
**Author:** Aisling Leiva
**Current version:** v9.3.0
**Next milestones:**
- v9.1 "The Polish" — P0 bug fixes (backtest crash, broken icons, tooltip contrast, disable rotation) + demo prep polish
- v10.0 "The Signal" — Data quality (Lagrange score sparsity fix, RSS tone sentiment, fetch-this-ticker, richer template agent)
- v11.0 "The Intelligence" — Astro symbols/houses on chart, Sun/Moon/Rising summary, backtest→history correlation, calculator defaults, loading progress bar, nav redesign

---

## Changelog

### v9.0.0 — "The Performance" (2026-04-29)

**Theme:** Systematic animation overhaul across all 3 rendering layers: GPU shaders, Iced widgets, and existing polish. Every visual element now has purposeful motion, from breathing natal planets to cursor-reactive dust motes. Designed for the "astrology-meets-finance" audience that values atmosphere as much as data.

**Key technical discoveries:**
- **Per-planet phase offset prevents synchronized pulsing:** Using `f32(i) * 1.7` as phase offset (1.7 is irrational relative to 2π) ensures 13 natal planets never visibly sync, creating organic breathing rhythm. Same technique could apply to any particle system needing varied timing
- **Line-progress projection for shimmer waves:** Computing `dot(pc - n_pos, seg) / dot(seg, seg)` gives a 0→1 progress along each aspect line segment, enabling per-fragment traveling alpha waves. Different shimmer speeds per aspect type (conjunction slow, square fast) adds semantic meaning to motion
- **Cursor-to-UV coordinate pipeline in Iced shaders:** `cursor.position_in(bounds)` returns widget-local coordinates. Dividing by bounds size gives [0,1] UV space matching the shader. Default (0.5, 0.5) when cursor is outside prevents edge artifacts
- **Uniform buffer field reuse:** VignetteUniforms stayed at 64 bytes by replacing 2 padding floats with `mouse_pos: [f32; 2]`. No buffer resize, no pipeline recreation. NatalWheel3DUniforms grew 496→512 (active_sign + 3 pad u32s, maintaining 16-byte alignment)
- **Candle grow-from-midpoint:** Using the price midpoint `(high + low) / 2` as the vertical origin and scaling outward via `mid_y + (target_y - mid_y) * candle_t` creates a natural "sprouting" effect rather than growing from top or bottom edge
- **bg_alpha acceleration for perceived staging:** `ease_out_cubic((progress * 3.0).min(1.0))` makes the background reach full alpha at 33% of the transition duration. No state machine needed — just faster math on the same progress float. Perceived as two-stage (background then content) despite being a single animation

**Changes (9 items, 12 files):**

| # | Change | Technical detail |
|---|--------|-----------------|
| 1 | 60fps astrology tab | `still_animating \|= self.active_tab == Tab::Astrology` in tick loop |
| 2 | Planet pulse/breathe | `PLANET_R * (1.0 + 0.15 * sin(time * 2.0 + i * 1.7))` per natal planet |
| 3 | Orbital transit trails | 5 ghost SDF dots per transit at `angle - g * 0.02` with alpha [0.08..0.60] |
| 4 | Aspect shimmer wave | `sin(line_t * TAU - time * speed)` with speed per aspect type |
| 5 | Zodiac active sign glow | `active_sign: f32` uniform, `1.3 + 0.1 * sin(time * 1.5)` brightness boost |
| 6 | Dust mote cursor repulsion | `mouse_pos` uniform, push force `(0.15 - dist) * 0.3` within 0.15 UV radius |
| 7 | Candlestick draw-in | Per-candle stagger `(i / total) * 0.6` delay, 500ms total, grow from midpoint |
| 8 | Layered page transitions | 300ms total, bg_alpha 3× speed, PAGE_TRANSITION_DURATION 0.25→0.30 |
| 9 | Tab sparkle tuning | 8 particles (was 5), sizes 1.5-4.0px, gravity drift, faster stagger |

### v8.0.0 — "The Observatory" (2026-04-28)

**Theme:** The natal chart goes 3D. Replaced the Canvas-based `NatalWheel` (2D `canvas::Program`) with a GPU-rendered `NatalWheel3DProgram` (`shader::Program`) that uses procedural signed distance functions to render a perspective-tilted zodiac wheel with glowing planets, animated aspect lines, twinkling stars, and rim glow — all computed per-pixel in a WGSL fragment shader.

**Key technical discoveries:**
- Procedural "fake 3D" via Y-axis foreshortening beats true vertex geometry for this use case: the inverse perspective transform (`y / (1 - tilt)`) maps each screen pixel back to "chart space" where the zodiac is a perfect circle, making all SDF calculations trivial. With `camera_tilt=0.32` (32% Y compression), the visual result — a pronounced elliptical tilted disc with directional lighting — is indistinguishable from true 3D at 400×400px
- Uniform buffer layout between Rust `#[repr(C)]` and WGSL must match byte-for-byte. `[[f32; 4]; 13]` in Rust maps to `array<vec4<f32>, 13>` in WGSL with identical 16-byte stride — no padding surprises because `[f32; 4]` happens to start at 16-byte-aligned offsets in the struct layout
- WGSL `switch` with explicit `case N:` is more portable than `const` arrays for the 12 sign colors — avoids implementation-dependent behavior with module-scope array constructors
- Aspect computation in the fragment shader runs a nested loop (max 13×13 = 169 iterations) per pixel — acceptable on modern GPUs because each iteration is just angle arithmetic + one `sdf_segment()` call with an early `if asp_alpha > 0.0` guard
- `shader::Storage` uses `TypeId` for keying — `NatalWheel3DPipeline` and `VignettePipeline` coexist with zero collision risk

**Changes:**

1. **NatalWheel3DProgram** (`shaders/mod.rs`): New `shader::Program<Message>` implementation following the VignetteProgram pattern exactly. 496-byte uniform buffer packs resolution, time, tilt, 3 color channels, 13 natal planet slots, 13 transit planet slots, counts, and retrograde color. Planet data packed in `draw()` from `Vec<NatalPosition>` / `Vec<DailyTransit>`.
2. **natal_wheel_3d.wgsl** (new file): ~230-line WGSL fragment shader. Full-screen triangle vertex shader (identical to vignette.wgsl). Fragment shader renders: 12 zodiac arc segments, 4 concentric ring strokes, 12 sign dividers, aspect lines (4 types), natal planet dots with glow halos, transit planet dots with animated drift, directional lighting, rim glow, star field, outer vignette.
3. **View integration** (`astrology_tab.rs`): `Canvas::new(NatalWheel{...})` → `Shader::new(NatalWheel3DProgram{...})` with theme colors passed from `palette()`.
4. **Dead code suppression** (`astrology.rs`): `#[allow(dead_code)]` on NatalWheel, SIGN_COLORS, lon_to_angle — kept as 2D reference documentation.

**Files modified:** `shaders/mod.rs`, `view/astrology_tab.rs`, `astrology.rs`, `Cargo.toml`, `CHANGELOG.md` + 1 new (`shaders/natal_wheel_3d.wgsl`)

---

### v7.6.0 — "The Consistency" (2026-04-28)

**Theme:** Visual consistency pass. Every sub-scrollable now matches the main gold scrollbar. Canvas-rendered sparkle particles replace the Unicode fallback. Transit planets animate. Concordance column no longer truncates. All ornaments visible in Parchment.

**Key technical discoveries:**
- Iced scrollbar style functions have signature `Fn(&Theme, Status) -> Style` — extracting to a named function in `shared.rs` enables reuse across 15+ scrollables without closure repetition
- Canvas sparkle particles use deterministic positions seeded per tab index, so each tab's particle burst looks unique without randomness (no PRNG needed)
- Transit ring drift uses `lon_to_angle() + drift` where `drift = (time * 0.5).to_radians()` — 0.5°/sec rotation is slow enough to feel atmospheric but fast enough to notice within 10 seconds

**Changes:**

1. **Gold sub-scrollbar styling:** Extracted `gold_scrollbar_style()` function to `shared.rs`. Applied to all 15 data-table scrollables across `overview.rs` (2), `universe.rs` (3), `research.rs` (6), `fundamentals.rs` (2). Main page scrollbar in `mod.rs` refactored to use same helper. Sub-scrollable alphas slightly lower (0.35/0.25 vs 0.45/0.30) so they don't compete with the main scrollbar.
2. **Concordance column fix:** Universe table "Conc" column width increased 50→90px in both header and data rows. "Strong Confirm", "Mild Confirm", "Divergence" now fully readable.
3. **Animated transit ring:** Added `time: f32` field to `NatalWheel` struct. Transit planet angles offset by `(time * 0.5).to_radians()` creating 0.5°/sec celestial drift. View passes `self.shader_time` from dashboard state. Animation runs during active tick mode (16ms/60fps).
4. **Canvas sparkle particles:** New `TabSparkle` canvas program in `ornaments.rs`. Draws 5 gold dots at deterministic positions (no PRNG). Each particle staggers fade-in by `i * 0.12` delay. Replaces Unicode `\u{2726}` in tab hover state. 20×16px canvas in tab row.
5. **Fetch error guidance:** Improved scraper-not-found message to include `cargo build --bin scraper` instruction instead of raw path.
6. **Ornament contrast (Tier 1):** All 3 canvas ornaments alpha-boosted for Parchment visibility: PageHeaderOrnament (gold 0.5→0.7, fill 0.2→0.35, rule 0.25→0.4), BookSpine (rule 0.3→0.45, diamond 0.4→0.55), PageBorderCorner (line 0.4→0.55, dot 0.5→0.65).
7. **Ticker-specific empty states (Tier 1):** 8 generic "for this ticker" messages now interpolate `self.selected_ticker` via `format!()` across `overview.rs`, `fundamentals.rs`, `astrology_tab.rs`, `research.rs`.

**Files modified:** `ornaments.rs`, `shared.rs`, `mod.rs`, `overview.rs`, `universe.rs`, `research.rs`, `fundamentals.rs`, `astrology_tab.rs`, `astrology.rs`, `data.rs`, `Cargo.toml`, `CHANGELOG.md`

---

### v7.5.0 — "The Polish" (2026-04-28)

**Theme:** Video review feedback sprint. User recorded a narrated screen recording identifying bugs, UX issues, and visual polish needs across all tabs. This version addresses 12 items in 3 sprints: functional bugs, UX polish, and visual enhancements.

**Key technical discoveries:**
- Iced 0.13 `scrollable::Style` uses `Rail` struct (not `Scrollbar`) with `background`, `border`, `scroller` sub-fields
- `button::Style { background: None }` makes buttons visually transparent while keeping click behavior — needed for custom tab containers
- Canvas `fill()` with arc paths creates filled zodiac ring segments — no gradient API needed, element colors per-sign achieve the effect
- Active tab label in `p.gold` color pops against both cream and dark backgrounds without needing different per-theme logic

**Changes:**

1. **Scrollbar styling (Sprint 1A):** Custom `scrollable::Style` with gold scroller on translucent rail. 10px right padding on page_content prevents content overlap with scrollbar.
2. **Fetch error display (Sprint 1B):** Persistent orange warning banner renders `fetch_ticker_error` below nav bar. Pre-flight check: if `scraper.exe` missing, shows path in error immediately. Gold loading bar during active fetch.
3. **Gauge grid layout (Sprint 2A):** 5 gauges rearranged from horizontal scrollable row to 3+2 grid (two rows). Eliminates horizontal scrollbar.
4. **Leather vignette warmth (Sprint 2B):** `grimoire_outer_bg()` multipliers bumped 0.15→0.25/0.22/0.18. Shader `desk_center` multiplier 1.2→1.5. Leather mode shows warm brown desk instead of near-black.
5. **Natal chart beautification (Sprint 3A):** Element-colored zodiac ring segments (fire=red, earth=green, air=yellow, water=blue at 30% alpha). Gold glow halos on natal planets. Blue glow on transits. Aspect lines drawn in center circle with type-specific widths (conjunction=1.5, square/trine=1.2, sextile=0.8). Planet glyphs (☉♀♂♃♄) replace abbreviations. Canvas 300→400px.
6. **Tab sparkle animation (Sprint 3B):** Gold "✦" character fades in after label during hover with delayed alpha ramp `(eased * 1.5 - 0.3)`.
7. **Active tab visibility:** Active tab always shows bold icon + gold-colored label + 3px gold underline + surface background. Three-tier states: active (gold label, always visible), hovered (ink label + sparkle fade in), default (icon only).

**Before/After:**

| Feature | Before (v7.4.1) | After (v7.5.0) |
|---------|-----------------|----------------|
| Scrollbar | Default iced, overlaps content | Gold scroller, translucent rail, right padding |
| Fetch errors | Toast only (5s expiry) | Persistent orange banner + pre-flight scraper check |
| Gauges | Horizontal scrollable row | 3+2 grid, no horizontal scroll |
| Leather bg | Near-black (0.15 multiplier) | Warm brown (0.25 multiplier) |
| Natal chart | 300px, flat circles, abbreviations | 400px, colored zodiac ring, glow halos, glyphs |
| Active tab | Icon only, 2px underline | Gold label always visible, 3px underline |
| Tab hover | Label fades in | Label + sparkle ✦ fade in |

**Files modified:** 8 (`view/mod.rs`, `update/data.rs`, `view/overview.rs`, `view/astrology_tab.rs`, `theme.rs`, `shaders/vignette.wgsl`, `astrology.rs`, `DESIGN.md`, `CHANGELOG.md`, `Cargo.toml`)

---

### v7.4.1 — "The Grimoire — Header Redesign" (2026-04-28)

**Theme:** User video review flagged right-side vertical tabs as a regret — "I'm starting to regret the decision to put it on the right side" and "I don't like the square tabs." Moved all 8 navigation tabs from the right-side vertical strip to a horizontal bar at the top of the book page, positioned between the header ornament and the compact nav row. Tabs function as grimoire chapter headers.

**Key technical insight:**
- The animation system (`hovered_tab`, `tab_hover_progress[8]`, `TabHoverEnter/Exit` messages, per-tab tick logic) was designed layout-agnostic — it tracks hover state without knowing rendering direction. Converting vertical→horizontal was a **single-file view change** with zero state/update modifications.
- Iced's `button()` widget applies default blue chrome over nested content. Fix: transparent `button::Style` with `background: None` so inner container styling shows through.
- Iced 0.13 `Radius` only implements `From<f32>` (uniform), not `From<[f32; 4]>` (per-corner).

**Before/After:**

| Feature | Before (v7.3–7.4) | After (v7.4.1) |
|---------|-------------------|----------------|
| Tab position | Right-side vertical column | Horizontal bar under header ornament |
| Tab shape | Square containers with stagger cascade | Inline icons with gold bottom border |
| Layout root | `row![spine, book_page, grimoire_tabs]` | `row![spine, book_page]` (tabs inside page_content) |
| Right side | Dark strip with tab column | Clean vignette showing through |
| Easing | `ease_out_back` (bounce overshoot) | `ease_out_cubic` (smooth decel) |
| Active indicator | 3px gold full border | 2px gold bottom underline |

**Changes:**
1. **`build_grimoire_tabs()` → `build_tab_bar()`:** Horizontal `Row` replaces vertical `Column`. Icon-only at rest, icon+label fades in on hover. Active tab: gold bottom border + surface background. No stagger offset.
2. **Layout restructure:** Tab bar inserted into `page_content` column between `PageHeaderOrnament` and `compact_nav`. Removed `grimoire_tabs` from `book_layout` row.
3. **Transparent button style:** `button::Style { background: None, border: Border::default() }` so container's gold-underline styling shows through.
4. **Dead code removal:** Entire `build_grimoire_tabs()` method deleted (~110 lines). Dark background strip, stagger cascade, vertical column all gone.

**Files modified:** 1 (`src/dashboard/view/mod.rs` — ~120 lines changed)

---

### v7.4.0 — "The Atmosphere" (2026-04-28)

**Theme:** v7.3's dark outer frame was a flat container. This version replaces it with a GPU-rendered atmospheric vignette using Iced 0.13's `Shader` widget and wgpu. The background now has a radial vignette (lighter center, dark edges), static noise grain for texture, 12 procedural golden dust motes that drift during animations, and a gold edge glow during page transitions.

**Key technical discoveries:**
- `wgpu` feature already enabled in iced defaults — no Cargo.toml iced line change needed
- wgpu 0.19 (bundled in iced 0.13): `entry_point` is `&str` not `Option<&str>`, no `compilation_options` or `cache` fields
- Full-screen triangle technique: 3 vertices from `vertex_index`, no vertex buffer needed
- LCD shadow color distortion: RGB values below ~0.05 appear purple on LCD panels. Keep visible darks above 0.10.
- Power-efficient animation: `shader_time` only advances during 16ms ticks. Dust motes freeze at idle (30s ticks).

**Before/After:**

| Feature | Before (v7.3) | After (v7.4) |
|---------|---------------|--------------|
| Outer background | Flat dark container (`grimoire_outer_bg()`) | GPU vignette shader with radial falloff |
| Texture | None | Static hash noise grain (luminance-adaptive) |
| Particles | None | 12 golden dust motes (Lissajous drift, frozen at idle) |
| Tab transition effect | Page alpha fade only | + gold edge glow on book border |
| Compositing | `container(book_layout)` | `stack![vignette_shader, padded_book]` |

**Changes:**
1. **WGSL shader** (`shaders/vignette.wgsl`, new): Full-screen triangle vertex shader, fragment with radial vignette, hash noise grain, 12 procedural dust motes, gold edge glow
2. **Shader pipeline** (`shaders/mod.rs`, new): `VignetteUniforms` (bytemuck Pod, 64B), `VignettePipeline` (lazy-init in Storage), `VignettePrimitive` (prepare/render), `VignetteProgram` (Program trait impl)
3. **State:** `shader_time: f32` field, advances during animation ticks
4. **View:** Replaced `outer_frame()` with `stack![Shader::new(VignetteProgram{..}), padded_book]`
5. **Bug fixes:** Parchment vignette too dark (LCD purple), tab icon colors inverted

**Files modified:** 7 + 2 new (`shaders/mod.rs`, `shaders/vignette.wgsl`)

---

### v7.3.0 — "The Grimoire" (2026-04-27)

**Theme:** v7.2 added motion, but the layout was still conventional — top tab bar, standard column flow. The user wanted something dramatically different: a video game grimoire. Inspired by RPG spellbook UIs (physical tab dividers, aged parchment pages, dark atmospheric backgrounds), this version transforms the dashboard into an open book with right-side tab dividers that expand on hover, canvas-rendered ornamental decorations, and a dark frame creating the effect of a book sitting on a desk.

**Key technical discoveries:**
- `iced::widget::mouse_area` — first usage in codebase, provides `on_enter`/`on_exit` for hover detection
- `ease_out_back` easing — elastic overshoot (c1=1.70158) for playful game-feel tab expansion
- Canvas ornaments follow exact same `canvas::Program` pattern as existing gauges/charts

**Before/After:**

| Feature | Before (v7.2) | After (v7.3) |
|---------|---------------|--------------|
| Tab position | Horizontal top bar with gold underline | Right-side vertical book dividers with stagger cascade |
| Tab interaction | Click only | Click + hover expand (icon→icon+label, 200ms ease_out_back) |
| Layout root | `column![header, tabs, content]` | `row![spine, book_page, grimoire_tabs]` inside dark outer frame |
| Outer frame | None (content fills window) | Dark atmospheric background (grimoire_outer_bg: bg * 0.15) |
| Decorations | None | Canvas: BookSpine (cross-stitch), PageHeaderOrnament (Renaissance flourish), PageBorderCorner (bracket + diamond) |
| Tab switch | Instant content swap | 250ms page transition fade ("materializing from darkness") |
| Navigation | Header row + separate nav strip + tab bar (~130px) | Compact single-row nav (~50px) |

**Changes:**
1. **State foundation:** 4 new Dashboard fields (`hovered_tab`, `tab_hover_progress[8]`, `page_transition_progress`, `page_transition_from`), 2 new Messages (`TabHoverEnter`, `TabHoverExit`), `Tab::index()` method
2. **Animation additions:** `ease_out_back` easing, `TAB_HOVER_EXPAND_DURATION` (200ms), `TAB_HOVER_COLLAPSE_DURATION` (150ms), `PAGE_TRANSITION_DURATION` (250ms)
3. **Right-side grimoire tabs:** `build_grimoire_tabs()` in `view/mod.rs` — 8 physical tab dividers with `mouse_area` hover detection, animated width (48px→168px), staggered cascade (`idx * 3px` offset), gold accent on active/hovered
4. **Book page styling:** Content wrapped in parchment container with transition alpha, inside dark outer frame. Page spine on left edge
5. **Canvas ornaments:** `ornaments.rs` (new) — `BookSpine` (cross-stitch binding with diamond endcaps), `PageHeaderOrnament` (central lozenge + sine-wave scrollwork + extending rules), `PageBorderCorner` (perpendicular arms + gold diamond vertex)
6. **Page transition:** Background alpha fade from 40%→100% over 250ms with `ease_out_cubic` on tab switch
7. **Compact navigation:** Header + search + tickers + actions merged into single row, removed horizontal rules, reduced vertical chrome
8. **Theme additions:** `GRIMOIRE_SPINE`, `GRIMOIRE_STITCH`, `GRIMOIRE_TAB_SHADOW` colors, `grimoire_outer_bg()` function

**Files modified:** 9 + 1 new (`ornaments.rs`)

---

### v7.2.0 — "The Motion" (2026-04-27)

**Theme:** v7.1 got the spatial layout right — the dashboard looked like a book. But it felt static. Everything changed instantly: gauges popped to values, tabs snapped, toasts appeared and vanished. UI/UX teacher feedback: "right direction, long way to go." This version adds motion design, upgrades from Bootstrap to Phosphor Icons, implements viewport-aware responsive font scaling, and fixes version control drift. Also fixes the vertical text wrapping bug in TECHNICAL INDICATORS identified in the v7.1 video review.

**Before/After:**

| Feature | Before (v7.1) | After (v7.2) |
|---------|---------------|--------------|
| Icons | Bootstrap Icons (1 weight, generic) | Phosphor Icons (regular + bold, 1530 icons, duotone capable) |
| Animation | None — all state changes instant | Gauge sweep (600ms), toast fade (500ms), tab indicator crossfade (200ms) |
| Font scaling | Manual 4-step toggle only | Viewport-aware auto-scale + manual toggle |
| Tick rate | 30s fixed | 16ms adaptive (60fps when animating, 30s at rest) |
| Version control | Cargo.toml: 0.1.0, no git tags | Cargo.toml synced, 7 git tags, CHANGELOG.md in root |
| Indicator layout | 6 items horizontal (wraps vertically) | 2×3 grid (always fits) |
| Recently viewed | Unbounded overflow | Capped at 6 most recent |

**Changes:**
1. **Version control cleanup:** Cargo.toml 0.1.0→7.1.0 (then 7.2.0), created CHANGELOG.md in root, git tags for v4.0.0-v7.2.0
2. **Bug fix — indicator text wrapping:** Changed TECHNICAL INDICATORS from 6-item horizontal `row![]` to 2×3 grid (`column![row![3], row![3]]`). Root cause: `FillPortion(3)` (60% width) too narrow for "Sentiment: Somewhat-Bullish". File: `overview.rs`
3. **Bug fix — recently-viewed overflow:** Cap at 6 most recent tickers via `rev().take(6).rev()`, reduce font to `text_sm()`. File: `mod.rs`
4. **Phosphor Icons:** Replaced Bootstrap Icons with Phosphor (phosphoricons.com). 28 codepoints remapped. Removed `iced_fonts` crate dependency. Added `Phosphor.ttf` + `Phosphor-Bold.ttf` (~980KB). `icon_bold()` helper for emphasis states. Files: `icons.rs`, `main.rs`, `Cargo.toml`
5. **Animation infrastructure:** New `animation.rs` module with easing functions (`ease_out_cubic`, `ease_in_out_quad`, `ease_in_quad`, `lerp`). Animation state fields in Dashboard struct. Adaptive tick rate: 16ms during animation, 30s at rest. Files: `animation.rs`, `state.rs`, `update/mod.rs`
6. **Gauge sweep animation:** Fear/Greed gauge needle sweeps from old score to new over 600ms with `ease_out_cubic`. Triggered on `FearGreedLoaded`. File: `overview.rs`, `update/data.rs`
7. **Toast fade-out:** Toasts fade opacity from 1.0→0.0 over last 500ms of 4s lifetime (background + text + border). File: `mod.rs`
8. **Tab indicator crossfade:** Gold underline fades out from old tab, fades in on new tab over 200ms with `ease_in_out_quad`. File: `mod.rs`
9. **Responsive font scaling:** Viewport width tracking via `window::resize_events()` subscription. Auto-scale multiplier: <1024px→0.85, 1024-1440→1.0, 1440-1920→1.05, >1920→1.1. Multiplied into existing `s()` function alongside manual FONT_SCALE. Files: `theme.rs`, `state.rs`, `update/mod.rs`

**Files modified:** 13 (`Cargo.toml`, `CHANGELOG.md`, `icons.rs`, `main.rs`, `animation.rs`, `state.rs`, `update/mod.rs`, `update/data.rs`, `gauges.rs`, `theme.rs`, `view/overview.rs`, `view/mod.rs`, `tabs.rs`)
**New files:** 3 (`animation.rs`, `Phosphor.ttf`, `Phosphor-Bold.ttf`)
**Removed deps:** `iced_fonts`

### v7.1.0 — "The Ledger — Spatial Polish" (2026-04-27)

**Theme:** Bridging the spatial gap between the Renaissance book aesthetic (shipped in v7.0) and real-world layout. v7.0 got the palette and typography right — circadian colors, four-role fonts, semantic theming. But the layout was still generic: full-bleed content on wide monitors, 200px of header chrome consuming 26% of a 768px viewport, no visual rhythm between sections. This version applies the spatial language extracted from the Berkshire Hathaway HTML redesign reference: 1240px max-width, 8px spacing base, eyebrow labels, 1px rule dividers with breathing room, and a vertical flow overview with the price chart as hero element. Driven by a 185-frame video review that identified 12 visual issues across all 8 tabs.

1. **Bug fix: paper trail buy threshold** — the paper trail view displayed "exceed 75" for the buy threshold, but the paper engine was already lowered to 65 in a prior commit. Updated the stale UI text to match the actual `BUY_THRESHOLD` constant.
   - *Files:* `src/dashboard/view/paper_trail.rs` (1 string change)
   - *Insight:* A good example of why threshold values should be referenced from constants rather than hardcoded in UI strings. The scraper-side threshold changed but the dashboard-side text was a separate string literal.

2. **Spacing constants and layout primitives** — added 7 spatial constants to `theme.rs`: `SPACE_XS` (4px), `SPACE_SM` (8px), `SPACE_MD` (16px), `SPACE_LG` (24px), `SPACE_XL` (40px), `MAX_WIDTH` (1240px), `RADIUS_CARD` (4px). Added 3 reusable layout functions to `shared.rs`: `max_container()` wraps content in a 1240px centered container, `eyebrow()` renders an uppercase gold label in Inter SemiBold, `section_rule()` renders a horizontal rule with 8px vertical breathing room. Updated `card()` to reference `SPACE_MD` and `RADIUS_CARD` constants instead of magic numbers.
   - *Files:* `src/dashboard/theme.rs` (+7 constants), `src/dashboard/view/shared.rs` (+3 functions, card updated)
   - *Insight:* The 1240px max-width comes directly from the BH redesign's CSS `max-width`. The 8px spacing base was extracted via `designlang` from the BH reference HTML. These aren't arbitrary — they're the exact values that make the reference design scannable. Iced's `max_width()` takes `impl Into<Pixels>` which accepts `f32` directly; the initial `as u32` cast failed because `Pixels` only implements `From<f32>` and `From<u16>`.

3. **Compact 2-row header** — collapsed 7+ vertical header elements (~200px) into 2 rows (~80px). Row 1: ticker name (Fraunces display) + right-aligned refresh/fetch/theme buttons. Row 2: search bar + ticker buttons + recently viewed (single nav strip). Removed the tab subtitle text (redundant with the visible tab bar). Removed the "Loaded 100 rows for AAPL" status text (developer-facing, not user-facing). Replaced magic `spacing(10)` and `padding(20)` with `SPACE_SM` and `SPACE_LG` constants.
   - *Files:* `src/dashboard/view/mod.rs` (lines 73-184 rewritten)
   - *Insight:* The status text removal is significant UX-wise. Status messages like "Loaded 100 rows" are useful during development but create visual noise for users. The same information is available in Settings > Dashboard Info for those who need it. The ~120px vertical reclaim means 15% more content visible on a 768px laptop screen.

4. **Max-width container** — wrapped the scrollable content area in `max_container()`, capping content at 1240px centered. On a 1920px monitor, this creates ~340px margins on each side. One-line change (`scrollable(content)` → `scrollable(shared::max_container(content))`) with massive visual impact.
   - *Files:* `src/dashboard/view/mod.rs` (1 line changed)
   - *Insight:* The max-width constraint is the single biggest readability improvement. Without it, text lines on wide monitors stretch to 200+ characters, forcing constant eye-tracking across the full viewport width. The 1240px cap keeps text within the BH redesign's 68ch measure guideline.

5. **Eyebrow labels and section rules across all 8 tabs** — inserted `eyebrow("LABEL")` before each major section and `section_rule()` between sections in all 8 tab views. Added imports for `eyebrow` and `section_rule` to all view files. Replaced hard `horizontal_rule(1)` dividers in final assembly columns with breathing-room `section_rule()`. Replaced magic `spacing(10)` with `theme::SPACE_SM` in assembly columns.
   - *Eyebrows by tab:*
     - Overview: MARKET SENTIMENT, PRICE ACTION, SIGNAL INTELLIGENCE, SCORED UNIVERSE, PREDICTION MARKETS, MACRO & MARKETS
     - Astrology: NATAL CHART, ASTRO CALENDAR, BACKTEST
     - Universe: UNIVERSE EXPLORER, SECTOR MAP, LAGRANGE ALERTS
     - Fundamentals: VALUATION, DCF CALCULATOR, OPTIONS GREEKS, THE COUNCIL, COMPARATIVE ANALYSIS, EARNINGS, PRICE HISTORY
     - Research: FILINGS & NEWS, MARKET NEWS, GEOPOLITICS, INSIDER ACTIVITY, INSTITUTIONAL HOLDERS
     - Portfolio: PORTFOLIO, TRANSACTIONS, WATCHLISTS
     - Paper Trail: PAPER TRADING, OPEN POSITIONS, PERFORMANCE, EQUITY CURVE, TRADE LOG
     - Settings: APPEARANCE, DATA & REFRESH, API KEYS, ALERTS, DASHBOARD INFO
   - *Files:* All 8 view files in `src/dashboard/view/` (imports + eyebrow/section_rule insertions)
   - *Insight:* Eyebrow labels are the BH redesign's most distinctive structural element — uppercase, small, gold-colored category tags above sections (like "SIXTY YEARS OF COMPOUNDING"). They create a scanning rhythm that lets users jump between sections without reading headings. Combined with section rules that have 8px breathing room (vs the previous raw 1px horizontal_rule), they establish the book-like vertical cadence that makes the BH redesign feel typeset rather than coded.

6. **Font role enforcement** — added ~65 `.font(font::INTER)` calls to numeric display text across 7 view files. All prices, scores, percentages, ratios, and statistical values now render in Inter (the clean sans-serif optimized for tabular numerals). Added `use crate::font` import to `paper_trail.rs` which hadn't needed it before. Added one `.font(font::BODY)` call to Polymarket question text for Source Serif 4 prose rendering.
   - *Targets:* Overview (indicators, watchlist scores, polymarket %, Lagrange verdict), Shared (13 macro strip values), Paper Trail (account values, positions, stats, SPY benchmark, trade log), Universe (score columns, alert scores), Fundamentals (DCF results, Greeks results, OHLCV prices), Portfolio (P&L totals), Research (insider shares/prices, holdings)
   - *Files:* `overview.rs`, `shared.rs`, `paper_trail.rs`, `universe.rs`, `fundamentals.rs`, `portfolio_tab.rs`, `research.rs`
   - *Insight:* v7.0 deliberately left numeric values in the default Source Serif 4 body font, arguing that "serif numerals reinforce the Renaissance book identity." After the BH reference extraction, the design token analysis showed the reference uses Inter for all numeric values. The distinction matters: Fraunces for headings (ornate identity), Source Serif 4 for prose (reading comfort), Inter for data (tabular clarity). Numbers in a financial dashboard are data, not prose — they need Inter's consistent widths for column alignment and quick scanning.

7. **Overview tab restructure** — converted from a two-column split (60/40 FillPortion) to vertical flow with the price chart as hero element. Chart height increased from 250px to 300px. Gauges row unwrapped from card container to let canvas elements breathe. Technical indicators and patterns placed in a 60/40 side-by-side row. Signals and watchlist kept side-by-side in cards. Macro strip and polymarket combined at the bottom under "MACRO & MARKETS" eyebrow.
   - *Files:* `src/dashboard/view/overview.rs` (assembly section rewritten)
   - *Insight:* The BH redesign's most impactful layout pattern is leading with a full-width hero visualization (the book value chart spanning the entire content width). Our price chart was previously squeezed into 60% of the viewport. At full width on a 1240px-capped layout, the chart is now ~1200px wide — 2x the previous effective width. More data points are visible, candle bodies are wider and easier to read, and the chart becomes the visual anchor of the Overview tab rather than competing with the signals panel for attention.

**Post-upgrade metrics:**

| Metric | v7.0.0 | v7.1.0 |
|--------|--------|--------|
| Header chrome height | ~200px (7 elements) | ~80px (2 rows) |
| Content max-width | Unbounded (full viewport) | 1240px centered |
| Spacing system | Magic numbers (10, 20) | 5 named constants (XS/SM/MD/LG/XL) |
| Layout primitives | 2 (card, section_heading) | 5 (+max_container, eyebrow, section_rule) |
| Eyebrow labels | 0 | 38 across 8 tabs |
| Section rules | 0 | ~30 across 8 tabs |
| Font-annotated numerics | 0 (.font() calls on data text) | ~65 Inter-annotated values |
| Overview chart width | ~60% of viewport (FillPortion 3/5) | 100% of 1240px container |
| Overview chart height | 250px | 300px |
| Overview layout | Two-column split | Vertical flow with hero chart |
| Unused code warnings | 0 | 0 |
| Tests | 70 | 70 (all pass) |
| Build | 0 warnings | 0 warnings |
| Files modified | N/A | 11 (theme.rs, shared.rs, mod.rs, + 8 view files) |

### v7.0.0 — "The Ledger" (2026-04-26)

**Theme:** Complete UI/UX overhaul replacing generic Catppuccin theming with a Renaissance book aesthetic. The visual identity shifts from "generic fintech terminal" to "Venetian merchant's ledger meets Bloomberg." Light mode (Parchment) renders as aged paper with sepia-tinted typography; dark mode (Leather) renders as a leather-bound book with cream text on deep oak. Both modes shift through 24 circadian stages (one per hour), smoothly interpolating between 4 anchor palettes per mode. Typography moves from a single sans-serif (Inter) to a four-role system: ornate display serif for headings, readable body serif as default, sans-serif for numerics, and monospace for tabular data. The design philosophy blends Warren Buffett's "clear signal over noise" with Apple's "clarity, deference, depth."

1. **LedgerPalette engine** — replaced all Catppuccin color constants (~30 MOCHA_*/LATTE_* values) with an 11-channel semantic palette struct: bg, surface, ink, ink_soft, ink_faint, rule, rule_strong, accent, gold, bullish, bearish. A global `RwLock<Option<LedgerPalette>>` cache holds the current computed palette, updated every 30 seconds on Tick. All 18 semantic color functions (`canvas_bg()`, `fg()`, `fg_dim()`, `surface()`, etc.) read from this cache instead of branching on dark/light mode. This means every part of the UI (widgets, canvas charts, cards, toast overlays) gets circadian-aware colors from a single source of truth.
   - *Files:* `src/dashboard/theme.rs` (~350 lines rewritten)
   - *Insight:* The palette cache uses `RwLock` rather than `Mutex` because canvas widgets call semantic color functions during `draw()` on the render thread, while palette updates happen on `Tick` from the subscription thread. `RwLock` allows concurrent reads from multiple canvas widgets without blocking each other. The `Option` wrapper handles the initial state before the first `Tick` fires, falling back to a Day palette.

2. **8 anchor palettes with linear interpolation** — each mode (Parchment and Leather) has 4 anchor palettes at dawn (hour 5), day (hour 10), dusk (hour 18), and night (hour 22). Between anchors, `lerp_color()` performs per-channel linear interpolation in sRGB space across all 11 palette fields. The interpolation curve: hours 5-7 blend dawn to day, 8-16 hold day, 17-19 blend day to dusk, 20-22 blend dusk to night, 23-4 hold night. Parchment anchors are derived from the Berkshire Hathaway HTML redesign CSS variables (cream backgrounds, sepia ink, gold accents). Leather anchors are new designs (walnut/oak/mahogany backgrounds, cream text, amber accents).
   - *Files:* `src/dashboard/theme.rs` (+lerp_color, +lerp_palette, +compute_palette, +8 const LedgerPalette anchors)
   - *Insight:* Linear interpolation in sRGB space is technically imperfect (perceptual uniformity would require okLab), but for the subtle shifts between neighboring palettes the difference is imperceptible. The simplicity of sRGB lerp (4 multiplications per channel, no gamma conversion) matters because `compute_palette()` runs on every Tick and is called during theme rebuild.

3. **ThemeMode simplification** — replaced the 4-mode cycle (Auto/AlwaysLight/AlwaysDark/TokyoNight) with 3 modes: Auto (Parchment by day, Leather by night, based on system clock), Parchment (always light, still shifts through 24 stages), and Leather (always dark, still shifts through 24 stages). TokyoNight was removed as it conflicted with the unified design language. Backward compatibility: "Light"/"Dark" strings in the settings database map to Parchment/Leather respectively.
   - *Files:* `src/dashboard/theme.rs` (ThemeMode enum), `src/dashboard/update/mod.rs` (ToggleTheme, SettingsLoaded, SaveSetting handlers)
   - *Insight:* Even in forced Parchment or Leather mode, the circadian stages still cycle. A user who works late nights in Parchment mode will see warmer, deeper parchment tones after 10pm (the Night anchor uses bg #e8dcc8 vs Day's #faf7f0). The palette always breathes with the time of day.

4. **Circadian preview slider** — new circadian override system allows previewing any hour's palette without waiting for real time. A slider (0-23) in the Settings tab sets `circadian_override: Option<u32>`. When Some, the palette is frozen at that hour. When None (default), the system clock drives updates. The slider shows the current hour with a time-of-day label (e.g., "14:00 -- Afternoon"). A "Reset to clock" button clears the override. This is essential for demo presentations where you want to show the full circadian range in seconds rather than waiting 24 hours.
   - *Files:* `src/dashboard/state.rs` (+circadian_override field, +2 Message variants), `src/dashboard/update/mod.rs` (+2 handlers), `src/dashboard/view/settings.rs` (+slider card, ~30 lines)
   - *Insight:* The override is intentionally not persisted to the database. If you override to hour 3 (deep night leather) for a demo, you don't want to relaunch the app the next morning and wonder why it's dark. The override resets to auto on every launch.

5. **Four-role typography system** — replaced the single-font approach (Inter everywhere) with four purpose-built roles. Display headings use Fraunces, an ornate serif with optical size variation that evokes Renaissance printing. Body text uses Source Serif 4, a highly readable serif designed for screen text. Numerics (prices, scores, percentages) keep Inter for its tabular figure support. Code and tabular columns use JetBrains Mono. Both Fraunces and Source Serif 4 are variable fonts (~1.5MB total) embedded via `include_bytes!()` and registered at app startup. The default font for the entire app changed from Inter to Source Serif 4.
   - *Files:* `assets/fonts/Fraunces-Variable.ttf` (new, 360KB), `assets/fonts/SourceSerif4-Variable.ttf` (new, 1.2MB), `src/dashboard/font.rs` (rewritten, +DISPLAY/BODY/BODY_BOLD constants), `src/dashboard/main.rs` (+2 .font() registrations, changed default_font)
   - *Insight:* Variable fonts contain the entire weight axis in a single file. Iced's fontdb selects the appropriate weight based on the `Weight` field in each `Font` constant. This means Fraunces Semibold (display) and a hypothetical Fraunces Regular (body) come from the same TTF, without shipping separate files for each weight.

6. **Shared component restyling** — the `card()` wrapper now uses custom container styling with palette-driven surface/rule colors and 4px border radius (was the generic `container::rounded_box` from Iced). `section_heading()` uses Fraunces display font. The tab bar's active indicator changed from a bordered box to a gold accent line using `palette().gold`. The toast notification overlay uses palette-aware surface colors with 94% opacity instead of hardcoded dark rgba. The header title uses Fraunces display font.
   - *Files:* `src/dashboard/view/shared.rs` (card, section_heading), `src/dashboard/view/mod.rs` (header, tab bar, toast)
   - *Insight:* The card border radius is deliberately small (4px) rather than the modern trend of 12-16px pill shapes. This matches the Renaissance book aesthetic: sharp corners with thin rule borders, like a woodcut frame around a page of text. The subtle rounding is just enough to avoid pixelation artifacts on low-DPI displays.

7. **Canvas widget color propagation** — all 7 canvas widgets (PriceChart, LagrangeSparkline, EquityCurve, FearGreedGauge, NatalWheel, AstroCalendar, SectorHeatMap) get circadian-aware colors automatically through the semantic function rewrite in Phase 1. Three hardcoded color values in charts.rs were manually updated: tooltip backgrounds now use the palette surface color at 90% opacity, and the equity curve baseline uses palette ink at 30% opacity. Domain-specific colors (astrology wheel gold/blue, zone colors, gauge arc tuples) remain as constants since they represent semantic categories, not ambient theming.
   - *Files:* `src/dashboard/charts.rs` (3 hardcoded colors replaced)
   - *Insight:* The zero-touch propagation works because every canvas widget already called `theme::canvas_bg(theme)` and `theme::fg(theme)` in their `draw()` methods. By rewriting those functions to read from the global palette cache (ignoring the `theme` parameter), all 7 widgets got circadian colors with no code changes to their draw logic.

8. **View-by-view font polish** — applied the four-role typography system across all 8 dashboard tabs. 43+ heading text instances changed from default font to `.font(font::DISPLAY)` (Fraunces). Affected views: Overview (8 headings), Astrology (8), Universe (3 + alerts), Fundamentals (11), Research (12), Portfolio (5). Paper Trail and Settings already used the `section_heading()` helper which was updated in Phase 4.
   - *Files:* All 8 view files in `src/dashboard/view/` (2-12 edits each)
   - *Insight:* The decision to leave numeric values in Source Serif 4 (the default body font) rather than adding explicit `.font(font::INTER)` to hundreds of text widgets was deliberate. Source Serif 4 has well-designed numerals, and serif numerals reinforce the Renaissance book identity. Inter is reserved for tight tabular columns where proportional spacing would misalign.

**Post-upgrade metrics:**

| Metric | v6.2.0 | v7.0.0 |
|--------|--------|--------|
| Theme modes | 4 (Auto/Light/Dark/TokyoNight) | 3 (Auto/Parchment/Leather) |
| Color constants | ~30 (Catppuccin MOCHA_*/LATTE_*) | 8 anchor palettes, 11 channels each |
| Circadian stages | 4 discrete phases | 24 continuous (interpolated) |
| Font families | 1 (Inter + JetBrains Mono) | 4 roles (Fraunces, Source Serif 4, Inter, JetBrains Mono) |
| Font assets | ~120KB (Inter + Mono) | ~1.7MB (+Fraunces 360KB, +Source Serif 4 1.2MB) |
| Canvas widget changes | N/A | 3 hardcoded colors fixed, 7 widgets auto-propagated |
| View heading edits | N/A | 43+ text instances across 8 tabs |
| New state fields | N/A | 1 (circadian_override) |
| New Message variants | N/A | 2 (CircadianSliderChanged, CircadianSliderReset) |
| New crate deps | N/A | 0 (used std::sync::RwLock, no once_cell needed) |
| Tests | 70 | 70 (no new tests; all visual changes) |
| Build | 0 warnings | 0 warnings |

### v6.2.0 — "The Priority Queue" (2026-04-26)

**Theme:** Data reliability for the paper trading engine. Before this change, paper engine positions (up to 20 tickers) and buy candidates (up to 10 more) were invisible to the price fetch pipeline. They only got fresh prices if they happened to overlap with the 10-ticker watchlist or the astro-ranked set. A position could go days without a price update, making stop-loss evaluations and Lagrange scores stale. This version closes that gap by merging paper engine tickers into the priority pipeline so every downstream data source (Alpha Vantage prices, Tiingo history, Finnhub news, sentiment, short interest, EDGAR filings) fetches data for them.

1. **Paper engine priority ticker collection** — new `collect_priority_tickers()` function in `paper_engine.rs` queries two pools: all tickers from `paper_portfolio` (open positions needing daily prices for stop-loss and P&L evaluation) and top Lagrange candidates above the buy threshold (score > 75, not already held, limited to 10). Returns a deduplicated Vec with positions first, then candidates. This function is called before Phase 2 of the scraper pipeline.
   - *Files:* `src/scraper/paper_engine.rs` (+collect_priority_tickers, ~35 lines)
   - *Insight:* Positions are prioritized over candidates because stale prices on an open position mean stop-losses can't fire. A missed stop-loss costs real (simulated) money. A missed candidate just delays a buy by one day.

2. **Pipeline Phase 1.5: priority merge** — after Phase 1 (astrology computation) and before Phase 2 (financial data fetching), the pipeline now calls `paper_engine::collect_priority_tickers()` and merges the results into the existing `priority` vector. Deduplication ensures tickers appearing in both the astro-ranked and paper engine lists aren't fetched twice. The merged priority list flows into ALL Phase 2 steps: price data (2.0), sentiment (2.2), Finnhub (2.3), short interest (2.5), and EDGAR filings (3.4). This means paper engine positions get the same data freshness as the watchlist and astro-favored tickers.
   - *Files:* `src/scraper/main.rs` (+Phase 1.5 block, ~15 lines, modified Phase 2 header)
   - *Insight:* By merging into the existing `priority` vector rather than adding a separate fetch step, paper engine tickers automatically benefit from every data source in Phase 2 and 3 that uses the priority list (sentiment, Finnhub, short interest, EDGAR). One insertion point, six data sources covered.

3. **Tiingo bulk priority upgrade** — the Tiingo price history SQL now includes `paper_portfolio` tickers at tier-0 priority alongside the watchlist. Previously, the ORDER BY clause only prioritized watchlist tickers (tier 0) over the general universe (tier 1). Now both watchlist and paper portfolio tickers share tier 0. The WHERE clause also adds `OR cm.ticker IN (SELECT ticker FROM paper_portfolio)` so paper positions are always fetched even if they already have 26+ price rows (the normal Lagrange activation threshold).
   - *Files:* `src/scraper/tiingo.rs` (modified priority SQL in `fetch_all_prices_tiingo`)
   - *Insight:* The Tiingo fetcher runs in Phase 3 (bulk data), not Phase 2 (targeted), so it has its own ticker selection SQL independent of the `priority` vector. Without this change, paper positions with full price history would be skipped by Tiingo, missing the most recent trading day's OHLCV data.

**Post-upgrade metrics:**

| Metric | v6.1.0 | v6.2.0 |
|--------|--------|--------|
| Priority pipeline tickers | Watchlist (10) + Astro (~10) | + Paper positions (up to 20) + candidates (up to 10) |
| Paper position price freshness | Only if overlapping watchlist/astro | Guaranteed daily via AV + Tiingo |
| Tiingo priority tiers | 2 (watchlist, universe) | 2 (watchlist + paper = tier 0, universe = tier 1) |
| Data sources covering paper tickers | 0-2 (by overlap) | 6 (AV price, Tiingo, sentiment, Finnhub, short interest, EDGAR) |
| New lines | -- | ~55 (collect_priority_tickers ~35, pipeline merge ~15, Tiingo SQL ~5) |
| Tests | 70 | 70 (no new tests needed — DB-dependent queries tested via integration) |

### v6.1.0 — "The Benchmark" (2026-04-26)

**Theme:** SPY benchmark comparison, equity curve visualization, NYSE holiday awareness, and position rebalancing. The Paper Trail tab now answers the most important question in active investing: "did this signal beat the market?" Every metric has a SPY counterpart, the equity curve shows both lines on the same chart, and the simulation engine skips holidays and trims overweight positions.

1. **SPY benchmark metrics in stats card** — the Performance Statistics card now has two rows. The top row shows portfolio metrics (return %, Sharpe, max drawdown, win rate, avg hold, closed trades). A second row, "vs. SPY Benchmark," shows SPY return %, SPY Sharpe, SPY max drawdown, and alpha (portfolio return minus SPY return). Alpha is the single most important number: positive alpha means the Lagrange scoring system outperformed passive index investing. Returns are color-coded (green positive, red negative) for at-a-glance comparison.
   - *Files:* `src/dashboard/view/paper_trail.rs` (modified `build_paper_stats_card`)
   - *Insight:* Alpha = portfolio return % - SPY return % over the same period. This is a simplified Jensen's alpha (not risk-adjusted). For a paper trading simulation, this gives you the answer to "should I follow Lagrange signals or just buy SPY?" without needing CAPM beta estimation.

2. **Equity curve Canvas widget** — new `EquityCurve` struct in `charts.rs` (~200 lines) implementing `canvas::Program<Message>`. Both the paper portfolio and SPY benchmark series are normalized to percentage change from their first value, creating a common 0% origin. Portfolio line drawn in green (2px), SPY in muted gray-blue (1.5px). The 0% baseline is always forced visible via Y-range clamping. Features: 5-step percentage Y-axis labels, title and legend, end-of-line labels showing final return %, and a hover crosshair tooltip displaying day number, portfolio %, SPY %, and alpha at any point. Chart is 220px tall, placed between the stats card and trade log on the Paper Trail tab.
   - *Files:* `src/dashboard/charts.rs` (+EquityCurve, +normalize_to_pct helper), `src/dashboard/view/paper_trail.rs` (+build_equity_curve_card, +Canvas import)
   - *Insight:* Normalizing to percentage change is essential for dual-series comparison. Raw values ($100K portfolio vs $500 SPY) are incomparable on the same axis. With %-change normalization, the visual gap between the lines IS the alpha at every point in time.

3. **NYSE trading calendar / holiday awareness** — the paper engine now skips simulation on weekends and NYSE market holidays. The `is_nyse_holiday()` function covers all 9 regular NYSE holidays: New Year's Day, MLK Day, Presidents' Day, Memorial Day, Juneteenth (2022+), Independence Day, Labor Day, Thanksgiving, and Christmas. Fixed holidays use observed-date shifting (Saturday -> Friday, Sunday -> Monday). Floating holidays use the `nth_weekday(month, day_of_week, occurrence)` pattern to compute dates like "3rd Monday of January." 5 unit tests verify against known 2025 NYSE holidays, observed-date shifting, Juneteenth pre-2022 exclusion, and Thanksgiving 2026.
   - *Files:* `src/scraper/paper_engine.rs` (+is_nyse_holiday, +weekend guard, +5 tests)
   - *Insight:* Floating holidays are computed from first principles rather than stored as a lookup table. The `nth_weekday` helper finds the first occurrence of a weekday in a month (via `rem_euclid(7)` modular arithmetic), then adds `(n-1) * 7` days. This means the calendar is correct for any year without maintaining a static list.

4. **Position sizing drift rebalancing** — after signal-driven buys and sells, the engine now runs a rebalance pass. It computes each position's current market value and the equal-weight target (total portfolio value / number of positions). If any position exceeds the target by more than 25% (`REBALANCE_DRIFT = 0.25`), the excess shares are sold back to cash at current price. Only sell-side rebalancing is performed (trimming winners), not buy-side (the freed cash naturally funds next cycle's new positions). Rebalance trades are logged with `score = NULL` to distinguish them from signal-driven trades. This prevents a single runaway position from dominating the portfolio and distorting Sharpe ratio measurement.
   - *Files:* `src/scraper/paper_engine.rs` (+REBALANCE_DRIFT constant, +rebalance_positions function ~80 lines)
   - *Insight:* Sell-only rebalancing is deliberately asymmetric. Buying up undersized positions would require knowing their current Lagrange score (are they undersized because the signal weakened?). By only trimming overweight winners, the rebalancer respects the signal's buy/sell logic while correcting pure sizing drift from price appreciation.

5. **Hard stop-loss and trailing stop** — two price-based risk management mechanisms added to the position evaluation loop, checked before signal-based sell logic. Hard stop: if current price drops 15% below entry price (`HARD_STOP_PCT = 0.15`), force sell immediately regardless of Lagrange score. Trailing stop: if current price drops 20% below the position's peak closing price since entry (`TRAILING_STOP_PCT = 0.20`), force sell. The peak price is queried via `MAX(close) FROM price_data WHERE ticker = $1 AND date >= entry_date`, requiring no additional state storage. Priority order in the evaluation loop: hard stop > trailing stop > signal sell > zombie sell. This ensures catastrophic losses are caught even when Lagrange scores haven't updated yet.
   - *Files:* `src/scraper/paper_engine.rs` (+2 constants, +peak_close field on PositionEval, +entry_date in position query, +stop-loss checks in evaluation loop, +peak query in evaluate_position)
   - *Insight:* Hard stop and trailing stop complement each other. Hard stop caps the maximum loss per position at -15% from cost basis (protects against sudden crashes). Trailing stop locks in gains by selling when the price retraces 20% from its high (protects against slow reversals after a run-up). Neither requires Lagrange score data, so they work even when the scoring pipeline is delayed.

**Post-upgrade metrics:**

| Metric | v6.0.0 | v6.1.0 |
|--------|--------|--------|
| Paper Trail cards | 4 | 5 (+Equity Curve) |
| SPY benchmark | Data fetched, unused | Full comparison: return, Sharpe, drawdown, alpha |
| Chart widgets | 2 (PriceChart, Sparkline) | 3 (+EquityCurve) |
| NYSE holidays | None | 9 holidays, weekend guard, observed-date shifting |
| Rebalancing | None (drift accepted) | 25% drift threshold, sell-only trim |
| Stop-loss | None | Hard stop -15% + trailing stop -20% from peak |
| New lines | -- | ~500 (chart ~200, holidays ~70, rebalance ~80, stops ~50, stats ~100) |
| Tests | 65 | 70 (+5 NYSE holiday tests) |

### v6.0.0 — "The Paper Trail" (2026-04-26)

**Theme:** Autonomous paper trading simulation engine. The scraper's daily pipeline now includes a Phase 5 that simulates buy/sell decisions based on Lagrange composite scores (the same blend of astrology + quant signals that powers the Universe Explorer). A new "Paper Trail" dashboard tab surfaces the simulation results: account summary, open positions with live P&L, performance statistics (Sharpe ratio, max drawdown, win rate), and a full trade log. This turns the scoring system from a passive display into an actionable, backtestable signal.

1. **Paper trading engine (scraper Phase 5)** — new `paper_engine.rs` module (~280 lines) runs as the final phase of the daily scraper pipeline (after Lagrange score computation). The engine evaluates all open positions and finds new buy candidates using Lagrange scores. Buy threshold: score > 75 (strong bullish signal). Sell threshold: score < 40 (signal deterioration). Hold zone: 40-75 (no action). Position sizing uses equal-weight allocation: available cash divided by min(candidates, MAX_POSITIONS - current_positions), capped at 20 simultaneous positions and 10 new buys per day. Entry prices use the most recent closing price from `price_data`. Zombie position detection: force-sells any position where the ticker hasn't received a Lagrange score update in 10+ trading days (stale data = unreliable signal). Idempotency guard: `last_sim_date` column on `paper_account` prevents double-execution if the scraper runs twice. Buys use `ON CONFLICT (ticker) DO UPDATE` to merge into existing positions with weighted-average entry prices.
   - *Files:* `src/scraper/paper_engine.rs` (new, ~280 lines), `src/scraper/main.rs` (+mod, +Phase 5 block)
   - *Insight:* The engine deliberately uses closing prices rather than intraday prices. Since Lagrange scores are computed from daily data (astro transits + daily closes + macro indicators), simulating at the close-to-close level matches the signal's actual time resolution. Using intraday prices would create false precision.

2. **Three paper trading migrations** — `paper_account` (singleton row with $100,000 initial capital, cash balance, last sim date), `paper_portfolio` (open positions with UNIQUE(ticker) constraint for upsert support), and `paper_trades` (full trade log with CHECK constraint limiting action to 'BUY'/'SELL'). Indexes on paper_trades.ticker and paper_trades.trade_date for the dashboard's most-recent-first query pattern. The account table seeds itself with a conditional INSERT (only if no row exists) so repeated migration runs are safe.
   - *Files:* `migrations/0033_paper_account.sql`, `migrations/0034_paper_portfolio.sql`, `migrations/0035_paper_trades.sql`
   - *Insight:* The `UNIQUE(ticker)` constraint on `paper_portfolio` is load-bearing. It enables `ON CONFLICT (ticker) DO UPDATE SET shares = shares + excluded.shares, entry_price = (entry_price * shares + excluded.entry_price * excluded.shares) / (shares + excluded.shares)` for weighted-average cost basis when adding to an existing position. Without it, the engine would create duplicate rows.

3. **Strategy engine extensions** — added `LagrangeAbove(f64)` and `LagrangeBelow(f64)` conditions to the existing `Condition` enum in the strategy builder. `DaySnapshot` gained a `lagrange_score: Option<f64>` field. Both conditions return false when the score is `None` (missing data = no signal = no action). This lets the user-facing strategy builder reference the same thresholds the paper engine uses internally. 3 new unit tests cover above/below/missing-score paths.
   - *Files:* `src/dashboard/strategy.rs` (+2 enum variants, +DaySnapshot field, +check/label impls, +3 tests), `src/dashboard/update/portfolio.rs` (+lagrange_score: None in DaySnapshot construction)

4. **Shared statistics module** — new `src/stats.rs` library module (re-exported from `lib.rs`) with four pure functions: `sharpe_ratio()` (annualized from log returns, 252 trading days, returns 0.0 for flat/insufficient data), `max_drawdown_pct()` (peak-to-trough percentage), `win_rate_pct()` (percentage of positive returns), and `avg_holding_days()` (mean duration from buy-sell date pairs). All functions are designed for the paper trail dashboard but live in the shared library crate so the scraper can use them later for logging. 9 unit tests covering edge cases (empty input, flat series, no drawdown, zero trades).
   - *Files:* `src/stats.rs` (new, ~120 lines), `src/lib.rs` (+pub mod stats)
   - *Insight:* Sharpe ratio uses log returns (`ln(v[i]/v[i-1])`) rather than simple returns because log returns are additive across time periods, which makes the annualization factor (`sqrt(252)`) mathematically correct. Simple returns require geometric compounding for annualization.

5. **Paper Trail dashboard tab (8th tab)** — new tab between Portfolio and Settings (Ctrl+7 shortcut, Settings moves to Ctrl+8). Four cards: Account Summary (initial capital, cash balance, portfolio value, total value, total return %, last sim date, trade count), Open Positions (table with ticker, shares, entry price, current price, P&L%, entry/current Lagrange scores, entry date), Performance Statistics (Sharpe ratio, max drawdown, win rate, avg holding days, closed trade count), and Trade Log (most recent 50 trades with date, action, ticker, shares, price, score). Color-coded throughout: green for gains/buys, red for losses/sells. Empty states guide the user ("Run the scraper to initialize", "The paper engine runs automatically as Phase 5"). Stats card computes win rate and avg holding by matching BUY/SELL pairs from the trade log in-memory.
   - *Files:* `src/dashboard/view/paper_trail.rs` (new, ~285 lines), `src/dashboard/view/mod.rs` (+mod, +dispatch, +subtitle), `src/dashboard/tabs.rs` (+PaperTrail variant, all()/label()/icon())
   - *Insight:* The conditional rendering pattern (`if has_data { row![...].into() } else { text(...).into() }`) inside Iced's `column![]` macro causes type inference failures because the compiler can't resolve the `Theme` parameter across divergent branches. The fix is to pull the conditional into a `let stats_body: Element<'_, Message> = if ... { } else { };` binding, giving the compiler a concrete type anchor.

6. **Paper trading DB queries** — new `src/dashboard/db/paper.rs` module with 4 async fetch functions. `fetch_paper_account()` uses a subquery to compute live portfolio value (`SUM(shares * latest_close)` via LATERAL JOIN) and total trade count. `fetch_paper_positions()` joins each position with its most recent close price and Lagrange score via LATERAL JOINs for live P&L display. `fetch_paper_trades()` returns most recent 200 trades ordered by date DESC. `fetch_paper_daily_values()` reconstructs a portfolio value time series from trade dates crossed with position snapshots (simplified; full daily reconstruction deferred to a future `paper_snapshots` table), plus a parallel SPY benchmark series for relative performance.
   - *Files:* `src/dashboard/db/paper.rs` (new, ~155 lines), `src/dashboard/db/mod.rs` (+pub mod paper)

7. **Dashboard state wiring** — 5 new state fields (`paper_account`, `paper_positions`, `paper_trades`, `paper_daily_values`, `paper_spy_values`), 4 new message variants (`PaperAccountLoaded`, `PaperPositionsLoaded`, `PaperTradesLoaded`, `PaperValuesLoaded`), 4 fetch tasks added to the `TickersLoaded` batch, and 8 message handlers (Ok/Err pairs) in the update dispatcher. All paper data loads in parallel with the existing 20+ initial queries on startup.
   - *Files:* `src/dashboard/state.rs` (+5 fields, +4 messages), `src/dashboard/update/mod.rs` (+imports, +4 fetch tasks, +8 handlers), `src/dashboard/update/helpers.rs` (+Ctrl+7/Ctrl+8 remap)

**Post-upgrade metrics:**

| Metric | v5.0.0 | v6.0.0 |
|--------|--------|--------|
| Dashboard tabs | 7 | 8 (+Paper Trail) |
| Scraper phases | 4 | 5 (+Paper Engine) |
| New files | 0 | 5 (paper_engine.rs, stats.rs, paper_trail.rs, db/paper.rs, 3 migrations) |
| New Message variants | -- | +4 (PaperAccountLoaded, PaperPositionsLoaded, PaperTradesLoaded, PaperValuesLoaded) |
| New state fields | -- | +5 (paper_account, paper_positions, paper_trades, paper_daily_values, paper_spy_values) |
| New lines | -- | ~850 (engine ~280, stats ~120, view ~285, db ~155) |
| New crate deps | 0 | 0 |
| Migrations | 0 | 3 (paper_account, paper_portfolio, paper_trades) |
| Tests | 52 | 65 (48 lib + 17 dashboard, all pass) |
| Strategy conditions | 7 | 9 (+LagrangeAbove, +LagrangeBelow) |

### v5.0.0 — "The Council" (2026-04-25)

**Theme:** LLM-backed agent analysis, single-ticker data fetch from the dashboard, and compiler warnings cleanup. The four investment personas (Buffett, Graham, Lynch, Munger) can now consult Claude via the Anthropic Messages API for live, context-aware analysis alongside their existing template-based reasoning. Users can fetch fresh data for any ticker without leaving the dashboard.

1. **Compiler warnings cleanup (47 -> 0)** — eliminated all 47 compiler warnings across both binaries. Design-system files (`icons.rs`, `theme.rs`) use module-level `#![allow(dead_code)]` since their unused constants form a curated palette for incremental adoption. Individual items (`font.rs:MONO`, `greeks.rs:OptionType::label()`, `state.rs:UniverseSortCol::label()`, `shared.rs:section_heading/titled_card`) use item-level `#[allow(dead_code)]`. Removed truly dead imports in `view/overview.rs` (unused `crate::icons` and `titled_card`).
   - *Files:* `src/dashboard/icons.rs`, `src/dashboard/theme.rs`, `src/dashboard/font.rs`, `src/dashboard/greeks.rs`, `src/dashboard/state.rs`, `src/dashboard/view/shared.rs`, `src/dashboard/view/overview.rs`
   - *Insight:* Module-level `#![allow(dead_code)]` (with `!`) suppresses warnings for all items in the file, while item-level `#[allow(dead_code)]` (without `!`) only suppresses the single item it annotates. The distinction matters for design-system files where you intentionally define more constants than currently referenced.

2. **"Fetch this ticker" button** — users can now fetch fresh data for the selected ticker directly from the dashboard. The dashboard locates the scraper binary adjacent to its own executable (via `std::env::current_exe().parent()`) and spawns it as a subprocess with `--ticker AAPL` using `tokio::process::Command`. The scraper's new `--ticker` CLI mode runs a focused 6-phase pipeline: (1) astrology seed + transits + scores, (2) price data via AlphaVantage, (3) Finnhub news + recommendations, (4) sentiment analysis, (5) FMP fundamentals, (6) Lagrange score recomputation. On completion, the dashboard auto-refreshes from the DB. Button appears in both the main header (next to Refresh) and the empty-fundamentals state. Disabled with "Fetching..." text while the subprocess runs. Toast notifications provide progress feedback.
   - *Files:* `src/scraper/main.rs` (+CLI parsing, +fetch_single_ticker ~60 lines), `src/scraper/prices.rs` (+pub(crate)), `src/scraper/fundamentals.rs` (+pub(crate)), `src/scraper/finnhub.rs` (+2x pub(crate)), `src/dashboard/state.rs` (+2 fields, +2 messages), `src/dashboard/update/data.rs` (+FetchThisTicker/FetchTickerComplete handlers), `src/dashboard/view/mod.rs` (+fetch button in header), `src/dashboard/view/fundamentals.rs` (+fetch button in empty state)
   - *Insight:* Subprocess spawning via `tokio::process::Command` is simpler than restructuring the scraper as a library. The two binaries share no mutable state, communicate only through the PostgreSQL database, and `std::env::current_exe().parent()` reliably finds the sibling binary in both debug and release builds. Windows gets `.exe` suffix via `cfg!(windows)`.

3. **LLM-backed agent analysis via Anthropic Claude API** — the four investment personas can now generate live analysis by calling the Anthropic Messages API (`api.anthropic.com/v1/messages`) using `claude-sonnet-4-20250514`. Each persona gets a tailored system prompt encoding their investment philosophy (Buffett: moat + FCF + margin of safety, Graham: deep value + P/B < 1.5, Lynch: PEG ratio + "know what you own", Munger: mental models + quality). The system prompt includes all available financial context (price, astro score, Lagrange score, concordance, moon phase, mercury retrograde, and 20+ fundamental metrics from AgentContext). Claude returns structured JSON parsed into the existing `AgentAnalysis` struct (headline, analysis, verdict, key_metrics, astro_take). Response parsing uses a dual-path strategy: direct JSON parse, then fallback extraction of JSON from markdown-wrapped responses via `find('{')/rfind('}')`.
   - *Files:* `src/dashboard/agents.rs` (+AgentMode enum, +build_system_prompt, +format_context_for_llm, +analyze_llm, +parse_llm_response ~200 lines)
   - *Insight:* LLMs often wrap JSON in markdown code fences even when instructed not to. The dual-path parser handles both clean JSON and ````json ... ``` `` wrapped responses. Using `claude-sonnet` (fast, cheap) rather than opus keeps response time under 3 seconds and cost under $0.01 per analysis call.

4. **Agent mode toggle + state machine** — the Fundamentals tab now shows an "Analysis Mode: [Template] [LLM]" toggle above the persona buttons. Template mode runs the existing deterministic template analysis synchronously. LLM mode spawns an async `Task::perform` that calls the Anthropic API and delivers results via `LlmAnalysisComplete`. Three-state UI: idle (no persona selected), loading ("Consulting the council... Buffett is thinking..."), and results with a mode badge ("(LLM)" or "(Template)"). Graceful fallback chain: no API key -> template with toast, API error -> template with error toast. Agent context extraction refactored into `build_agent_context()` shared by both paths. Agent mode persists in the settings table and loads on startup.
   - *Files:* `src/dashboard/state.rs` (+4 fields: agent_mode/agent_loading/agent_llm_error/api_key_input, +3 messages: SetAgentMode/LlmAnalysisComplete/ApiKeyInput), `src/dashboard/update/data.rs` (+branched AgentSelected handler, +SetAgentMode/LlmAnalysisComplete/ApiKeyInput handlers), `src/dashboard/update/helpers.rs` (+build_agent_context extracted from recompute_agent_if_active), `src/dashboard/update/mod.rs` (+agent_mode in SettingsLoaded), `src/dashboard/view/fundamentals.rs` (+mode toggle, +loading state, +mode badge, +error display)

5. **API key management in Settings** — new "API Keys" card in the Settings tab with a text input for the Anthropic API key. The key is persisted via the existing `upsert_setting()` mechanism (no migration needed). Current key is displayed masked (first 4 + last 4 characters) for verification without full exposure. Help text explains the key's purpose and which model is used. New `KEY` icon added to the Bootstrap Icons palette.
   - *Files:* `src/dashboard/view/settings.rs` (+API Keys card), `src/dashboard/icons.rs` (+KEY codepoint)

**Post-upgrade metrics:**

| Metric | v4.2.0 | v5.0.0 |
|--------|--------|--------|
| Compiler warnings | 47 | 0 |
| New Message variants | -- | +5 (FetchThisTicker, FetchTickerComplete, SetAgentMode, LlmAnalysisComplete, ApiKeyInput) |
| New state fields | -- | +6 (fetching_ticker, fetch_ticker_error, agent_mode, agent_loading, agent_llm_error, api_key_input) |
| Agent modes | Template only | Template + LLM (Claude Sonnet) |
| Ticker fetch | Scraper CLI only | Dashboard button + scraper CLI |
| New files | 0 | 0 |
| New lines | -- | ~400 |
| New crate deps | 0 | 0 |
| Tests | 52 | 52 (all pass) |

### v4.2.0 — "The Expansion" (2026-04-24)

**Theme:** New analytical features. Candlestick charts, Black-Scholes options calculator, server-side sortable tables, in-app toast notifications, GDELT geopolitical events, and astro calendar (already present from v3.0.5).

1. **Candlestick charts replacing area fill** — rewrote the price chart Canvas from an area-fill line chart to proper OHLC candlestick bars. Each bar has a thin wick line (high to low) and a filled body rectangle (open to close). Green for bullish candles (close >= open), red for bearish, using `theme::bullish()` and `theme::bearish()` (Catppuccin green/red). Bar width auto-scales with data count, clamped to 2..12px. Price range calculation expanded to include high/low values (not just close), preventing wick clipping. Doji candles get a minimum 1px body height. Volume bars, SMA overlays, Bollinger Bands, and astro markers all remain functional.
   - *Files:* `src/dashboard/charts.rs` (candlestick rendering replaces area fill)
   - *Insight:* `rust_decimal::Decimal` doesn't implement `Into<f32>`, so all OHLC values use `.to_string().parse::<f32>()` for conversion. Not elegant, but zero-allocation overhead at 100 data points.

2. **Black-Scholes Options Greeks calculator** — new `greeks.rs` module (~200 lines) implementing the complete Black-Scholes model. Computes call and put prices plus 5 Greeks: delta, gamma, theta (per calendar day), vega (per 1% vol), and rho (per 1% rate). Standard normal CDF uses the Abramowitz & Stegun polynomial approximation (max error ~1.5e-7) via erf identity. Implied volatility solver uses Newton-Raphson iteration (50 max iterations, 1e-8 tolerance) with Brenner-Subrahmanyam initial guess `sigma_0 = sqrt(2pi/T) * price/spot`. UI in Fundamentals tab: input row for spot/strike/days/rate/vol/type, compute button, IV solver row, and colored results display (delta by magnitude, theta in red for time decay). Auto-fills spot price from current ticker data when empty.
   - *Files:* `src/dashboard/greeks.rs` (new), `src/dashboard/main.rs` (+mod), `src/dashboard/state.rs` (+8 input fields, +2 result fields, +9 messages), `src/dashboard/update/data.rs` (+9 handlers), `src/dashboard/update/helpers.rs` (+compute_greeks, +solve_implied_vol), `src/dashboard/view/fundamentals.rs` (+Greeks section)
   - *Tests:* 5 unit tests: call price sanity, put-call parity, IV roundtrip, ATM delta near 0.5, degenerate input handling

3. **Server-side sortable Universe table** — Universe Explorer column headers are now clickable sort buttons. `UniverseSortCol` enum with 6 variants (Ticker, AstroScore, LagrangeScore, FinSub, MacroSub, ShortSub), each mapping to a SQL expression via `sql_expr()`. Click toggles ascending/descending, resets to page 0, and re-fetches with dynamic `ORDER BY`. Column headers show active sort indicator (triangle-up or triangle-down). Server-side sort is critical here: 1,700+ tickers across 35 pages, client-side sort on a 50-row page would be meaningless.
   - *Files:* `src/dashboard/state.rs` (+UniverseSortCol enum, +sort fields, +Message::UniverseSort), `src/dashboard/update/universe.rs` (+sort handler), `src/dashboard/db/universe.rs` (+sort_col/sort_asc params), `src/dashboard/view/universe.rs` (+clickable headers with indicators)

4. **In-app toast notifications** — replaced silent operations with visual feedback. Toasts are dark semi-transparent pills (rgba 0.1, 0.1, 0.15, 0.92) rendered as a right-aligned overlay at the top of the content area. Each toast auto-expires after 4 seconds, cleaned up on the existing 30-second `Tick` subscription. Max 5 visible toasts, oldest drops first. Applied to: clipboard copy ("Copied to clipboard"), settings save ("Setting saved"), portfolio transaction creation, and more.
   - *Files:* `src/dashboard/state.rs` (+toasts Vec), `src/dashboard/update/helpers.rs` (+push_toast, +expire_toasts), `src/dashboard/update/mod.rs` (+expire on Tick, +toast on CopyText/SettingSaved), `src/dashboard/update/portfolio.rs` (+toast on TxCreated), `src/dashboard/view/mod.rs` (+toast overlay rendering)

5. **GDELT geopolitical events** — new scraper module and Research tab section for the GDELT 2.0 DOC API. Five query categories: trade/sanctions, monetary policy, military conflict, political instability, and energy/OPEC. Each category fetches up to 25 articles from the past 24 hours (English only). Articles stored with URL deduplication (`ON CONFLICT (url) DO NOTHING`). GDELT tone scores (-10 to +10) displayed with color coding: green for positive, gray for neutral, red for negative. Research tab shows title, source country, tone, domain, and timestamp in a 160px scrollable list.
   - *Files:* `src/scraper/gdelt.rs` (new, ~160 lines), `src/scraper/main.rs` (+mod, +pipeline step), `migrations/0032_gdelt_events.sql` (new), `src/models.rs` (+GdeltEvent struct), `src/dashboard/db/ticker_data.rs` (+fetch_gdelt), `src/dashboard/state.rs` (+gdelt_events, +GdeltLoaded), `src/dashboard/update/data.rs` (+handler), `src/dashboard/update/mod.rs` (+initial fetch), `src/dashboard/view/research.rs` (+GDELT section)

6. **Astro calendar (confirmed present)** — the monthly calendar Canvas widget showing astro-score-colored day cells was already implemented in v3.0.5. Each day cell is colored on a green (favorable) to red (unfavorable) gradient based on the ticker's daily astro score. Day-of-week headers, score labels in cells, and month/year title. Fully wired: `calendar.rs` widget, `fetch_astro_calendar()` DB function, `CalendarLoaded` message handler, displayed in the Astrology tab.
   - *Files:* `src/dashboard/calendar.rs` (existing), `src/dashboard/view/astrology_tab.rs` (existing integration)

**Post-upgrade metrics:**

| Metric | v4.1.0 | v4.2.0 |
|--------|--------|--------|
| New files | -- | 3 (greeks.rs, gdelt.rs scraper, migration) |
| Chart type | Area fill | OHLC Candlestick |
| Options analytics | None | Full Black-Scholes + IV solver |
| Table sorting | None | 6-column server-side sort |
| User feedback | Silent operations | Toast notifications |
| Geopolitical data | None | GDELT 5-category feed |
| Tests | 47 | 52 (38 lib + 14 dashboard) |

### v4.1.0 — "The Glass" (2026-04-23)

**Theme:** UI/UX overhaul. Custom typography, icon system, Catppuccin theme palette, redesigned tab bar, card layout system, and polished numeric formatting. The dashboard goes from functional to professional.

1. **Typography foundation (Inter + JetBrains Mono)** — embedded three font files at compile time via `include_bytes!`. Inter Regular (body text, labels), Inter SemiBold (section headings), and JetBrains Mono (future: numeric columns). New `font.rs` module defines `Font` constants (`INTER`, `INTER_BOLD`, `MONO`). All three registered in `main.rs` via `.font()` on the Iced application builder. Zero runtime font loading, zero external file dependencies.
   - *Files:* `src/dashboard/font.rs` (new), `src/dashboard/main.rs` (+mod font, +.font() calls), `assets/fonts/{Inter-Regular,Inter-SemiBold,JetBrainsMono-Regular}.ttf` (new)

2. **Bootstrap icon system** — added `iced_fonts` 0.3.0 crate with `bootstrap` feature, providing 2,048 Bootstrap Icons as an embedded TTF. Created `icons.rs` module that re-exports the font bytes and defines our own `Font` constant (avoiding iced_core 0.13/0.14 type mismatch). 35 icon codepoint constants defined for tabs (stars, speedometer, globe, bar-chart, newspaper, briefcase, gear), actions (search, refresh, download, filter), navigation (chevrons), indicators (arrows, carets, sort), and status (bell, clock, info, warning). Helper function `icon(codepoint, size)` returns iced 0.13-compatible `Text` elements.
   - *Files:* `src/dashboard/icons.rs` (new), `src/dashboard/main.rs` (+mod icons, +.font(BOOTSTRAP_BYTES)), `Cargo.toml` (+iced_fonts)

3. **Catppuccin + TokyoNight theme upgrade** — replaced manual theme color definitions with Iced 0.13's built-in `CatppuccinMocha`, `CatppuccinLatte`, and `TokyoNight` themes. `ThemeMode` expanded from 3 to 4 variants (Auto/Latte/Mocha/Tokyo). Auto mode uses circadian phase detection (Dawn/Day=Latte, Dusk/Night=Mocha). `is_dark()` rewritten to use background luminance check `(r+g+b)/3 < 0.5` instead of enum comparison, making it work with any custom theme. Added 23 Catppuccin palette constants (17 Mocha, 6 Latte) for canvas widgets. All semantic color functions (`fg()`, `surface()`, `accent()`, `bullish()`, etc.) updated to adapt to active theme.
   - *Files:* `src/dashboard/theme.rs` (major update), `src/dashboard/update/mod.rs` (TokyoNight parsing in 2 match blocks), `src/dashboard/view/settings.rs` (Tokyo button)

4. **Tab bar redesign (icon + label + underline)** — replaced plain text tab buttons with icon + label pairs. Each tab shows its Bootstrap icon alongside the label name. Active tab gets a visual underline via a 2px `container` with `bordered_box` style. Tab icon mapping added to `tabs.rs` via `Tab::icon()` method. Refresh button also updated with icon + label layout.
   - *Files:* `src/dashboard/tabs.rs` (+icon method), `src/dashboard/view/mod.rs` (tab bar + refresh redesign)

5. **Card/panel layout system** — created reusable `card()`, `section_heading()`, and `titled_card()` helpers in `view/shared.rs`. `card()` wraps content in a `container` with `rounded_box` style (theme-adaptive background color with 2px rounded corners) and 12px padding. `section_heading()` renders icon + bold title. Applied across Settings tab (4 cards: Appearance, Data, Alerts, Info) and Overview tab (gauges, signal intelligence, scored universe, polymarket sections wrapped in cards).
   - *Files:* `src/dashboard/view/shared.rs` (+card helpers), `view/settings.rs` (full card redesign), `view/overview.rs` (card wrapping)

6. **Numeric formatting polish** — new formatting functions in `helpers.rs`: `format_price()` produces comma-grouped prices ($1,234.56), `format_pct()` adds sign prefix (+12.3%, -4.5%), `format_compact()` abbreviates large numbers (1.2B, 345M, 12.3K). Applied across Overview tab (SMA prices) and Portfolio tab (transaction prices, cost basis, P&L totals, percentage returns). Handles NaN/Infinity gracefully with "—" fallback.
   - *Files:* `src/dashboard/helpers.rs` (+format_price, +format_pct, +format_compact, +comma_group), `view/overview.rs`, `view/portfolio_tab.rs`

7. **Swiss Ephemeris test serialization** — the Swiss Ephemeris C library has global mutable state that corrupts under parallel test execution. Added a `pub(crate) SWE_TEST_LOCK: Mutex<()>` in `swisseph_bridge.rs`, acquired by all 11 Swiss Ephemeris-touching tests (7 in swisseph_bridge, 4 in natal). Tests now pass reliably under parallel execution. 47/47 stable across multiple runs.
   - *Files:* `src/astrology/swisseph_bridge.rs` (+SWE_TEST_LOCK), `src/astrology/natal.rs` (+lock guards in 4 tests)

**Post-upgrade metrics:**

| Metric | v4.0.0 | v4.1.0 |
|--------|--------|--------|
| New files | -- | 3 (font.rs, icons.rs, 3 TTFs) |
| New crate deps | -- | `iced_fonts` 0.3.0 |
| Theme modes | 3 (Auto/Light/Dark) | 4 (+TokyoNight) |
| Icon codepoints | 0 | 35 |
| Card-wrapped sections | 0 | 8+ (Settings 4, Overview 4) |
| Font families | 1 (system default) | 3 (Inter, Inter SemiBold, JetBrains Mono) |
| Test stability | Flaky (SWE race) | 47/47 stable |

### v4.0.0 — "Structural Steel" (2026-04-23)

**Theme:** Tech debt reduction. No visual changes. Dashboard behaves identically before and after. Codebase reduced by ~726 net lines, all 47 tests passing.

1. **Split update/mod.rs (921 -> 315 lines)** -- the monolithic 921-line match block that handled every message variant was the biggest maintenance bottleneck. Split into 5 domain-focused files: `update/astro.rs` (astro score, horoscope, natal, retrogrades), `update/data.rs` (ticker selection, price/news/fundamentals loading), `update/universe.rs` (universe pagination, alerts, search, export), `update/portfolio.rs` (portfolio, backtest, DCF, watchlist, transactions). The `mod.rs` dispatch file is now 315 lines, routing each message to its handler.
   - *Files:* `src/dashboard/update/{astro,data,universe,portfolio}.rs` (new), `update/mod.rs` (trimmed), `update/helpers.rs` (moved shared helpers)

2. **Unwrap audit (44 -> ~8 remaining)** -- replaced unsafe `.unwrap()` calls with proper error handling across the astrology engine and agent modules. `swisseph_bridge.rs` (6 unwraps removed: FFI returns wrapped in `Option`/`Result`), `interpretation.rs` (8 removed: `.unwrap_or_default()` and format fallbacks), `natal.rs` (5 removed: `?` operator chains), `aspects.rs` (4 removed), `agents.rs` (4 removed). Remaining unwraps are justified (`Mutex::lock`, test assertions).
   - *Files:* `src/astrology/{swisseph_bridge,interpretation,natal,aspects}.rs`, `src/dashboard/agents.rs`

3. **Agent code deduplication (905 -> 651 lines)** -- the 4 agent personas (Buffett, Graham, Lynch, Munger) shared ~60% identical structure. Extracted three shared helpers: `eval_metric()` replaces duplicated if/else threshold cascades with data-driven tier arrays `&[(f64, bool, &str, i32)]`; `score_to_verdict()` maps accumulated scores to `AgentVerdict` enums; `assemble_analysis()` handles the common tail pattern of building the final `AgentAnalysis`. Each persona retains its unique philosophy, thresholds, and narrative voice.
   - *Files:* `src/dashboard/agents.rs` (254 lines removed, 3 helpers added)

4. **Hardcoded watchlist extraction to DB** -- moved `WATCHLIST` (10 tickers), `CIK_MAP`, `CUSIP_MAP`, and `INSTITUTION_MAP` from const arrays in `scraper/main.rs` to DB-backed config. New migration `0031_scraper_config.sql` creates `scraper_watchlist` and `scraper_institutions` tables, seeded with the 10 default tickers and 4 institutions. Scraper loads from DB at startup via `init_config()`, using `OnceLock` + `Box::leak` to produce `&'static [&'static str]` slices for zero-blast-radius compatibility with all 11 existing call sites. Falls back to compiled defaults when DB is empty.
   - *Files:* `src/scraper/main.rs` (+init_config, +OnceLock statics, renamed consts), `migrations/0031_scraper_config.sql` (new), 11 scraper modules (mechanical rename `crate::WATCHLIST` -> `crate::watchlist()`)

5. **Error handling consistency (DashError + SqlResultExt)** -- created `src/dashboard/error.rs` with a `DashError` enum (Clone-safe for Iced's Message constraint) and `SqlResultExt` trait providing `.ctx("function_name")` as a drop-in replacement for `.map_err(|e| e.to_string())`. Applied across all 5 `db/` modules (~50 call sites). Every DB error now carries the originating function name: `[fetch_prices] relation price_data does not exist` instead of a raw sqlx message.
   - *Files:* `src/dashboard/error.rs` (new), `src/dashboard/main.rs` (+mod error), `src/dashboard/db/{mod,ticker_data,astro,universe,portfolio}.rs` (~50 `.map_err` -> `.ctx()`)

6. **Swiss Ephemeris NaN fix** -- the Swiss Ephemeris C library has global mutable state. Mixing flag sets (file-based ephemeris first, then Moshier fallback) corrupted internal state, causing TrueNode (NorthNode) to return NaN and subsequent calls to return inconsistent positions. Fixed by using Moshier analytical ephemeris as the primary flag set for all calls (sub-arcminute accuracy, built-in, no external `.se1` files needed). Added NaN guard as defense-in-depth. All 38 lib tests + 9 dashboard tests now pass (was 37/38).
   - *Files:* `src/astrology/swisseph_bridge.rs` (Moshier primary, NaN guard, updated docs)

**Post-refactor metrics:**

| Metric | Before | After |
|--------|--------|-------|
| Total Rust lines | 17,851 | 17,986 |
| `update/mod.rs` | 921 lines | 315 lines (+ 4 domain files) |
| `agents.rs` | 905 lines | 651 lines |
| `.unwrap()` (non-test) | 44 | ~25 |
| Test results | 37/38 pass | 47/47 pass |
| New files | -- | 6 (error.rs, 4 update/, migration) |

### v3.1.9 — "New Tools" (2026-04-23)

**Theme:** Feature additions building on the polished v3.1.8 base. New chart overlays, alert management, CSV export, portfolio import, and peer comparison.

1. **News sentiment coloring** — news headlines in the Research tab now carry color-coded sentiment badges. "Bullish" (green) and "Bearish" (red) labels appear next to the section header based on the ticker's latest sentiment score. Headlines from sentiment-scored RSS feeds are tinted accordingly.
   - *Files:* `view/research.rs` (sentiment badge from `self.sentiment`, conditional headline color)

2. **Export Universe CSV** — "Export CSV" button on the Universe tab exports the current filtered/searched universe view to a CSV file via native file dialog. Includes all 10 columns: ticker, company, sector, score, label, astro, fin, macro, short, concordance.
   - *Files:* `state.rs` (+ExportUniverseCsv message), `update/mod.rs` (+handler dispatching to helper), `update/helpers.rs` (+export_universe_csv async fn), `view/universe.rs` (+button)

3. **Portfolio import from watchlist** — "Import to Portfolio" button in the Portfolio tab's watchlist manager inserts all tickers from the active named watchlist as portfolio positions (1 share at $0 avg cost). Uses `INSERT ... ON CONFLICT DO NOTHING` to avoid duplicates.
   - *Files:* `state.rs` (+ImportWatchlistToPortfolio message), `update/mod.rs` (+handler), `db/portfolio.rs` (+import_tickers_to_portfolio fn), `view/portfolio_tab.rs` (+button)

4. **Comparative auto-suggest peers** — the Fundamentals tab now shows up to 6 sector peers below the comparison table. One-click "+" buttons add peers directly via `CompareAddDirect(String)`, separate from the text-input flow. Peers are fetched via a `company_metadata` self-join on sector.
   - *Files:* `state.rs` (+sector_peers field, +CompareAddDirect/SectorPeersLoaded messages), `db/universe.rs` (+fetch_sector_peers fn), `update/mod.rs` (+handlers, wire into TickerSelected), `view/fundamentals.rs` (+peer suggestion row)

5. **Chart astro event overlay** — the price chart canvas now renders two types of vertical markers: (a) extreme astro scores from lagrange_history (★ green at >=75, ⚠ red at <=25), and (b) retrograde station events (☿Rx, ♃D, etc.) from a new `fetch_retrograde_events()` query. The query uses `LAG()` window function on the `daily_transits` table to detect retrograde status transitions across the last year. Markers render as dashed vertical lines with glyph labels.
   - *Files:* `db/astro.rs` (+RetroEvent struct, +fetch_retrograde_events fn), `state.rs` (+retrograde_events field, +RetroEventsLoaded message), `update/mod.rs` (+handler, +load on startup), `view/overview.rs` (merge retrograde markers into astro_markers vec)

6. **Alert management UI** — alerts panel now has "Ack" (acknowledge/mark-read) and "✕" (dismiss/delete) buttons per alert. "Mark All Read" batch button appears when unread alerts exist. Dismissing permanently deletes from DB. Settings tab shows alert threshold info.
   - *Files:* `db/universe.rs` (+mark_all_alerts_read, +dismiss_alert fns), `state.rs` (+MarkAllAlertsRead, +DismissAlert messages), `update/mod.rs` (+handlers), `view/universe.rs` (redesigned alerts panel with action buttons), `view/settings.rs` (+alert threshold section)

*Note: "Fetch this ticker" button was deferred — requires scraper-side work beyond dashboard scope.*

### v3.1.7-3 — EDGAR Prioritization (2026-04-23)

**Backfill:** This item was deferred from v3.1.7 due to CIK lookup infrastructure requirements. Now implemented.

- **Dynamic CIK lookup** — `fetch_cik_map()` in `edgar_enrich.rs` made public. Downloads SEC's `company_tickers.json` (~14,000 companies) once per scraper run and reuses across both EDGAR filing fetch and IPO date enrichment.
- **Priority-first EDGAR fetching** — `fetch_all_edgar()` now accepts a `&HashMap<String, u64>` CIK map and `&[String]` priority tickers. Processes astro-ranked tickers (Top 5 + Bottom 5) first, then watchlist tickers, deduped. Falls back to hardcoded `CIK_MAP` for original 10 tickers when dynamic lookup misses.
- **CIK map sharing** — `enrich_first_filing_dates_with_cik()` accepts an optional pre-fetched CIK map, avoiding a redundant SEC API call. Pipeline calls it with the map already fetched in step 3.4.
  - *Files:* `src/scraper/edgar.rs` (rewritten `fetch_all_edgar` signature), `src/scraper/edgar_enrich.rs` (`fetch_cik_map` made public, new `_with_cik` variant), `src/scraper/main.rs` (CIK map fetch + pass to both functions)

### v3.1.8 — "Polish the Glass" (2026-04-23)

**Theme:** UX improvements discovered during video review. No new data sources; all changes are dashboard-side polish.

1. **Universe search box** — text input on the Universe Explorer tab filters by ticker symbol or company name. Uses case-insensitive `LIKE` matching in SQL with `$5` parameter threaded through `fetch_universe_page()` and `fetch_universe_count()`. Resets to page 0 on each keystroke. Combines with existing zone and sector filters.
   - *Files:* `state.rs` (+field, +message), `db/universe.rs` (+search param in both queries), `update/helpers.rs` (pass search to refresh), `update/mod.rs` (+handler), `view/universe.rs` (+text_input)

2. **Backtest minimum-data guard** — backtests now require 30+ days of astro+price data. Below that threshold, both `run_backtest()` and `run_strategy_backtest()` return an `insufficient_data: Option<String>` message instead of zeroed-out results. The view shows the message in amber text with guidance to run the scraper.
   - *Files:* `backtest.rs` (+field, threshold 2→30, updated tests), `strategy.rs` (+field, threshold 2→30), `view/astrology_tab.rs` (guard in both backtest and strategy views)

3. **Collapsible price table** — the 100-row OHLCV price table on the Fundamentals tab starts collapsed. Click "Price History (N rows)" to expand into a 300px scrollable. Saves vertical real estate for the metrics, DCF, agents, and comparisons that matter more.
   - *Files:* `state.rs` (+show_price_table field, +TogglePriceTable message), `update/mod.rs` (+handler), `view/fundamentals.rs` (toggle button + conditional scrollable)

4. **RSS ticker relevance** — RSS articles in the Research tab are now sorted with ticker-relevant articles first (headline or summary mentions selected ticker). Relevant headlines highlighted in green. Secondary sort by publication date.
   - *Files:* `view/research.rs` (sort + conditional color)

5. **Recently Viewed cap at 8** — reduced from 10 to 8 in all SQL queries (fetch, upsert prune) to prevent header overflow on smaller screens.
   - *Files:* `db/portfolio.rs` (LIMIT 10 → 8 in 3 queries)

6. **Sparse sparkline fix** — Lagrange sparkline now draws with a single data point (centered dot) instead of requiring 2+. Prevents division-by-zero when `n=1` and shows a dot+label for new tickers with minimal history.
   - *Files:* `charts.rs` (threshold empty→1, denom guard)

---

## v4.0 Implementation Roadmap — "The Forge" (Approved 2026-04-23)

**Goal:** Three-phase evolution from v3.1.9 to a professional-grade financial terminal. Tech debt first (stable internals), UI/UX second (visual polish), features last (new capabilities). Each phase is independently shippable.

**Codebase snapshot at approval:**

| Metric | Value |
|--------|-------|
| Total Rust lines | 17,851 |
| Largest file | `update/mod.rs` (921 lines) |
| `.unwrap()` count | 44 (27 in astrology module) |
| `panic!` count | 3 (swisseph_bridge + ephemeris) |
| Dashboard view/ | 2,628 lines across 9 files |
| Dashboard update/ | 1,183 lines across 2 files |
| Dashboard db/ | 1,165 lines across 5 files |

---

### Phase 1: v4.0.0 — "Structural Steel" (Tech Debt)

**Goal:** Reduce unwraps, split the monolithic update handler, eliminate code duplication, add error resilience. No visual changes. Dashboard behaves identically before and after.

#### 4.0.1 — Split `update/mod.rs` (921 lines -> 5 files)

The single 921-line match block is the biggest maintenance bottleneck. Every new message variant means touching one massive file.

| New file | Handles messages for | Est. lines |
|----------|---------------------|------------|
| `update/astro.rs` | AstroScore, Horoscope, RetroEvents, NatalChart | ~150 |
| `update/data.rs` | TickerSelected, PriceLoaded, FundamentalsLoaded, NewsLoaded | ~200 |
| `update/universe.rs` | UniversePage, Alerts, Search, Filters, Export | ~150 |
| `update/portfolio.rs` | Portfolio, Backtest, DCF, Watchlist, Import | ~120 |
| `update/mod.rs` | Tab switching, Theme, Tick, top-level dispatch | ~200 |

Each sub-module exports `fn handle_xxx(state: &mut Dashboard, msg: XxxMessage) -> Task<Message>`.

**Files:** `src/dashboard/update/{astro,data,universe,portfolio}.rs` (new), `update/mod.rs` (trimmed)

#### 4.0.2 — Unwrap Audit (44 -> <10)

| Severity | Files | Action |
|----------|-------|--------|
| Critical | `swisseph_bridge.rs` (6 unwraps) | Wrap FFI returns in Option/Result, log + default on failure |
| Critical | `interpretation.rs` (8 unwraps) | Replace with `.unwrap_or_default()` or format fallbacks |
| High | `natal.rs` (5 unwraps) | Use `?` operator or `.ok()` chains |
| Medium | `aspects.rs` (4), `agents.rs` (4) | Context-specific fixes |
| Safe | `Mutex::lock()` unwraps | Keep with documented justification |

**Target:** <10 unwraps remaining, all justified.

#### 4.0.3 — Agent Code Deduplication

The 4 agent personas (Buffett, Graham, Lynch, Munger) share ~60% identical structure. Extract shared template:

```rust
struct AgentTemplate {
    name: &'static str,
    philosophy: &'static str,
    metrics_focus: &[&str],
    astro_stance: AstroStance,  // Supportive / Skeptical / Pragmatic
}
fn generate_analysis(ctx: &AgentContext, template: &AgentTemplate) -> AgentAnalysis
```

**Estimated reduction:** ~400 lines removed from `agents.rs` (~1000 -> ~600).

#### 4.0.4 — Hardcoded Watchlist Extraction

Move `WATCHLIST` and `CIK_MAP` from `const` arrays in `scraper/main.rs` to DB-backed config. Query `SELECT ticker FROM watchlists WHERE name = 'default'` at scraper start, fall back to hardcoded list if empty. No migration needed (watchlists table exists from v3.1.8).

#### 4.0.5 — Error Handling Consistency

Replace `String` error types in dashboard DB functions with a proper enum:

```rust
pub enum DashError {
    Db(sqlx::Error),
    Parse(String),
    NotFound(String),
}
```

Affects: `db/*.rs` (5 files), `update/*.rs` (after split).

**Phase 1 totals:** ~800 lines changed, ~400 lines net reduction. Zero visual changes.

---

### Phase 2: v4.1.0 — "The Glass" (UI/UX Overhaul)

**Goal:** Professional financial terminal look. Icons, typography, refined theme, better widgets.

#### 4.1.1 — Typography Foundation

| Font | Usage | Weight |
|------|-------|--------|
| Inter | All UI text, labels, headers | Regular (400), SemiBold (600) |
| JetBrains Mono | Numbers, prices, scores, code | Regular (400) |

Load via `include_bytes!` in `main.rs`. Create `font.rs` with `const INTER: Font`, `const MONO: Font`. Apply MONO to all numeric columns, INTER to all labels and body text.

**Files:** `assets/fonts/` (4 .ttf files), `src/dashboard/font.rs` (new), `main.rs`, `theme.rs`

#### 4.1.2 — Icon System (Lucide via iced_fonts)

**New dependency:** `iced_fonts = { version = "0.1", features = ["lucide"] }`

Key icon mappings:

| Context | Lucide icon |
|---------|-------------|
| Tab: Astrology | Star |
| Tab: Overview | LayoutDashboard |
| Tab: Universe | Globe |
| Tab: Fundamentals | BarChart3 |
| Tab: Research | Newspaper |
| Tab: Portfolio | Briefcase |
| Tab: Settings | Settings |
| Bullish / Bearish | TrendingUp / TrendingDown |
| Refresh / Search | RefreshCw / Search |
| Alert / Export | Bell / Download |
| Mercury Rx | AlertTriangle |

Create `src/dashboard/icons.rs` mapping icon names to Lucide codepoints. Replace all text-based pseudo-icons with proper icons.

**Files:** `Cargo.toml`, `src/dashboard/icons.rs` (new), all `view/*.rs` files

#### 4.1.3 — Theme Upgrade (CatppuccinMocha + Custom Financial)

Replace manual `Theme::Custom` with Catppuccin Mocha palette:

| Element | New Color (Mocha) |
|---------|-------------------|
| Background | #1e1e2e (Base) |
| Surface | #313244 (Surface0) |
| Text | #cdd6f4 (Text) |
| Accent/Blue | #89b4fa (Blue) |
| Green (bullish) | #a6e3a1 (Green) |
| Red (bearish) | #f38ba8 (Red) |
| Gold (astro) | #f9e2af (Yellow) |
| Subtext | #a6adc8 (Subtext0) |

Also add TokyoNight as alternate dark theme, plus a clean Light theme. Circadian auto-switch: Dawn/Day = Light, Dusk/Night = Mocha.

**Files:** `theme.rs` (rewrite), `view/mod.rs` (theme toggle)

#### 4.1.4 — Tab Bar Redesign

Icon + label tabs with active accent-colored underline. Inactive tabs use subtext color, hover shows surface highlight. Icons from 4.1.2.

**Files:** `view/mod.rs` (tab bar), `tabs.rs`

#### 4.1.5 — Card/Panel Layout System

Wrap each dashboard section in a consistent card container with rounded corners, surface background, subtle border, icon + title header, and horizontal rule separator.

**Files:** `view/shared.rs` (card helper), all `view/*.rs` files

#### 4.1.6 — Numeric Formatting Polish

- Prices: `$1,234.56` (comma-grouped, 2 decimals)
- Percentages: `+12.3%` / `-4.7%` (sign prefix, colored)
- Large numbers: `$1.2B`, `$340M`, `$12.5K`
- Scores: monospace font, fixed width for column alignment
- Dates: `Apr 23, 2026` (human readable)

**Files:** `helpers.rs` (expand formatters), all view files

**Phase 2 totals:** ~1,200 new lines, ~400 lines changed. Major visual upgrade.

---

### Phase 3: v4.2.0 — "The Expansion" (New Features)

**Goal:** Port highest-value FinceptTerminal features and add polish features.

#### 4.2.1 — Plotters-Iced Candlestick Charts

**New dependency:** `plotters-iced = "0.11"`

Replace custom canvas chart with professional plotters-backed chart: OHLC candlesticks, volume bars, timeframe selector (1W/1M/3M/6M/1Y/ALL), astro event marker annotations, SMA overlay lines (20, 50, 200).

**Files:** `charts.rs` (rewrite), `view/overview.rs`, `db/ticker_data.rs` (+OHLC query)
**Migration:** `0021_ohlc_data.sql` (add open/high/low columns if missing)

#### 4.2.2 — Options Greeks Calculator

Black-Scholes implementation: delta, gamma, theta, vega, rho, implied volatility. Display in Fundamentals tab alongside existing metrics.

**Files:** `src/dashboard/greeks.rs` (new, ~200 lines), `view/fundamentals.rs`

#### 4.2.3 — Improved Table Widget

**New dependency:** `iced_table = "0.13"` (or custom reusable sorted table)

Sortable columns, alternating row colors, fixed header scroll. Apply to Universe Explorer, price history, insider trades, filings.

**Files:** `src/dashboard/table.rs` (new), `view/universe.rs`, `view/research.rs`, `view/fundamentals.rs`

#### 4.2.4 — Toast Notifications

Replace `println!` feedback with in-app toasts: success (green), info (blue), error (red). Auto-dismiss after 4 seconds, stack up to 3.

**Files:** `src/dashboard/toasts.rs` (new), `state.rs`, `update/mod.rs`

#### 4.2.5 — GDELT Geopolitical Events

Free API, no key. Fetch geopolitical events affecting markets. Display in Research tab alongside news.

**Files:** `src/scraper/gdelt.rs` (new, ~150 lines), `migration 0022_gdelt_events.sql`, `view/research.rs`

#### 4.2.6 — Astro Calendar View

Monthly calendar showing upcoming exact aspects for watchlist tickers. "Best days to buy" / "Days to avoid" based on aggregate astro forecast.

**Files:** `calendar.rs` (expand existing), `view/astrology_tab.rs`, `db/astro.rs`

**Phase 3 totals:** ~1,800 new lines, 3 new crates.

---

### v4.0 Dependency Chain

```
Phase 1: Tech Debt (v4.0.0)        <- do first, no visual changes
  |-- 4.0.1 Split update/mod.rs    <- unblocks everything
  |-- 4.0.2 Unwrap audit           <- independent
  |-- 4.0.3 Agent dedup            <- independent
  |-- 4.0.4 Watchlist extraction   <- independent
  +-- 4.0.5 Error enum             <- after split

Phase 2: UI/UX (v4.1.0)            <- after phase 1
  |-- 4.1.1 Typography             <- do first (fonts used everywhere)
  |-- 4.1.2 Icons (iced_fonts)     <- after fonts
  |-- 4.1.3 Theme (Catppuccin)     <- independent of icons
  |-- 4.1.4 Tab bar redesign       <- after icons + theme
  |-- 4.1.5 Card layout            <- after theme
  +-- 4.1.6 Numeric formatting     <- independent

Phase 3: Features (v4.2.0)         <- after phase 2
  |-- 4.2.1 Candlestick charts     <- highest impact, do first
  |-- 4.2.2 Options Greeks          <- independent
  |-- 4.2.3 Table widget            <- independent
  |-- 4.2.4 Toast notifications     <- independent
  |-- 4.2.5 GDELT geopolitics       <- independent
  +-- 4.2.6 Astro calendar          <- independent
```

### v4.0 New Dependencies

| Crate | Version | Phase | Purpose |
|-------|---------|-------|---------|
| `iced_fonts` | 0.1 | 4.1.2 | Lucide icons (1,400+ SVG-as-font icons) |
| `plotters-iced` | 0.11 | 4.2.1 | Professional candlestick charts |
| `iced_toasts` | latest | 4.2.4 | In-app notification toasts |

### v4.0 Effort Summary

| Phase | New lines | Changed lines | Net delta | New files |
|-------|-----------|---------------|-----------|-----------|
| 4.0 Tech Debt | ~400 | ~800 | -400 | 4 |
| 4.1 UI/UX | ~1,200 | ~600 | +800 | 3 |
| 4.2 Features | ~1,800 | ~400 | +1,400 | 5 |
| **Total** | **~3,400** | **~1,800** | **+1,800** | **12** |

### v4.0 Reference Material

- **Halloy** (IRC client, Iced): Best-in-class Iced app for theme and layout patterns
- **Liana** (Bitcoin wallet, Iced): Financial data display patterns
- **Cryptowatch** (financial terminal): Visual benchmark for chart and data density
- **FinceptTerminal** (C++20/Qt6): Feature source for Greeks, GDELT, advanced charts
- **Catppuccin Mocha**: Theme palette specification (catppuccin.com)

---

## Implementation Roadmap (v3.1.6 through v3.1.9)

*Compiled from video review of dashboard (158 frames, ~8 min screen recording) and scraper console output analysis on 2026-04-23. Items ordered by user impact, grouped into coherent shipping units.*

### v3.1.6 — "Fix the Pipes" (Critical Fixes)

| # | Item | Type | Details |
|---|------|------|---------|
| 1 | DBnomics API URL fix | Bug | `api.db.nomics.world` DNS fails. All 5 international macro indicators show "---". Verify current endpoint, update URL, add fallback. |
| 2 | Polymarket financial filtering | Bug | Markets displayed include sports/politics (Houston Astros, French elections, Ligue 1 soccer). Filter to `economics`, `fed`, `inflation`, `recession`, `crypto` only. |
| 3 | Dead RSS feed cleanup | Bug | Reuters, NYT, Defense News return 404/403. Remove or replace with working alternatives. |
| 4 | Hide empty international macro row | UX | When all 5 DBnomics values are "---", hide the row entirely instead of showing 5 dashes. |

**Files:** `src/scraper/dbnomics.rs`, `src/scraper/polymarket.rs`, `src/scraper/rss_news.rs`, `src/dashboard/view/shared.rs`

### v3.1.7 — "The Verification Layer" (Sub-scores + Sector Data)

| # | Item | Type | Details |
|---|------|------|---------|
| 1 | Free sector/industry data | Feature | Populate sector/industry from Polygon ticker details API (already integrated). Unblocks Sector Heat Map, sector filtering, Universe sector column. |
| 2 | Lagrange sub-score computation | Bug | Financial, Macro, Short sub-scores all show "---" in Universe. Wire existing data into sub-score columns so composite isn't just Astro echoed. |
| 3 | EDGAR 8-K + Form 4 prioritization | Bug | Watchlist/astro-priority tickers should get filings and insider trades fetched. Wire EDGAR into priority system. |
| 4 | Smarter agent analysis | UX | Agents should analyze available data (price, astro, sentiment, short %) when FMP fundamentals missing. "Insufficient Data" only when nothing is available. |

**Files:** `src/scraper/ticker_seed.rs`, `src/scraper/lagrange.rs`, `src/scraper/edgar_enrich.rs`, `src/dashboard/agents.rs`, `src/dashboard/db/universe.rs`

### v3.1.8 — "Polish the Glass" (UX Improvements)

| # | Item | Type | Details |
|---|------|------|---------|
| 1 | Universe search box | Feature | Text filter to search 1739 tickers by symbol or company name without paging 35 pages. |
| 2 | Backtest minimum-data guard | UX | Show "Need 30+ days of astro history" instead of zeroed-out "0 trades, 0%" result. |
| 3 | Collapsible price table | UX | Fundamentals tab: show 15 rows default with "Show all" toggle. Full OHLCV shouldn't dominate. |
| 4 | RSS ticker relevance | UX | Tag RSS articles mentioning selected ticker. Show ticker-relevant first, general market news below. |
| 5 | Recently Viewed cap at 8 | UX | Drop oldest when exceeding 8 tickers. Keeps header tight. |
| 6 | Lagrange sparkline with sparse data | UX | Draw mini-chart with few points instead of "not enough history" text fallback. |

**Files:** `src/dashboard/view/universe.rs`, `src/dashboard/view/overview.rs`, `src/dashboard/view/fundamentals.rs`, `src/dashboard/view/research.rs`, `src/dashboard/state.rs`

### v3.1.9 — "New Tools" (Feature Additions)

| # | Item | Type | Details |
|---|------|------|---------|
| 1 | "Fetch this ticker" button | Feature | On no-data tickers, show button to queue single-ticker scrape. |
| 2 | Alert management UI | Feature | Dismiss/acknowledge alerts, configure thresholds (score change %, zone transitions). |
| 3 | Chart astro event overlay | Feature | Vertical markers on price chart for retrogrades and exact aspects. |
| 4 | News sentiment coloring | Feature | Green/red/gray badges on news headlines using existing sentiment data. |
| 5 | Export Universe CSV | Feature | Export current filtered universe view to CSV. |
| 6 | Portfolio import from watchlist | Feature | One-click import watchlist tickers as portfolio positions. |
| 7 | Comparative auto-suggest peers | Feature | Suggest sector peers for comparison (depends on v3.1.7 sector data). |

**Files:** `src/dashboard/view/overview.rs`, `src/dashboard/view/universe.rs`, `src/dashboard/view/research.rs`, `src/dashboard/view/portfolio_tab.rs`, `src/dashboard/charts.rs`

**Dependency chain:** v3.1.6 (independent) -> v3.1.7 (uses fixed DBnomics for macro sub-score) -> v3.1.8 (polishes features from v3.1.7) -> v3.1.9 (adds new features on solid base). v3.1.9 item 7 depends on v3.1.7 item 1 (sector data).

---

## Changelog

### v3.1.7 — "The Verification Layer" *(completed 2026-04-23)*

Three improvements that make the Lagrange composite score meaningful and the dashboard useful without an FMP paid key.

**Item 1 -- Free sector/industry data via Finnhub:** Added `enrich_sectors()` to `finnhub.rs` that calls `/stock/profile2` for tickers in the scored universe missing sector data. Finnhub returns `finnhubIndustry` (e.g., "Biotechnology", "Software"), which we map to ~11 GICS-like sectors via `finnhub_industry_to_sector()` (a 70-category lookup). Capped at 50 tickers per scraper run to stay within rate limits. Populates both `sector` and `industry` columns in `company_metadata`. Unblocks the Sector Heat Map, sector filtering, and Universe sector column.

**Item 2 -- Auto-expand scoring universe:** Previously, only the 10 hardcoded watchlist tickers had `scoring_active = true` in the `tickers` table. The other 1739 tickers from Polygon had astro scores but no Lagrange scores (sub-scores all "---"). Added an auto-expansion step at the start of `compute_all_scores()` that INSERTs any ticker with 26+ price rows into `tickers` with `scoring_active = true`. Uses `ON CONFLICT DO UPDATE` so existing watchlist tickers aren't disturbed. As Tiingo fills in price history (~490 tickers/day), the Lagrange scoring universe grows automatically.

**Item 3 -- Smarter agent analysis without FMP fundamentals:** All four agents (Buffett, Graham, Lynch, Munger) previously returned "Verdict: Insufficient Data" with a one-line dismissal when FMP fundamentals were missing. Added `no_fundamentals_fallback()` that generates a partial analysis from available signals: astro score, Lagrange composite, current price, Mercury retrograde status, and moon phase. Each agent gets persona-specific commentary. "Insufficient Data" only shows when literally no signals are available. The fallback produces a real verdict (Buy/Hold/Sell) with metric cards, so the Fundamentals tab is useful even without FMP.

**Deferred:** EDGAR 8-K/Form 4 prioritization (requires CIK lookup infrastructure for arbitrary tickers). Will address when the watchlist expansion makes it impactful.

**Modified:** `src/scraper/finnhub.rs` (sector enrichment + industry-to-sector mapping), `src/scraper/lagrange.rs` (auto-expand scoring universe), `src/dashboard/agents.rs` (fallback analysis for all 4 personas)

---

### v3.1.6 — "Fix the Pipes" *(completed 2026-04-23)*

Four critical fixes identified from 158-frame video review and scraper console output analysis.

**Fix 1 -- DBnomics API URL:** The scraper used `api.dbnomics.org` which doesn't exist. The correct domain is `api.db.nomics.world` per the [official docs](https://docs.db.nomics.world/web-api/). This single-character fix unblocks all 5 international macro indicators (Euribor 3M, PBoC, EU CPI, OECD CLI, Credit/GDP) that were showing "---" on the macro strip.

**Fix 2 -- Polymarket financial filtering:** The `fetch_top_markets()` function fetched 20 markets by volume with no category filter, pulling in sports (Houston Astros, Ligue 1 soccer) and niche politics (French elections). Fix: removed `fetch_top_markets()` entirely. Replaced `elections` tag with `stocks`, `interest-rates`, and `markets` tags. All markets now come from financially relevant tag queries only.

**Fix 3 -- Dead RSS feeds:** Reuters (2 feeds), NYT, and Defense News all return 404/403 (feeds retired). Replaced with: Yahoo Finance (markets), FT (wire), Naked Capitalism (analysis), Politico Economy (wire). Feed count stays at 25.

**Fix 4 -- Empty international macro row:** When all 5 DBnomics values are missing, the macro strip showed 5 dashes ("Euribor 3M: --- PBoC: --- ..."). Now the international row is hidden entirely when no DBnomics data exists. A `has_value()` helper checks each series_id. The row reappears automatically once DBnomics data is fetched.

**Modified:** `src/scraper/dbnomics.rs` (API URL fix), `src/scraper/polymarket.rs` (removed unfiltered fetch, updated tags), `src/scraper/rss_news.rs` (replaced 4 dead feeds), `src/dashboard/view/shared.rs` (conditional international macro row)

---

### v3.1.5 — Module Decomposition Refactor *(completed 2026-04-22)*

Decomposed 3 monolithic dashboard files (~4,000 lines total) into 16 focused domain modules. No functional changes; pure structural refactor for AI navigability, human readability, and incremental compile performance.

**view.rs (1843 lines) → view/ directory (9 files):**
- `mod.rs` (136) — shared chrome: header, search bar, tab bar, tab dispatch
- `shared.rs` (71) — `make_gauge()` helper, `build_macro_strip()` method
- `overview.rs` (609) — price chart, gauges, indicators, signals, watchlist, Polymarket
- `astrology_tab.rs` (400) — natal wheel, horoscope, transits, backtest, calendar
- `universe.rs` (335) — universe table, sector heat map, alerts panel
- `fundamentals.rs` (348) — metrics grid, DCF, agents, comparative analysis, earnings
- `research.rs` (245) — 8-K filings, news, RSS, insider trades, holdings
- `portfolio_tab.rs` (226) — P&L, named watchlists, transaction log
- `settings.rs` (63) — theme toggle, font scale, refresh interval, info stats

**db.rs (1070 lines) → db/ directory (5 files):**
- `mod.rs` (94) — connection, ticker list/search, settings CRUD, `pub use *` re-exports
- `ticker_data.rs` (173) — per-ticker queries: prices, filings, news, fundamentals, macro
- `astro.rs` (162) — astro scores, natal charts, transits, aspects, horoscope, backtest data
- `universe.rs` (381) — WatchlistRow, UniverseRow, CompareRow, sector summaries, fear/greed
- `portfolio.rs` (244) — positions, P&L, transactions, named watchlists, recently viewed

**update.rs (1078 lines) → update/ directory (2 files):**
- `mod.rs` (844) — `new()`, `update()` match (80+ handlers), `subscription()`, `fetch_all()`
- `helpers.rs` (223) — refresh helpers, agent/DCF recompute, CSV export, toast, keyboard handler

**Technique:** Rust directory-module conversion (`foo.rs` → `foo/mod.rs`) with `pub use submodule::*` re-exports preserves all existing import paths. `impl Dashboard` blocks split across files with `pub(crate)` visibility.

### v3.1.4 — Video Review Bug Fixes *(completed 2026-04-22)*

**Theme:** Six fixes identified from screen recording review of the live dashboard at 2560x1540 resolution.

**Bug 5 fix -- Y-axis labels garbled for low-price stocks:** `charts.rs:144` used `format!("{pr:.0}")` which truncated all labels to zero decimals. A $2.18 stock showed "2, 2, 2, 2, 2" on the Y-axis. Fix: dynamic precision based on price range (range < $5: 2 decimals, < $50: 1 decimal, else: integer).

**Bug 3 fix -- Polymarket volume "$1000K" instead of "$1.0M":** When volume was ~$999,500, rounding to zero decimals in the $K branch produced "$1000K". Fix: lowered the million threshold to $999,500 so borderline values display as "$1.0M".

**UX 5 fix -- Calendar nav buttons showed "0":** The Unicode characters "◀" and "▶" aren't in Iced's default font, rendering as "0". Fix: replaced with ASCII text "< Prev" and "Next >".

**Bug 2 fix -- Polymarket categories all showing "---":** The Gamma API returns NULL for `category` on most markets. Fix: (1) pass the query tag (e.g. "economics", "fed") as a fallback category when the API doesn't provide one, (2) the UPSERT now also updates `category` with COALESCE to preserve existing non-NULL values.

**Bug 6 fix -- Macro strip "---%" formatting:** When DBnomics data was unavailable, the format string `"Euribor 3M: —%"` showed a trailing "%" after the dash. Fix: refactored macro_fmt helper to only include the suffix/prefix when actual data exists. Missing data now shows clean "Euribor 3M: —".

**UX 3 fix -- Current price label on chart:** Added a blue pill with white text at the right edge of the price chart showing the last closing price. Uses the same dynamic precision as Y-axis labels. Makes it immediately obvious what the current price is without hovering.

**Modified:** `src/dashboard/charts.rs` (Y-axis precision + current price label), `src/dashboard/view.rs` (Polymarket volume threshold, calendar buttons, macro strip formatting), `src/scraper/polymarket.rs` (category fallback from query tag, UPSERT updates category)

---

### v3.1.3 — Font Scale Setting + Astro Priority Full Scrape *(completed 2026-04-22)*

**Theme:** Two user-requested improvements. (1) Bigger, bolder text with runtime-adjustable font scale. (2) Top 5 astro-ranked tickers get complete data from all scraper sources, not just price data.

**Font scale system:** Converted 6 hardcoded type-scale constants (`TEXT_XS` through `TEXT_2XL`) to runtime functions backed by `std::sync::atomic::AtomicU32`. The base scale was bumped ~25% (body text 12px to 15px, headings proportionally). Four presets are available in the Settings tab: Compact (0.85x), Default (1.0x), Large (1.15x), XL (1.35x). The setting persists in the `settings` table and applies instantly without restart. All 384 `.size()` calls across `view.rs` and `astrology.rs` were migrated from constants to function calls via batch replacement.

**Astro priority full scrape:** Previously, only `prices::fetch_priority_prices()` used the astro ranking's top 5 + bottom 5 tickers. Now three additional scraper modules accept `extra_tickers: &[String]` and process astro-priority tickers alongside the watchlist: Finnhub (news + analyst ratings), FINRA short interest, and Alpha Vantage sentiment. AV sentiment processes priority tickers first to ensure they get data before the 25-call daily budget is exhausted. All ticker lists are deduplicated so watchlist members appearing in the astro top/bottom 10 are not fetched twice.

**Modified:** `src/dashboard/theme.rs` (atomic font scale), `src/dashboard/view.rs` (384 constant-to-function migrations + font scale UI in Settings), `src/dashboard/astrology.rs` (21 constant-to-function migrations), `src/dashboard/state.rs` (font_scale_label field), `src/dashboard/update.rs` (font_scale setting handler), `src/scraper/finnhub.rs` (extra_tickers param), `src/scraper/short_interest.rs` (extra_tickers param), `src/scraper/sentiment.rs` (extra_tickers param, priority-first ordering), `src/scraper/main.rs` (pass priority to 3 modules)

---

### v3.1.2 — Polymarket Prediction Markets *(completed 2026-04-22)*

**Theme:** Prediction market integration from Polymarket's free Gamma API. Ported from FinceptTerminal's PolymarketService.cpp (843 lines C++ to ~210 lines Rust). Adds macro sentiment visibility via real-money prediction markets.

**Implementation:** Fetches top active markets by volume across 6 financially relevant tags (economics, fed, inflation, recession, crypto, elections). The Gamma API returns numeric fields as JSON strings ("1567632.01") and outcomes/outcomePrices as either JSON arrays or string-encoded arrays. Two helpers (`num_or_str`, `parse_str_or_array`) handle both forms, matching the C++ original's approach. Markets are upserted by `market_id` so probabilities stay fresh across scraper runs.

**Dashboard wiring:** Overview tab gains a "Prediction Markets" section below the two-column layout showing top 8 active markets with Yes probability percentage, category badge, question text, and formatted volume. The data loads once on startup (global, not per-ticker).

**Created:** `src/scraper/polymarket.rs`, `migrations/0030_polymarket.sql`
**Modified:** `src/scraper/main.rs` (mod + Phase 3 step 3.8), `src/models.rs` (PolymarketMarket), `src/dashboard/db.rs` (fetch_polymarket), `src/dashboard/state.rs` (polymarket field + PolymarketLoaded message), `src/dashboard/update.rs` (handler + initial load), `src/dashboard/view.rs` (Overview tab prediction markets section)

### v3.1.1 — RSS News Aggregation *(completed 2026-04-22)*

**Theme:** Free RSS/Atom news feed aggregation from 25 curated sources. Ported from FinceptTerminal's NewsService.cpp (1327 lines C++ to ~170 lines Rust). The massive reduction is thanks to the `feed-rs` crate which handles RSS 0.9/1.0/2.0 and Atom format detection + parsing automatically.

**Feed selection (25 of 80+ from C++ source):** Wire services (Reuters Business, Reuters Markets, BBC Business, NYT World, Al Jazeera), regulators (SEC, Federal Reserve, ECB), financial media (Bloomberg Markets, WSJ Markets, MarketWatch, CNBC Finance, Seeking Alpha), analysis (Economist, Calculated Risk, Wolf Street), tech (TechCrunch, Finextra), Asia-Pacific (SCMP, Nikkei Asia), energy (OilPrice), crypto (CoinDesk, CoinTelegraph), geopolitics (Foreign Policy, Defense News).

**Parallel fetch:** All 25 feeds fetched concurrently via `tokio::spawn` with 5-second per-feed timeout. HTML stripped from summaries with a simple state-machine parser, truncated to 300 chars (matching C++ `kSummaryMaxChars`). Articles deduplicated by `link` unique constraint.

**Dashboard wiring:** Research tab gains "Market News (RSS)" section between Finnhub per-ticker news and insider trades. Shows date, source name, category badge, headline, and Open button. Scrollable list of 50 most recent articles.

**New dependency:** `feed-rs = "2"` (RSS/Atom parser)
**Created:** `src/scraper/rss_news.rs`, `migrations/0029_rss_articles.sql`
**Modified:** `Cargo.toml`, `src/scraper/main.rs`, `src/models.rs` (RssArticle), `src/dashboard/db.rs`, `src/dashboard/state.rs`, `src/dashboard/update.rs`, `src/dashboard/view.rs`

### v3.1.0 — DBnomics International Economics *(completed 2026-04-22)*

**Theme:** Free international macroeconomic data from DBnomics (api.dbnomics.org), an aggregator of 70+ statistical providers. No API key needed. Ported from FinceptTerminal's DBnomicsService.cpp (372 lines C++ to ~180 lines Rust).

**Design decision:** Rather than creating a separate table, DBnomics observations reuse the existing `macro_indicators` table with a `DBNOMICS:` prefix on `series_id`. This means the dashboard macro strip displays international indicators with zero new model types, zero new queries, and zero new state fields. A partial index (`WHERE series_id LIKE 'DBNOMICS:%'`) keeps queries efficient.

**6 series selected:** ECB Euribor 3-month rate (EU short-term benchmark), BIS PBoC policy rate (China monetary policy), IMF US GDP growth forecast (World Economic Outlook), Eurostat CPI YoY (Eurozone inflation), OECD Composite Leading Indicator (US), BIS Total Credit to Private Sector as % of GDP (US).

**Period parsing:** DBnomics returns dates in 4 formats depending on the provider: `2024-01-15` (daily), `2024-01` (monthly), `2024-Q1` (quarterly), `2024` (annual). The `parse_dbn_period()` function normalizes all four to `NaiveDate`, defaulting to the 1st of the period for non-daily formats.

**Dashboard wiring:** Overview and Portfolio tabs gain a second macro strip row showing international indicators: Euribor 3M, PBoC rate, EU CPI, OECD CLI, US Credit/GDP. Shows "—" until first scraper run populates data.

**Created:** `src/scraper/dbnomics.rs`, `migrations/0028_dbnomics_series.sql`
**Modified:** `src/scraper/main.rs` (mod + Phase 3 step 3.6), `src/dashboard/view.rs` (international macro strip row)

---

### v3.0.7 — Dashboard UI Review & Fixes *(completed 2026-04-22)*

**Theme:** Full dashboard UI review via video frame extraction (87 frames, all 7 tabs, 3 tickers). Six bugs fixed plus four UX improvements. Covers search interaction, Universe data pipeline, horoscope display, scraper rate limiting, and visual polish.

**Bug 4 (CRITICAL UX): Search Autocomplete Dropdown Never Dismisses.** Root cause was an async race condition, not a missing handler. Selecting a suggestion clears `ticker_search_input`, but Iced's `text_input` fires `on_input("")` when cleared, which triggers a new `search_tickers("")` query. The async result arrives after the selection handler ran, repopulating the dropdown. Fix: added `autocomplete_dismissed: bool` guard flag to `Dashboard` state. Set to `true` on selection, submit, and Escape. Checked in `TickerSearchInput` and `AutocompleteResults` handlers to short-circuit the stale async cycle. Reset to `false` on next real keypress.

**Bug 5+6 (MODERATE): Universe Explorer Shows Only 3 of 1675 Scored Tickers.** The Universe Explorer used `FROM lagrange_history` as its primary table, which only contains the 10 watchlist tickers with composite scores. The 1665 tickers that have astro scores but no Lagrange history were excluded. Fix: rewrote `fetch_universe_page()` and `fetch_universe_count()` in `db.rs` to use `FROM astro_scores` as the primary table with `LEFT JOIN lagrange_history`. Uses two CTEs (`latest_astro`, `latest_lagrange`) to get the most recent score per ticker. Astro scores display directly; Lagrange sub-scores show "---" when absent. Sort by `astro_score DESC NULLS LAST`. The astro label is derived via CASE expression from the score value (Optimal/Favorable/Neutral/Unfavorable/Misaligned).

**Bug 7 (MODERATE): Horoscope Reading Narrative Not Displayed.** The scraper generates horoscope readings (overall outlook, dominant theme, key transits, moon guidance, mercury retrograde warnings, timing windows) and stores them in `horoscope_readings` as JSONB, but the dashboard never queried or rendered them. Fix: added `fetch_horoscope()` to `db.rs` that queries the latest reading and reconstructs `HoroscopeReading` via `horoscope_from_json()`. Added `horoscope` field to state, `HoroscopeLoaded` message, wired into `fetch_all()` batch. View renders the reading below the natal wheel with structured sections: outlook, theme + confidence, key transits table, moon guidance + mercury warning, timing window. Falls back to "not yet generated" message.

**Bug 8 (MINOR): AV Rate Limiter No Backoff on "Information" Response.** Alpha Vantage returns HTTP 200 with JSON `{"Information": "..."}` or `{"Note": "..."}` when rate-limited instead of HTTP errors. The scraper treated these as parse failures. Fix: added retry loop in `fetch_and_store()` (`for attempt in 0..2`). On first rate-limit JSON response, logs and sleeps 60s before retrying. On second hit, bails with descriptive error. This prevents 4-of-10 priority ticker failures in rapid succession.

**Bug 9 (MINOR): Earnings Calendar Not Ticker-Specific.** `fetch_all_earnings()` had no WHERE clause, showing all watchlist earnings regardless of selected ticker. Fix: added `fetch_ticker_earnings()` with `WHERE ticker = $1` filter. Wired into per-ticker data loading. Section title now shows "{ticker} Earnings" instead of generic "Earnings Calendar".

**UX Improvements:**
1. Natal wheel enlarged from 240px to 300px for readable planet glyphs (plan specified 300x300).
2. Astro calendar now has a color legend below the grid: green = favorable (>50), neutral (~50), red = unfavorable (<50).
3. Title bar now shows tab-contextual subtitle: "Astrology & Timing", "Daily Price Data", "Universe Explorer", etc.
4. Agent empty state now shows each persona's philosophy (Buffett: moat/FCF, Graham: margin of safety, Lynch: PEG ratio, Munger: mental models).

**Modified:** `src/dashboard/db.rs` (universe SQL rewrite, `fetch_horoscope()`, `fetch_ticker_earnings()`), `src/dashboard/state.rs` (`autocomplete_dismissed`, `horoscope` field, `HoroscopeLoaded` message), `src/dashboard/update.rs` (autocomplete guard, horoscope wiring, earnings wiring), `src/dashboard/view.rs` (horoscope render, wheel size, calendar legend, title bar, agent empty state), `src/scraper/prices.rs` (retry loop)

### v3.0.6 — Post-Release Bugfix Patch *(completed 2026-04-22)*

**Theme:** Quality pass after reviewing scraper and dashboard output at scale. Three bugs fixed, 16 compiler warnings eliminated. Both binaries now build with zero warnings.

**Bug 1 (CRITICAL): Score Polarization.** Astro scores clustered bimodally at 0-5 and 95-100 across 1642 tickers. The logistic sigmoid in `natal.rs` fed raw `delta_sum` (range +/-300 with 50-70 aspects per ticker) into a sigmoid with k=0.04, which saturated at extremes. Fix: normalize by `sqrt(aspect_count)` before the sigmoid, increase k to 0.10. The sqrt normalization is the standard statistical approach for sums of independent variables. It preserves the signal ("more aligned aspects = stronger") while preventing large aspect counts from overwhelming the sigmoid. New distribution is bell-shaped, centered at 50, with meaningful spread from ~27 to ~73 at the tails.

**Bug 2 (MODERATE): Theme-Score Mismatch.** Tickers like AG scored 64 ("Greed") but showed theme "Mild Caution & Restructuring". The theme classifier counts aspect frequencies by planet type (how many Saturn aspects), while the score sums signed deltas. These independent pathways disagreed in the 36-64 range because the reconciliation logic only overrode at extremes (65+ or 0-35). Fix: expand thresholds to 56+/0-44, matching the score label boundaries where Neutral ends (45-55).

**Bug 3 (MINOR): AstroRanking Not Consumed.** The `AstroRanking` struct was computed in Phase 1 but stored as `_ranking` (unused). Phase 2 fetched financial data in hardcoded WATCHLIST order, ignoring astro-prioritized tickers. Fix: renamed to `ranking`, added `fetch_priority_prices()` in `prices.rs` that fetches price data for top 5 + bottom 5 astro tickers before the watchlist. Tickers already in the watchlist are skipped (HashSet dedup) to avoid wasting API calls.

**16 Compiler Warnings Resolved:**
- 5 true dead code removed: unused `Theme` import (view.rs:3), unused `Logic` import (view.rs:936), unused variable `n` (view.rs:103), unused `shortcut()` method (tabs.rs:53), unused `TEXT_XL` constant (theme.rs:100)
- 9 incomplete feature fields suppressed with `#[allow(dead_code)]`: AgentContext (6 fields for LLM path), CalendarDay.label, DcfResult (3 breakdown fields), SectorSummary.avg_lagrange, PortfolioPnlRow.notes, TransactionRow.notes, AstroRanking.total_scored, FmpKeyMetrics.earnings_yield_ttm
- 2 partially wired strategy features suppressed: MacdCrossUp/MacdCrossDown variants, Condition::all_options(), Strategy.name

**Modified:** `src/astrology/natal.rs` (sigmoid fix), `src/astrology/interpretation.rs` (reconciliation fix), `src/scraper/main.rs` (ranking wiring), `src/scraper/prices.rs` (new `fetch_priority_prices()`), `src/scraper/astrology.rs` (allow dead_code), `src/scraper/fundamentals.rs` (allow dead_code), `src/dashboard/view.rs` (3 dead code removals), `src/dashboard/tabs.rs` (remove shortcut), `src/dashboard/theme.rs` (remove TEXT_XL), `src/dashboard/agents.rs` (allow dead_code), `src/dashboard/calendar.rs` (allow dead_code), `src/dashboard/dcf.rs` (allow dead_code), `src/dashboard/db.rs` (allow dead_code x3), `src/dashboard/strategy.rs` (allow dead_code x3)
**Tests:** 37 passed, 1 pre-existing failure (lunar node tolerance, unrelated)

---

### v3.0.5 — Astro Calendar *(completed 2026-04-22)*

**Theme:** A monthly calendar view in the Astrology tab where each day is colored by the astro score for the selected ticker. Green days = favorable astro conditions, red days = unfavorable. Navigate months with prev/next buttons. This answers: "When are the best days to buy this month?"

**Why a calendar?** Traders think in time windows. "Is next week a good time to enter AAPL?" The calendar turns the astro score time series into a visual heat map of favorable and unfavorable days. At a glance, the user sees clusters of green (favorable windows) and red (avoid windows).

**Canvas widget:** `AstroCalendar` implements `canvas::Program<Message>` with a 7-column Monday-to-Sunday grid. Each day cell is colored using two-phase interpolation (same scheme as the sector heat map). The day number appears top-left, the score bottom-right. A month/year label anchors the bottom.

**Date math:** Uses `chrono::NaiveDate` weekday offset to position day 1 in the correct column. Days-in-month is computed by finding the first day of the next month and subtracting one day. The prev/next navigation wraps correctly across year boundaries (December -> January).

**DB query:** `fetch_astro_calendar(pool, ticker, start_date, end_date)` returns `(date, score, label)` tuples for the date range. Only days with non-null scores are returned.

**New file:** `src/dashboard/calendar.rs` (~165 lines)
**New query:** `fetch_astro_calendar()` in db.rs
**Modified:** `src/dashboard/main.rs` (mod calendar), `src/dashboard/state.rs` (3 fields + 3 messages), `src/dashboard/update.rs` (3 handlers + refresh_calendar + initial load), `src/dashboard/view.rs` (calendar in Astrology tab)
**Tests:** 8 dashboard tests passing (no new tests; calendar is visual)

---

### v3.0.4 — Settings Panel *(completed 2026-04-22)*

**Theme:** A persistent settings system using a database key-value store. Users can change theme mode and refresh interval from a dedicated Settings tab, and the changes persist across sessions.

**Why DB-backed settings?** Previously, theme mode and other preferences reset on every app launch. The `settings` table stores key-value pairs that are loaded on startup and applied immediately. The `ON CONFLICT DO UPDATE` upsert pattern means the same code handles both first-run initialization and subsequent changes.

**Settings tab (7th tab):** Shows theme selection (Auto/Light/Dark buttons), refresh interval input, and a dashboard info section showing counts of tickers, universe size, transactions, watchlists, and alerts. Keyboard shortcut: Ctrl+7.

**New migration:** `migrations/0027_settings.sql` (key-value table, seeded with defaults)
**New queries:** `fetch_settings()`, `upsert_setting()` in db.rs
**Modified:** `src/dashboard/tabs.rs` (added Settings tab), `src/dashboard/state.rs` (settings HashMap + refresh input + 4 messages), `src/dashboard/update.rs` (4 handlers + Ctrl+7 shortcut + initial load), `src/dashboard/view.rs` (Settings tab view)
**Tests:** 8 dashboard tests passing

---

### v3.0.3 — Portfolio Transaction Log *(completed 2026-04-22)*

**Theme:** A buy/sell transaction log that records every trade. Users can add BUY or SELL transactions with ticker, shares, price, and date. Transactions display in the Portfolio tab with color-coded action labels (green for BUY, red for SELL) and a delete button per row.

**Why a transaction log?** The existing `portfolio_positions` table is a static snapshot. A transaction log captures the full trade history, enabling future P&L attribution ("Was my AAPL buy on March 5th a good entry?") and integration with the backtester ("Compare my actual trades vs what the astro signal recommended").

**New migration:** `migrations/0026_transactions.sql` with CHECK constraints on action ('BUY'/'SELL'), shares > 0, price > 0. Indexed on ticker and trade_date.

**CRUD operations:** `fetch_transactions()` (sorted by date DESC), `insert_transaction()` (RETURNING for optimistic UI), `delete_transaction()`. The toggle button cycles between BUY and SELL action modes.

**Display:** Last 20 transactions shown in a compact 7-column table (Action, Ticker, Shares, Price, Total, Date, Delete). BUY rows are green, SELL rows are red.

**New migration:** `migrations/0026_transactions.sql`
**New queries:** `fetch_transactions()`, `insert_transaction()`, `delete_transaction()` in db.rs
**Modified:** `src/dashboard/state.rs` (5 fields + 9 messages), `src/dashboard/update.rs` (9 handlers + initial load), `src/dashboard/view.rs` (transaction log in Portfolio tab)
**Tests:** 8 dashboard tests passing

---

### v3.0.2 — Strategy Builder *(completed 2026-04-22)*

**Theme:** A user-composable strategy system. Users build buy/sell rules from condition chains (e.g., "IF Astro > 70 AND RSI < 70 THEN BUY"), then run the strategy through the backtester to see how it would have performed.

**Why a strategy builder?** The v2.8 backtester uses simple astro threshold crossings. Real trading decisions combine multiple signals. The strategy builder lets users test compound hypotheses: "Does buying when astro is favorable AND RSI isn't overbought outperform either signal alone?"

**Condition types (8):**
| Condition | What it checks |
|-----------|---------------|
| AstroAbove(n) | Astro score >= n |
| AstroBelow(n) | Astro score <= n |
| RsiAbove(n) | RSI >= n |
| RsiBelow(n) | RSI <= n |
| MacdCrossUp | MACD line crosses above 0 |
| MacdCrossDown | MACD line crosses below 0 |
| PriceAboveSma50 | Close > SMA50 |
| PriceBelowSma50 | Close < SMA50 |

**Logic operators:** AND (all conditions must be true) and OR (any condition true). Buy and sell rules each have independent logic operators.

**Quick-add buttons:** Rather than a complex form, common conditions can be added with one click (+Astro>70, +RSI<70, +P>SMA50, etc.). Conditions display as removable chips with labels.

**Backtest integration:** `run_strategy_backtest()` uses the same `BacktestResult` struct as v2.8, so the results display identically. The strategy evaluates `DaySnapshot` structs that combine price, astro, and indicator data.

**New file:** `src/dashboard/strategy.rs` (~280 lines) with `Condition`, `Logic`, `Strategy`, `DaySnapshot`, `run_strategy_backtest()`
**Modified:** `src/dashboard/main.rs` (mod strategy), `src/dashboard/state.rs` (2 fields + 8 messages), `src/dashboard/update.rs` (8 handlers), `src/dashboard/view.rs` (strategy builder in Astrology tab)
**Tests:** 2 new (test_strategy_and_logic, test_strategy_or_logic), 8 dashboard tests total

---

### v3.0.1 — Advanced Charting *(completed 2026-04-22)*

**Theme:** Three enhancements to the price chart: volume bars, timeframe selector, and astro event markers.

**Volume bars:** Rendered in the bottom 18% of the chart canvas. Each bar's height is proportional to the day's volume relative to the max in the visible window. Green bars for up days (close >= previous close), red for down days. This adds the volume dimension that every serious financial chart needs.

**Timeframe selector:** 5 buttons (1M, 3M, 6M, 1Y, ALL) filter the visible data window. The `ChartTimeframe` enum maps each option to a max bar count (22, 66, 132, 252, MAX). Indicators (SMA20, SMA50, BB) are trimmed to match the visible window. Default is 6M.

**Astro event markers:** Vertical dashed lines on days with extreme astro scores. Days with astro >= 75 get a green star marker, days with astro <= 25 get a red warning marker. Markers are built from `lagrange_history` data which has date-aligned astro sub-scores. The dashed line uses 4px-on/4px-off segments.

**Chart height:** Increased from 220px to 250px to accommodate the volume bars strip without shrinking the price area.

**New enum:** `ChartTimeframe` in state.rs (5 variants, `max_bars()`, `label()`)
**New struct:** `AstroMarker` in charts.rs
**Modified:** `src/dashboard/charts.rs` (volumes + astro_markers fields, volume bar rendering, dashed line markers), `src/dashboard/state.rs` (chart_timeframe field + SetTimeframe message), `src/dashboard/update.rs` (handler), `src/dashboard/view.rs` (timeframe bar + data filtering + marker generation)
**Tests:** 8 dashboard tests passing

---

### v2.8.2 — Portfolio Gain/Loss Tracking *(completed 2026-04-22)*

**Theme:** The Portfolio tab now shows unrealized P&L for every position with color coding (green = profit, red = loss), plus the current astro score next to each holding. This answers the question every investor asks daily: "Am I making or losing money, and do the stars still favor my holdings?"

**Why P&L tracking?** The previous portfolio view only showed cost basis. Without current prices, the user had to mentally compute their gains. This is the financial equivalent of a speedometer that shows RPM but not speed. The enhanced view shows: shares, average cost, last close price, dollar P&L, percentage P&L, and the current astro score/label. The totals row at the bottom aggregates the entire portfolio.

**LATERAL JOIN for efficient fetching:**

The `fetch_portfolio_pnl()` query joins `portfolio_positions` with two `LEFT JOIN LATERAL` subqueries:
1. Latest `price_data.close` for each ticker (most recent close price)
2. Latest `astro_scores.astro_score` and `astro_label` for each ticker

This fetches all data in a single query instead of N+1 lookups per position. The `LEFT JOIN` ensures positions still display even if price or astro data is missing.

**Color coding:** P&L values use the theme's zone colors: `ZONE_OPTIMAL` (green) for gains, `ZONE_MISALIGNED` (red) for losses, `ZONE_NEUTRAL` for break-even. The same coloring applies to both per-position and total P&L.

**Astro integration:** Each position shows its current astro score and zone label. This lets the user ask: "My AAPL position is up 15%, but the astro signal just shifted to Misaligned. Time to take profits?" The astro column makes this visible without switching tabs.

**Fallback behavior:** If the P&L query returns no results (e.g., no price data yet), the view falls back to the original cost-basis-only display. The `portfolio_pnl` data loads on startup alongside all other initial data.

**New struct:** `PortfolioPnlRow` in db.rs (7 fields: ticker, shares, avg_cost, notes, last_close, astro_score, astro_label)
**New query:** `fetch_portfolio_pnl()` in db.rs (LATERAL JOIN pattern)
**Modified:** `src/dashboard/state.rs` (portfolio_pnl field + PortfolioPnlLoaded message), `src/dashboard/update.rs` (handler + initial load), `src/dashboard/view.rs` (enhanced portfolio display with 7 columns)
**Tests:** 6 dashboard tests passing (no new tests; P&L is query + UI integration)

---

### v2.8.1 — Astro-Driven Backtesting Engine *(completed 2026-04-22)*

**Theme:** A backtesting engine that tests the core thesis: "Does buying when the astro score is favorable and selling when it's unfavorable outperform buy-and-hold?" This is the ultimate accountability feature. If the astro signal doesn't predict price movement, the user should know.

**Why backtesting?** Every signal system needs empirical validation. The dashboard computes astro scores daily, but until now there was no way to ask: "Over the last 6 months of data, would following the astro signal have made money?" The backtester joins `astro_scores` and `price_data` by (ticker, date) and simulates a simple long-only strategy.

**Strategy:**
- **Buy** when astro score crosses above the buy threshold (default 65)
- **Sell** when astro score drops below the sell threshold (default 35)
- Fully invested while in a trade, fully cash while out
- No shorting, no leverage, no position sizing (all-in/all-out)
- Open positions close at the last available price

**Configurable thresholds:** The user can adjust buy/sell thresholds via text inputs in the Astrology tab. Different thresholds test different questions: strict thresholds (buy > 80, sell < 20) produce fewer trades but potentially higher quality signals; loose thresholds (buy > 55, sell < 45) produce more trades and test whether even mild astro direction has predictive value.

**Key metric: Astro Signal Accuracy (30d)**

The killer number. For every day where astro_score >= buy_threshold, the engine checks: "Did the price go higher at any point in the next 30 trading days?" The percentage of "yes" answers is the signal accuracy. 

- Above 55%: the astro signal has meaningful predictive power
- 45-55%: no better than a coin flip
- Below 45%: the astro signal is counter-predictive (contrarian use possible)

**Results displayed:**
| Metric | Description |
|--------|-------------|
| Strategy Return | Total % return of the astro-timed strategy |
| Buy & Hold Return | What you'd have made just holding (benchmark) |
| Trades | Number of buy/sell round-trips |
| Win Rate | % of trades that were profitable |
| Max Drawdown | Largest peak-to-trough decline during the backtest |
| Final Capital | Ending portfolio value ($10,000 starting) |
| Signal Accuracy | 30-day forward accuracy (the headline metric) |

**Trade log:** The last 10 trades display with buy date, buy price, sell date, sell price, and return percentage. Green for profitable trades, red for losing trades.

**Data flow:** `RunBacktest` message triggers `fetch_backtest_data()` which executes:
```sql
SELECT p.date, p.close, a.astro_score
FROM price_data p
INNER JOIN astro_scores a ON a.ticker = p.ticker AND a.score_date = p.date
WHERE p.ticker = $1 AND a.astro_score IS NOT NULL
ORDER BY p.date ASC
```
The INNER JOIN ensures only days with both price and astro data are included. Results are converted from `BacktestDayRow` (sqlx types) to `BacktestDay` (pure f64) and fed to `run_backtest()`.

**Placement:** The backtest section sits below the natal wheel and transits in the Astrology tab. This is deliberate: the user sees the current astro reading, then can test "But does this signal actually work?"

**New file:** `src/dashboard/backtest.rs` (~210 lines) with `BacktestConfig`, `BacktestDay`, `Trade`, `BacktestResult`, `run_backtest()`, `compute_signal_accuracy()`
**New struct:** `BacktestDayRow` in db.rs
**New query:** `fetch_backtest_data()` in db.rs
**Modified:** `src/dashboard/main.rs` (mod backtest), `src/dashboard/state.rs` (5 new fields + 4 messages), `src/dashboard/update.rs` (4 handlers), `src/dashboard/view.rs` (backtest section in Astrology tab)
**Tests:** 2 new (test_basic_backtest, test_signal_accuracy), 6 dashboard tests total

---

### v2.6.3 — Multiple Watchlists + CSV Export *(completed 2026-04-22)*

**Theme:** Users can now create, manage, and switch between multiple named watchlists. Each watchlist has its own set of tickers. A CSV export button lets users download their watchlist ranking data via a native file dialog.

**Why named watchlists?** A single global ticker list doesn't scale. Traders track different groups for different strategies: "Astro Favorites", "Earnings This Week", "Short Squeeze Candidates". Named watchlists let the user organize tickers by intent without losing track of anything. The Default watchlist is auto-seeded from existing active tickers so nothing breaks on migration.

**Database schema:**

The feature uses two new tables via `migrations/0025_named_watchlists.sql`:

```sql
watchlists (id SERIAL PK, name TEXT UNIQUE, created_at TIMESTAMPTZ)
watchlist_members (id SERIAL PK, watchlist_id INT FK CASCADE, ticker TEXT, added_at TIMESTAMPTZ, UNIQUE(watchlist_id, ticker))
```

The `CASCADE` on the foreign key means deleting a watchlist automatically cleans up all its member rows. The `UNIQUE(watchlist_id, ticker)` constraint prevents duplicate tickers within a single watchlist. The seed step inserts a "Default" watchlist populated from `tickers WHERE active = true`, so the migration is non-destructive.

**CRUD operations (6 functions in db.rs):**

| Function | SQL | Purpose |
|----------|-----|---------|
| `fetch_named_watchlists()` | `SELECT id, name FROM watchlists ORDER BY id` | List all watchlists |
| `fetch_watchlist_tickers()` | `SELECT ticker FROM watchlist_members WHERE watchlist_id = $1` | Get tickers in one watchlist |
| `create_watchlist()` | `INSERT INTO watchlists (name) VALUES ($1) RETURNING id, name` | Create new watchlist |
| `add_to_watchlist()` | `INSERT ... ON CONFLICT DO NOTHING` | Add ticker (idempotent) |
| `remove_from_watchlist()` | `DELETE FROM watchlist_members WHERE watchlist_id = $1 AND ticker = $2` | Remove single ticker |
| `delete_watchlist()` | `DELETE FROM watchlists WHERE id = $1` | Delete entire watchlist (cascades) |

**State management:** 12 new message variants handle the full lifecycle: load, select, create, add ticker, remove ticker, delete, export. The `NamedWatchlistsLoaded` handler auto-selects the first watchlist if none is active, loading its tickers immediately.

**Optimistic updates:** When adding or removing a ticker, the local `watchlist_tickers_list` is updated immediately before the async DB call returns. This makes the UI feel instant. The DB write happens in the background via `WatchlistMutated`.

**CSV export:** Uses `rfd` (Rust File Dialog) for a native OS save dialog and `csv` for serialization. The export includes 6 columns: Ticker, Astro Score, Astro Label, Sentiment, Sentiment Label, Short %. The user picks the save path via their OS file picker. If they cancel, no file is written.

**UI:** The Portfolio tab now includes a "Watchlists" section below the portfolio positions table. It shows: (1) watchlist selector buttons (active watchlist marked with triangle), (2) "create new" text input + button, (3) ticker list with per-ticker remove buttons, (4) "add ticker" text input, (5) action buttons for delete and CSV export.

**New migration:** `migrations/0025_named_watchlists.sql`
**New dependencies:** `rfd = "0.15"`, `csv = "1"` in Cargo.toml
**Modified:** `src/dashboard/db.rs` (NamedWatchlist struct + 6 CRUD functions), `src/dashboard/state.rs` (5 new fields + 12 messages), `src/dashboard/update.rs` (12 handlers + export_watchlist_csv async fn), `src/dashboard/view.rs` (watchlist manager UI in Portfolio tab)
**Tests:** 4 passing (no new tests; CRUD is integration-level)

---

### v2.6.2 — Sector Heat Map *(completed 2026-04-22)*

**Theme:** A canvas-rendered heat map widget showing each sector colored by its average astro score. This gives an at-a-glance view of which sectors the stars favor and which face headwinds.

**Why a heat map?** The Universe Explorer table shows individual tickers. But institutional thinking starts with sectors: "Is Technology in a favorable astro zone this week? What about Healthcare?" A heat map answers this instantly through color: green sectors are favorable, red are stressed, yellow are neutral. The proportional cell sizing also reveals sector concentration: if Technology has 180 tickers and Utilities has 12, the visual weight communicates the universe composition.

**Color interpolation:** The `score_to_color(score: f64)` function uses two-phase linear interpolation:
- Score 0-50: Red (255,80,80) to Yellow (255,220,80) -- danger to caution
- Score 50-100: Yellow (255,220,80) to Green (80,200,120) -- caution to favorable

This avoids the problem with single-phase interpolation where mid-range values become muddy brown. The two-phase approach ensures yellow at the midpoint, which is intuitive for financial dashboards.

**Proportional cell width:** Each sector's cell width is proportional to its ticker count relative to the total. A sector with 180/487 tickers gets 37% of the canvas width. This communicates both the score (color) and the sector's universe weight (size) simultaneously.

**Canvas rendering:** `SectorHeatMap` implements `canvas::Program<Message>`, following the same pattern as `FearGreedGauge` and `PriceChart`. Each cell renders 3 text labels: sector name (truncated at 18 chars), average score, and ticker count. Text color is white for dark backgrounds (low scores) and dark gray for light backgrounds (high scores), using a luminance threshold at score 65.

**Data source:** `fetch_sector_summaries()` queries the `company_metadata` + `astro_scores` + `lagrange_history` tables, grouping by sector. Returns `Vec<SectorSummary>` with sector name, average astro score, average Lagrange score, and ticker count.

**Placement:** The heat map sits at the top of the Universe tab, above the zone filter bar. Canvas height is 100px, full width.

**New file:** `src/dashboard/heatmap.rs` (~130 lines)
**Modified:** `src/dashboard/main.rs` (mod heatmap), `src/dashboard/db.rs` (SectorSummary struct + fetch_sector_summaries), `src/dashboard/state.rs` (sector_summaries field + SectorSummariesLoaded message), `src/dashboard/update.rs` (handler + initial load), `src/dashboard/view.rs` (heat map in Universe tab)
**Tests:** 4 passing (no new tests; canvas rendering is visual)

---

### v2.6.1 — Universe Explorer Panel *(completed 2026-04-22)*

**Theme:** A full-featured data explorer for the entire scored universe of 400+ tickers. Paginated table with zone and sector filters, sorted by astro score (astro-first column order). This is where the user discovers what the stars favor across the entire market, not just their watchlist.

**Why a Universe Explorer?** The existing watchlist shows ~10-20 tickers the user manually selected. But the scraper pipeline scores 400+ tickers. The Universe Explorer exposes the full scored universe, letting users discover tickers they might never have considered. "What's the highest astro-scored Healthcare ticker right now?" Previously unanswerable. Now it's one filter click.

**Server-side pagination:** The query uses SQL `LIMIT $4 OFFSET $5` with configurable page size (50 rows). This keeps memory usage constant regardless of universe size. The total count is fetched separately via `fetch_universe_count()` so the UI can show "Page 3 of 10".

**Optional filter pattern:** The SQL uses `$1::text IS NULL OR zone = $1` for both zone and sector filters. This avoids needing separate queries for filtered vs unfiltered states. When the filter is `None`, the `$1::text IS NULL` clause short-circuits and returns all rows.

**11-column table (astro-first ordering):**

| # | Column | Source | Why |
|---|--------|--------|-----|
| 1 | Rank | Row number | Position context |
| 2 | Ticker | company_metadata | Identity |
| 3 | Sector | company_metadata | Sector grouping |
| 4 | Astro | astro_scores | THE lead signal |
| 5 | Zone | astro_scores | Categorized astro signal |
| 6 | Lagrange | lagrange_history | Composite score |
| 7 | L-Zone | lagrange_history | Composite category |
| 8 | Fin | lagrange_history | Financial sub-score |
| 9 | Macro | lagrange_history | Macro sub-score |
| 10 | Short | lagrange_history | Short interest sub-score |
| 11 | Concordance | lagrange_history | Astro-financial agreement |

**Zone filter bar:** 6 buttons (All, Optimal, Favorable, Neutral, Unfavorable, Misaligned) that filter by the astro zone column. Clicking "Optimal" shows only tickers with astro_label = 'Optimal'. Clicking "All" clears the filter.

**Sector filter bar:** Dynamic buttons generated from `fetch_available_sectors()`. Only sectors that exist in the scored universe appear as filter options. This avoids showing empty sector filters.

**State fields:** `universe_rows`, `universe_total`, `universe_page`, `universe_filter_zone`, `universe_filter_sector`, `universe_sectors`. All 8 message handlers delegate to `refresh_universe()` which batches the page fetch and count fetch into a single `Task::batch`.

**Initial load:** Added to the `TickersLoaded` handler's initial batch alongside all other data fetches. The universe loads in the background on dashboard startup.

**New structs:** `UniverseRow` in db.rs (11 fields, all optional except ticker), `SectorSummary` in db.rs
**New queries:** `fetch_universe_page()`, `fetch_universe_count()`, `fetch_available_sectors()`, `fetch_sector_summaries()`
**Modified:** `src/dashboard/state.rs` (6 new fields + 8 messages), `src/dashboard/update.rs` (8 handlers + refresh_universe method + initial load), `src/dashboard/view.rs` (full Universe tab rebuild from stub)
**Tests:** 4 passing (no new tests; pagination is integration-level)

---

### v2.4.3 — Technical Pattern Recognition *(completed 2026-04-22)*

**Theme:** Automated detection of classic chart patterns (Golden Cross, Death Cross, Double Top, Double Bottom, Support, Resistance) displayed in the Overview tab. This gives users an instant read on the technical picture alongside the astrological and fundamental signals.

**Why pattern recognition?** The dashboard already shows RSI, MACD, Bollinger Bands, and moving averages as raw numbers. But experienced traders think in terms of patterns: "Is this a double bottom?" or "Did the golden cross fire?" Automating this detection saves the user from mentally computing crossovers and comparing price levels. It also sets up v3.0's strategy builder, which will let users combine astro + technical patterns into custom buy/sell rules.

**Patterns detected:**

| Pattern | Detection Method | Signal |
|---------|-----------------|--------|
| Golden Cross | SMA50 crosses above SMA200 in last 5 bars | Bullish — long-term trend reversal |
| Death Cross | SMA50 crosses below SMA200 in last 5 bars | Bearish — long-term trend reversal |
| Double Top | Two peaks within 2% of each other, 5+ bars apart, in last 60 bars | Bearish — price rejected at same level twice |
| Double Bottom | Two troughs within 2% of each other, 5+ bars apart, in last 60 bars | Bullish — price supported at same level twice |
| Support | 3+ local lows within 1.5% cluster in last 90 bars | Bullish — floor the price keeps bouncing off |
| Resistance | 3+ local highs within 1.5% cluster in last 90 bars | Bearish — ceiling the price keeps hitting |

**Detection algorithms:**

*Local extrema identification:* All patterns use 5-bar neighborhoods to find peaks and troughs. A bar is a local high if it's higher than both its 2 left and 2 right neighbors. This avoids noise from single-bar spikes.

*Golden/Death Cross:* The SMA200 was added to the `Indicators` struct (previously only SMA20 and SMA50 existed). The detection checks only the last 5 bars for a crossover event, avoiding stale signals from crossovers months ago. Once a cross is found, the scan stops to avoid duplicate detection.

*Double Top/Bottom:* After identifying all peaks (or troughs) in the last 60 bars, the algorithm checks each pair for: (1) separation of at least 5 bars, and (2) price difference under 2%. The 2% threshold accounts for normal price noise while still catching meaningful double formations.

*Support/Resistance:* Uses a clustering algorithm: sort all local lows (or highs), then slide through the sorted list checking if 3+ consecutive values fall within 1.5% of the base value. Returns the average of the best cluster. This is more robust than simple min/max detection.

**Display:** Patterns appear in the Overview tab's left column, between the indicator row and the macro strip. Bullish patterns show in green (`ZONE_OPTIMAL`) with a ▲ icon, bearish in red (`ZONE_MISALIGNED`) with a ▼ icon. If no patterns are detected, a simple "Patterns: none detected" message displays.

**SMA200 addition:** The `Indicators` struct in `src/indicators.rs` now includes `sma200: Vec<Option<f32>>`, computed via the existing `sma()` function with period 200. This is used by both the pattern detector and will be used by v3.0's chart annotations.

**New file:** `src/dashboard/patterns.rs` (~190 lines) — `Pattern` enum, `detect_patterns()`, `detect_cross()`, `detect_double_patterns()`, `detect_support_resistance()`, `find_cluster()`
**Modified:** `src/indicators.rs` (added `sma200` field + compute), `src/dashboard/main.rs` (mod patterns), `src/dashboard/view.rs` (patterns section in Overview tab)
**Tests:** 2 new (test_golden_cross_detected, test_support_cluster), total: 40 passing (38 lib + 4 dashboard)

---

### v2.4.2 — Comparative Analysis *(completed 2026-04-22)*

**Theme:** Side-by-side ticker comparison within the Fundamentals tab. Users can add up to 4 tickers and see their key metrics in a comparison table with astro scores included. This is where the financial verification principle becomes actionable: "The stars say NVDA is favorable and META is misaligned. But what do the numbers say?"

**Why comparative analysis?** Individual ticker analysis is valuable, but investment decisions often involve choosing between alternatives. "Should I buy NVDA or AMD?" requires seeing both side by side. By including astro score and zone in the comparison table, users can see whether the astrological signal and fundamental quality agree across multiple tickers simultaneously.

**How it works:**

The user types a ticker into a text input and clicks "Add" (or presses Enter). Up to 4 tickers can be compared at once. Each ticker appears as a removable chip button (e.g., "AAPL ✕"). Clicking the ✕ removes it from the comparison and refreshes the table.

**SQL: LATERAL JOIN for efficient multi-ticker fetch**

The comparison query uses `UNNEST($1::text[]) AS t(ticker)` with two `LEFT JOIN LATERAL` subqueries to get the latest `fundamental_metrics` row and latest `astro_scores` row for each ticker in a single round-trip. This avoids N+1 queries and preserves the input order via `array_position()`.

```sql
SELECT f.ticker, f.pe_ratio, f.pb_ratio, ..., a.astro_score, a.astro_label
FROM UNNEST($1::text[]) AS t(ticker)
LEFT JOIN LATERAL (
    SELECT * FROM fundamental_metrics fm
    WHERE fm.ticker = t.ticker ORDER BY fm.fetch_date DESC LIMIT 1
) f ON true
LEFT JOIN LATERAL (
    SELECT astro_score, astro_label FROM astro_scores asc_
    WHERE asc_.ticker = t.ticker ORDER BY asc_.score_date DESC LIMIT 1
) a ON true
ORDER BY array_position($1::text[], t.ticker)
```

**12 metrics compared:**
| Category | Metrics |
|----------|---------|
| Valuation | P/E, P/B, P/S, EV/EBITDA, PEG |
| Profitability | ROE, Net Margin |
| Health | Debt/Equity, FCF, Market Cap |
| Astrology | Astro Score, Astro Zone |

**Data types:** `CompareRow` struct in `db.rs` with 12 fields (`Option<f64>` for ratios, `Option<i64>` for monetary values, `Option<String>` for labels). Missing data displays as "---".

**New struct:** `CompareRow` in `src/dashboard/db.rs`
**New query:** `fetch_compare_data(pool, tickers: Vec<String>)` in `db.rs`
**Modified:** `src/dashboard/state.rs` (compare_tickers, compare_input, compare_data fields + 4 new messages), `src/dashboard/update.rs` (CompareInput/Add/Remove/DataLoaded handlers + refresh_compare method), `src/dashboard/view.rs` (comparison section in Fundamentals tab)
**Tests:** 40 passing (no new tests; comparison is UI + query integration)

---

### v2.4.1 — AI Agent Personas *(completed 2026-04-22)*

**Theme:** Four legendary investor personas that review each ticker through their own investment philosophy, including their take on the astrological reading. This is where the three product pillars converge: astrology leads, agents interpret through their philosophical lens, fundamentals verify. Principle #5 (Circle of Competence): each agent stays in their framework. Buffett talks moats, not charts. Graham talks valuation, not growth.

**Why template-based first, not LLM?** Template analysis is: (1) free — no API costs, (2) deterministic — same inputs always produce the same output, (3) instant — no network round-trip, (4) always available — works offline. The LLM path will be added in v2.8 as an optional upgrade when an API key is configured. Templates are not inferior — they encode exactly the investment criteria each philosopher would use, with consistent scoring.

**The four agents:**

| Agent | Philosophy | Key Metrics | Astro Relationship |
|-------|-----------|-------------|-------------------|
| **Buffett** | Moat + FCF + Margin of Safety | ROE > 15%, D/E < 0.5, FCF > $10B, P/E < 25, Net Margin > 20% | "When the cosmos and the balance sheet agree, I pay attention." Acknowledges astro when it confirms fundamentals. |
| **Graham** | Deep Value + Quantitative Discipline | P/E < 15, P/B < 1.5, P/E×P/B < 22.5, Current Ratio > 2.0, Dividend Yield | "My margin of safety analysis doesn't depend on planetary positions." Politely skeptical. Sticks to numbers. |
| **Lynch** | Growth at Reasonable Price (GARP) | PEG < 1.0, P/E < 20, Revenue size, Net Margin > 15%, D/E < 0.5 | "I see astrology as a proxy for market psychology." Uses it as sentiment indicator, not conviction driver. |
| **Munger** | Quality + Mental Models + Patience | ROE > 25%, Op Margin > 30%, EV/EBITDA < 10, FCF > $5B, D/E < 0.3 | "I view this the way I view any unfamiliar mental model: with curiosity, not conviction." One data point among many. |

**Internal scoring system:** Each agent evaluates 5 metrics, assigning +3 to -3 points per metric based on their thresholds. The total score maps to a verdict:

```
AgentVerdict: StrongBuy / Buy / Hold / Sell / StrongSell / InsufficientData
```

Each agent's score thresholds differ — Buffett is generous (StrongBuy at 5+), Graham is strict (StrongBuy at 7+), reflecting their real-world investment discipline. When fundamentals data is missing, all agents return `InsufficientData` rather than guessing.

**Narrative generation:** Each agent has a `build_*_narrative()` function that constructs 2-4 sentences of analysis. The narrative highlights the most notable metrics (ROE > 20% for Buffett, PEG < 1.0 for Lynch) and provides the agent's overall assessment. Extreme scores (very high or very low) trigger additional commentary.

**Astro take:** Each agent's `astro_take` field provides their philosophical reaction to the astrological signal. This varies based on: (1) the astro score value, (2) the concordance (if available), and (3) the dominant theme (if available). This is the killer feature — seeing how Buffett and Graham react differently to the same astrological reading for the same ticker.

**UI integration:** The Fundamentals tab now has an "Ask the Council" section with 4 persona buttons. The active persona is highlighted with brackets (e.g., `[Buffett]`). Clicking a persona instantly computes and displays their analysis. The verdict is color-coded: green for Buy/StrongBuy, yellow for Hold, red for Sell/StrongSell. Key metrics show the metric name, value, and the agent's assessment in a compact table.

**Recomputation:** When fundamentals data loads for a new ticker (via `FundamentalsLoaded` message), the active agent's analysis is automatically recomputed with the new data. Switching tickers clears the agent analysis (the user can re-select a persona to analyze the new ticker).

**New file:** `src/dashboard/agents.rs` (~815 lines) — `AgentPersona`, `AgentContext`, `AgentAnalysis`, `AgentVerdict`, `analyze()`, 4 analysis functions + 4 narrative builders + `format_large_number()`
**Modified:** `src/dashboard/main.rs` (mod agents), `src/dashboard/state.rs` (active_agent + agent_analysis fields + AgentSelected message), `src/dashboard/update.rs` (AgentSelected handler + recompute_agent_if_active method), `src/dashboard/view.rs` (agent buttons + analysis display in Fundamentals tab)
**Tests:** 40 passing (no new tests; agents are template-based with deterministic output)

---

### v2.2.4 — Two-Column Overview Layout *(completed 2026-04-22)*

**Theme:** Optimize the Overview tab's information density by splitting it into a two-column layout. The left column (60% width) shows the price chart, Lagrange sparkline, technical indicators, and macro strip. The right column (40% width) shows signal intelligence bullets and the watchlist ranking. Gauges remain full-width above both columns.

**Why two columns?** The Overview tab had 8 sections stacked vertically, requiring significant scrolling to see signals after the chart. A two-column layout puts the chart and signals side-by-side, so you can see the price action and the system's interpretation simultaneously. This is how Bloomberg and FinceptTerminal arrange their overview panels.

**How it works in Iced 0.13:** The layout uses `Row` with two `container` children, each set to `Length::FillPortion(3)` and `Length::FillPortion(2)` respectively. This gives a 60/40 split that adapts to any window width. The gauges row sits above the two-column `Row` in the outer `Column`.

**Modified:** `src/dashboard/view.rs` (Overview tab refactored from single column to `Row` with `FillPortion`)
**Tests:** 40 passing (no new tests needed for layout)

---

### v2.2.3 — Circadian Theme System *(completed 2026-04-22)*

**Theme:** Replace the binary Light/Dark toggle with a 3-mode circadian theme system that can automatically adapt to time of day. The button now cycles: Auto -> Light -> Dark -> Auto.

**Why circadian?** A financial dashboard used at 6am looks different from one used at midnight. In Auto mode, the theme follows the user's local time: Dawn (5-8am) and Day (9am-4pm) use the Light theme for high-contrast readability in daylight. Dusk (5-8pm) and Night (9pm-4am) use the Dark theme for reduced eye strain in low-light conditions. The user can always override to permanently Light or Dark.

**How it works:**

The `ThemeMode` enum has 3 states: `Auto`, `AlwaysLight`, `AlwaysDark`. The `CircadianPhase` enum has 4 states: `Dawn`, `Day`, `Dusk`, `Night`. In Auto mode, `CircadianPhase::current()` uses `chrono::Local::now().hour()` to determine the phase. The 30-second `Tick` subscription updates the theme on each tick when in Auto mode, so the dashboard transitions smoothly at phase boundaries.

**4 Circadian Phases:**
| Phase | Hours | Iced Theme | Character |
|-------|-------|------------|-----------|
| Dawn | 05:00-08:59 | Light | Warm, high contrast for early market prep |
| Day | 09:00-16:59 | Light | Maximum readability during trading hours |
| Dusk | 17:00-20:59 | Dark | Transition to reduced eye strain |
| Night | 21:00-04:59 | Dark | Deep navy for after-hours analysis |

**3 User Modes:**
| Mode | Behavior | Button Label |
|------|----------|-------------|
| Auto | Follows local time through 4 phases | "Theme: Auto" |
| Always Light | Forces Day phase regardless of time | "Theme: Light" |
| Always Dark | Forces Night phase regardless of time | "Theme: Dark" |

**New types:** `ThemeMode`, `CircadianPhase` enums in `theme.rs`
**New functions:** `active_phase()`, `iced_theme()`, `CircadianPhase::from_hour()`, `CircadianPhase::current()`
**Modified:** `src/dashboard/theme.rs` (new types + functions), `src/dashboard/state.rs` (theme_mode field), `src/dashboard/update.rs` (toggle cycles 3 modes, tick updates auto), `src/dashboard/view.rs` (button label shows mode)
**Tests:** 40 passing

---

### v2.2.2 — DCF Intrinsic Value Calculator *(completed 2026-04-22)*

**Theme:** A two-stage Discounted Cash Flow calculator that computes intrinsic value per share and margin of safety. This is Principle #7 (Margin of Safety) made interactive. The margin of safety percentage is the headline, not the raw intrinsic value. Buffett's language, Buffett's framing.

**What is DCF?** Discounted Cash Flow is the most widely used intrinsic value method. It answers: "If this company generates X dollars of free cash flow, growing at Y% per year, what is the present value of all those future cash flows?" The result is the company's intrinsic value, independent of what the stock market currently prices it at.

**Two-stage model:**
- **Stage 1 (Growth Period):** FCF grows at the user's specified growth rate (default 10%) for N years (default 5). Each year's projected FCF is discounted back to present value using the WACC.
- **Stage 2 (Terminal Value):** After the growth period, the company is assumed to grow at a perpetuity rate (default 2.5%) forever. The Gordon Growth Model computes the terminal value: `TV = FCF * (1+g) / (r-g)`, which is then discounted back to present value.

The sum of Stage 1 PVs + Stage 2 PV = Enterprise Value. Divided by shares outstanding = Intrinsic Value Per Share.

**Margin of Safety = (Intrinsic - Price) / Intrinsic x 100**
- Positive: the stock appears undervalued. "You're paying less than it's worth."
- Negative: the stock appears overvalued. "You're paying more than it's worth."

**User-editable inputs:**
| Input | Default | Description |
|-------|---------|-------------|
| Growth % | 10 | Annual FCF growth rate for Stage 1 |
| Years | 5 | Number of growth years |
| Terminal % | 2.5 | Perpetuity growth rate (GDP proxy) |
| WACC % | 10 | Discount rate / required return |

The calculator auto-computes when fundamentals data loads (FCF + shares from v2.2.1). Users can adjust inputs and press "Compute" to re-run with different assumptions.

**Color coding:** Margin of safety is green when positive (undervalued) and red when negative (overvalued), using `ZONE_OPTIMAL` and `ZONE_MISALIGNED` from the existing color token system.

**Edge case handling:** If discount rate <= terminal growth rate (which makes the Gordon Growth Model mathematically undefined, producing infinity), the calculator falls back to a 20x multiple on the final-year FCF. If FCF is negative (company is burning cash), the DCF section shows "Requires FCF data" rather than a misleading negative intrinsic value.

**New file:** `src/dashboard/dcf.rs` (~130 lines, `DcfInputs`, `DcfResult`, `compute_dcf()`)
**Modified:** `src/dashboard/main.rs` (mod dcf), `src/dashboard/state.rs` (DCF input fields + result + 5 new messages), `src/dashboard/update.rs` (DCF handlers + `compute_dcf_if_ready()`), `src/dashboard/view.rs` (DCF section in Fundamentals tab)
**Tests:** 2 new (test_dcf_basic, test_dcf_margin_of_safety), total: 40 passing

---

### v2.2.1 — Fundamental Analysis Panel + Scraper *(completed 2026-04-22)*

**Theme:** Add fundamental financial metrics to the dashboard. Market cap, P/E, P/B, EV/EBITDA, ROE, debt/equity, FCF, dividend yield, and more. This is the financial verification layer that lets users check whether the astrological signal is supported by the company's actual financial health. Principle #6 (Owner Economics) and Principle #2 (Candor): show the economics plainly.

**Why fundamentals matter for this product:** The astrological score is the lead signal. But a score of 92 means something very different for a company with ROE of 147% and no debt (AAPL) versus a company with negative FCF and 5x debt/equity. The fundamentals panel provides the verification layer. In v2.4, the AI agent personas will interpret these numbers through their investment philosophy.

**Data source: Financial Modeling Prep (FMP)**

Two FMP endpoints provide all the metrics we need:
- `/v3/key-metrics-ttm/{ticker}` — trailing twelve month valuation ratios (P/E, P/B, EV/EBITDA, etc.)
- `/v3/ratios-ttm/{ticker}` — profitability margins (net margin, operating margin)

Each ticker costs 2 API calls. With the FMP free tier (250 req/day) shared with the existing IPO enrichment module, we budget 60 tickers per daily run (120 calls). Watchlist tickers are prioritized via `watchlist_priority_sql()`.

**Per-share to absolute conversion:** FMP's TTM endpoint returns some values as per-share figures (FCF per share, revenue per share, etc.). We multiply by `shares_outstanding` to derive absolute dollar values for display. This is more intuitive for users than per-share figures.

**22 metrics stored and displayed, organized in two columns:**

**Left column — Valuation:**
| Metric | Source | What it tells you |
|--------|--------|-------------------|
| Market Cap | key-metrics | Company size |
| P/E Ratio | key-metrics | Price relative to earnings |
| P/B Ratio | key-metrics | Price relative to book value |
| P/S Ratio | key-metrics | Price relative to revenue |
| EV/EBITDA | key-metrics | Enterprise value relative to operating earnings |
| PEG Ratio | key-metrics | P/E adjusted for growth rate |
| P/FCF | key-metrics | Price relative to free cash flow |
| EPS | derived | Earnings per share |
| Div Yield | key-metrics | Annual dividend as % of price |

**Right column — Profitability & Health:**
| Metric | Source | What it tells you |
|--------|--------|-------------------|
| ROE | key-metrics | Return on shareholder equity |
| ROA | key-metrics | Return on total assets |
| Net Margin | ratios | Profit as % of revenue |
| Op Margin | ratios | Operating profit as % of revenue |
| Debt/Equity | key-metrics | Financial leverage |
| Current Ratio | key-metrics | Short-term liquidity |
| Revenue | derived | Total annual revenue |
| Net Income | derived | Total annual profit |
| FCF | derived | Free cash flow (cash available after capex) |

**Empty state:** When no fundamentals data exists for a ticker, the panel shows an informative message with the data source and instructions to run the scraper with an FMP API key.

**Budget tracking:** The scraper checks `fetch_log` for today's FMP calls before starting. If the budget is exhausted (by IPO enrichment), fundamentals are skipped gracefully with a console message. The module also checks `fundamental_metrics` for today's date to skip tickers already fetched.

**New migration:** `migrations/0024_fundamentals.sql` — `fundamental_metrics` table with 22 columns, unique on (ticker, fetch_date), indexed for fast latest-row lookups.
**New model:** `FundamentalMetric` struct in `src/models.rs` (22 fields, all `Option` for graceful handling of missing data)
**New scraper module:** `src/scraper/fundamentals.rs` (~200 lines) — `fetch_fundamentals()` entry point, `FmpKeyMetrics` + `FmpRatios` deserialization, per-share to absolute conversion, upsert with ON CONFLICT
**Modified:** `src/scraper/main.rs` (mod fundamentals, Phase 2 step 2.7), `src/dashboard/db.rs` (fetch_fundamentals query), `src/dashboard/state.rs` (fundamentals field + FundamentalsLoaded message), `src/dashboard/update.rs` (handler + fetch_all batch), `src/dashboard/view.rs` (two-column metrics grid in Fundamentals tab)
**Tests:** 40 passing (no new tests for scraper/view, tested via integration)

---

### v2.0.8 — Score Calibration *(completed 2026-04-22)*

**Theme:** Fix the scoring distribution so every ticker gets a meaningful, differentiated score instead of clustering at 0 or 100. Two fixes: sigmoid normalization for the numeric score, and theme-score reconciliation so the narrative theme never contradicts the number.

**The problem:** The old scoring formula was `(50 + delta_sum + moon_mod).clamp(0, 100)`. This looks reasonable until you realize the typical ticker has 53 active aspects, each contributing up to 27 points (base 15 x 1.5 applying x 1.2 dignity). The delta_sum routinely exceeds +/- 200 points. With only 50 points of headroom above and below center, the `.clamp(0, 100)` saturated constantly. Database analysis revealed: 368 tickers (23.4%) scored exactly 0, 225 tickers (14.3%) scored exactly 100. Over a third of the scored universe had no meaningful differentiation. A score of 0 for GOOGL and 0 for a random micro-cap conveys no information.

**Fix 1: Logistic sigmoid normalization (`natal.rs`)**

The new formula replaces linear clamping with a logistic sigmoid:

```
score = 100 / (1 + e^(-k * x))
```

where `x = delta_sum + moon_mod` and `k = 0.04`.

**Why a sigmoid?** A sigmoid (S-curve) maps any real number to a bounded range without ever hitting the boundaries. It naturally compresses extreme values while preserving the most resolution in the middle range where differentiation matters most.

The steepness parameter `k = 0.04` was chosen to match the observed delta_sum distribution:
- At x = 0 (neutral), the score is exactly 50.0
- At x = +/-50 (moderately bullish/bearish), the score is ~88 / ~12
- At x = +/-100 (strongly directional), the score is ~98 / ~2
- The score asymptotically approaches but never reaches 0 or 100

This means every ticker gets a unique score that reflects the actual magnitude of its astrological signal. A ticker with 60 aspects slightly favoring growth will score differently from one with 60 aspects strongly favoring growth.

Mercury retrograde cap (`MERCURY_RX_CAP = 65.0`) is applied after sigmoid: `base_score.min(65.0)`. This preserves the principle that no ticker should score "Optimal" during Mercury retrograde, regardless of how favorable the other aspects are.

**Fix 2: Theme-score reconciliation (`interpretation.rs`)**

The horoscope reading engine has two independent classification systems:
1. **Numeric score** (natal.rs): sums signed aspect deltas. Jupiter trine natal Venus = positive, Saturn square natal Sun = negative. The sign and magnitude of each aspect delta determines the final number.
2. **Dominant theme** (interpretation.rs): counts unsigned aspect weights by planet category. Jupiter/Venus aspects feed "Growth & Expansion", Saturn/Pluto aspects feed "Caution & Restructuring", regardless of whether the individual aspect is harmonious or challenging.

These can disagree. Example: a ticker with many Saturn aspects (driving the theme toward "Caution") where most of those Saturn aspects happen to be harmonious trines and sextiles (driving the score upward). Before this fix, ABUS had score 96 + theme "Caution & Restructuring", which is contradictory and confusing.

The reconciliation logic overrides the theme when the numeric score clearly contradicts it:
- Score >= 65 cannot be "Caution & Restructuring" (overridden to "Growth & Expansion")
- Score <= 35 cannot be "Growth & Expansion" (overridden to "Caution & Restructuring")

After reconciliation, the theme is further refined with intensity modifiers:
| Theme | Score Range | Refined Theme |
|-------|------------|---------------|
| Growth & Expansion | 76+ | Optimal Growth & Expansion |
| Growth & Expansion | 56-75 | Growth & Expansion |
| Growth & Expansion | below 56 | Mild Growth & Expansion |
| Caution & Restructuring | 0-20 | Extreme Caution & Restructuring |
| Caution & Restructuring | 21-35 | Caution & Restructuring |
| Caution & Restructuring | above 35 | Mild Caution & Restructuring |
| Deep Transformation | score > 50 | Constructive Transformation |
| Deep Transformation | score <= 50 | Destructive Transformation |
| Innovation & Disruption | score > 50 | Positive Disruption |
| Innovation & Disruption | score <= 50 | Volatile Disruption |

**Verification results (1,573 scored tickers):**

Before calibration:
```
Exact 0s:   368 (23.4%)    Exact 100s: 225 (14.3%)
Mean: 42.3    Stddev: 37.3    Range: 0 to 100
```

After calibration:
```
Exact 0s:   0 (0%)         Exact 100s: 0 (0%)
Mean: 43.7    Stddev: 32.8    Range: 0.1 to 99.9
```

Well-known ticker differentiation (these were previously clustered at extremes):
| Ticker | Old Score | New Score | Theme |
|--------|----------|----------|-------|
| MSFT | 100 | 98.8 | Optimal Growth & Expansion |
| NVDA | 100 | 94.9 | Optimal Growth & Expansion |
| AMZN | 100 | 63.8 | Growth & Expansion |
| AAPL | 100 | 62.2 | Growth & Expansion |
| META | 100 | 54.0 | Mild Growth & Expansion |
| TSLA | 100 | 51.0 | Mild Growth & Expansion |
| BRK.B | 0 | 24.9 | Caution & Restructuring |
| V | 0 | 17.7 | Caution & Restructuring |
| JPM | 0 | 10.5 | Caution & Restructuring |
| GOOGL | 0 | 5.4 | Extreme Caution & Restructuring |

Concordance verification (Lagrange composite scores with new calibration):
| Ticker | Lagrange | Label | Concordance |
|--------|---------|-------|-------------|
| AAPL | 63.2 | Favorable | Mild Confirm |
| MSFT | 79.3 | Optimal | Strong Confirm |
| NVDA | 78.9 | Optimal | Strong Confirm |
| GOOGL | 38.2 | Unfavorable | Divergence |

**Modified:** `src/astrology/natal.rs` (sigmoid normalization, ~20 lines changed), `src/astrology/interpretation.rs` (theme-score reconciliation, ~25 lines added to `classify_dominant_theme()`)
**No new migrations.** Score recalibration is purely in the computation layer. Re-running the scraper recomputes all scores with the new formula.
**Tests:** 35 passing (3 pre-existing Swiss Ephemeris FFI failures unrelated to scoring)

---

### v2.0.7 — Keyboard Navigation *(completed 2026-04-22)*

**Theme:** Power-user keyboard shortcuts that make the dashboard feel like a real terminal. Ctrl+1..6 for instant tab switching, Ctrl+T to jump into the search box, Ctrl+R to refresh, and Escape to clear. No mouse required for the core navigation loop.

**Why keyboard shortcuts?** A Bloomberg-class terminal lives and dies by keyboard efficiency. Traders don't mouse between tabs. With 6 tabs and a search box, the most common navigation actions should be a single keystroke away. This completes the v2.0 "Oracle's Desk" foundation by making the tab system from v2.0.6 actually fast to use.

**How it works in Iced 0.13:** The `subscription()` method now returns a `Subscription::batch` of two subscriptions: the existing 30-second auto-refresh timer and a new `iced::keyboard::on_key_press(handle_key_press)` subscription. Iced's keyboard subscription requires a bare `fn` pointer (not a closure) with signature `fn(Key, Modifiers) -> Option<Message>`. Returning `Some(msg)` fires the message; returning `None` passes the key through to the focused widget (so regular typing in the search box isn't intercepted).

**The `handle_key_press` function** checks `modifiers.control()` first. If Ctrl is held, it matches `Key::Character("1")` through `Key::Character("6")` to dispatch `Message::TabSelected(Tab::*)`, `"t"/"T"` for `Message::FocusSearch`, and `"r"/"R"` for `Message::RefreshNow`. Without Ctrl, `Key::Named(Named::Escape)` dispatches `Message::EscapePressed`.

**Programmatic focus with `text_input::Id`:** The search box now has a stable ID (`"ticker-search"`) assigned via `.id(SEARCH_INPUT_ID)`. When Ctrl+T fires `Message::FocusSearch`, the update handler returns `text_input::focus(SEARCH_INPUT_ID)`, which is an Iced `Task` that programmatically moves keyboard focus to the search box. This is the same pattern Iced uses internally for focus management.

**Shortcut summary:**

| Shortcut | Action | Message |
|----------|--------|---------|
| Ctrl+1 | Switch to Astrology tab | `TabSelected(Tab::Astrology)` |
| Ctrl+2 | Switch to Overview tab | `TabSelected(Tab::Overview)` |
| Ctrl+3 | Switch to Universe tab | `TabSelected(Tab::Universe)` |
| Ctrl+4 | Switch to Fundamentals tab | `TabSelected(Tab::Fundamentals)` |
| Ctrl+5 | Switch to Research tab | `TabSelected(Tab::Research)` |
| Ctrl+6 | Switch to Portfolio tab | `TabSelected(Tab::Portfolio)` |
| Ctrl+T | Focus ticker search box | `FocusSearch` |
| Ctrl+R | Refresh all data | `RefreshNow` |
| Escape | Clear search input + autocomplete | `EscapePressed` |

**New messages:** `FocusSearch`, `EscapePressed`
**New constant:** `SEARCH_INPUT_ID` in `update.rs`
**Modified:** `src/dashboard/update.rs` (keyboard imports, handle_key_press fn, FocusSearch/EscapePressed handlers, subscription batch), `src/dashboard/state.rs` (2 new Message variants), `src/dashboard/view.rs` (search input `.id()`)
**Tests:** 38 passing (keyboard subscription is integration-tested via Iced's event loop)

---

### v2.0.6 — Tab Navigation System *(completed 2026-04-22)*

**Theme:** Transform the single-page dashboard into a 6-tab interface with Astrology as the flagship first tab. Every section that was previously stacked in one long scroll is now organized into focused tabs, each serving a specific analytical purpose.

**Why tabs?** The dashboard had grown to 18+ sections in one scrollable view. Important information was buried. The astrology reading (the product's core differentiator) was sandwiched between indicator rows and price tables. Tabs solve this by putting Astrology front and center as the default view, with financial data organized into purpose-specific tabs.

**The 6 tabs (in order):**

| Tab | Default | Content | Purpose |
|-----|---------|---------|---------|
| **Astrology** | YES | Natal wheel, transits table, moon phase, horoscope | The lead signal |
| Overview | | Gauges, price chart, sparkline, indicators, signals, watchlist | Quick market pulse |
| Universe | | Alerts (future: full scored universe explorer) | Cross-ticker view |
| Fundamentals | | Earnings, price table (future: P/E, DCF, agents) | Financial verification |
| Research | | News, 8-K filings, insider trades, holdings | Due diligence |
| Portfolio | | Portfolio positions, macro strip | Portfolio management |

**How it works:** The `Tab` enum lives in `src/dashboard/tabs.rs`. Each tab has a `label()` for display and a `shortcut()` hint (Ctrl+1..6, implemented in v2.0.7). The `Dashboard` struct gained an `active_tab: Tab` field (default: `Tab::Astrology`). The `view()` function renders a shared header (ticker buttons, search bar, recently viewed, refresh button), then a tab bar, then dispatches to tab-specific content via `match self.active_tab`.

**Tab bar UI:** Active tab is visually distinguished with `[brackets]` around the label. All tabs are always visible as buttons. Clicking a tab updates `active_tab` via `Message::TabSelected(Tab)`.

**Zero regression guarantee:** All 18 existing dashboard sections are accessible. Nothing was removed, only reorganized into tabs. The same data loads regardless of which tab is active.

**New file:** `src/dashboard/tabs.rs`
**Modified:** `src/dashboard/main.rs` (mod tabs), `src/dashboard/state.rs` (active_tab + TabSelected), `src/dashboard/update.rs` (handle TabSelected), `src/dashboard/view.rs` (tab bar + dispatch refactor)
**Tests:** 38 passing (no new tests needed for UI layout)

---

### v2.0.5 — Lagrange Score Rebalance *(completed 2026-04-22)*

**Theme:** Astrology becomes the lead component in the Lagrange Score. When astrology and financials agree, the system says so with confidence. When they disagree, it flags the divergence plainly. This is Principle #2 (Candor).

**Weight change:**
| Component | Old Weight | New Weight | Role |
|-----------|-----------|-----------|------|
| Astrology | 25% | **40%** | Lead signal |
| Financial | 35% | **25%** | Verification |
| Macro | 25% | **20%** | Context |
| Short Squeeze | 15% | **15%** | Unchanged |

**Why 40% for astrology?** The astrological signal is the product differentiator. Financial data verifies or challenges it. The old 25/35 split treated astrology as a secondary input. The new 40/25 split reflects the core philosophy: astrology leads, financials verify. When they agree, you have high confidence. When they disagree, the Concordance indicator tells you.

**Concordance indicator:** A new enum computed alongside the Lagrange Score that measures whether astrology and financials point in the same direction:

| Concordance | Astro Score | Financial Score | Meaning |
|-------------|------------|----------------|---------|
| Strong Confirm (++) | > 60 | > 60 | Both signals agree favorably. High confidence. |
| Mild Confirm (+) | > 60 | 40-60 | Astro favorable, financials neutral. Moderate confidence. |
| Divergence (!) | > 60 AND < 40 | Opposite direction | Signals disagree. Flag for review. |
| Mild Deny (-) | < 40 | 40-60 | Astro unfavorable, financials neutral. Moderate caution. |
| Strong Deny (--) | < 40 | < 40 | Both signals agree unfavorably. High conviction to avoid. |

**How it works in the pipeline:** After the Lagrange Score is computed (Phase 4), `compute_concordance(astro, fin)` compares the two sub-scores. The result is stored in the `concordance` column of `lagrange_history` alongside the score. The dashboard reads this column and will display it as a badge next to the Lagrange score (implemented in v2.0.6 tab UI).

**New migration:** `migrations/0023_concordance.sql` (adds `concordance TEXT` to `lagrange_history`)
**Modified:** `src/indicators.rs` (weights + Concordance enum + compute_concordance), `src/models.rs` (concordance field), `src/scraper/lagrange.rs` (store concordance), `src/dashboard/db.rs` (query concordance)
**Tests:** 38 passing (concordance is deterministic logic, tested implicitly through Lagrange score computation)

---

### v2.0.4 — Astro-First Scraper Pipeline *(completed 2026-04-22)*

**Theme:** Restructure the scraper pipeline so astrology runs FIRST, before any financial data fetching. This is the architectural expression of the core product philosophy: astrology is THE differentiator, everything else verifies the astrological signal.

**Why this matters:** The old pipeline fetched financial data for all tickers equally, then ran astrology last. This was backward. Astrology computation requires zero API calls (Swiss Ephemeris is compiled into the binary), so it's instant and free. Financial data fetching burns API budget (Alpha Vantage: 5/min, FMP: 120/day, Finnhub: 60/min). By running astrology first, we know which tickers the stars favor BEFORE spending any API budget, so we can prioritize the most astrologically interesting tickers for financial verification.

**The 4-phase pipeline:**

```
PHASE 1: ASTROLOGY (zero API calls, local computation)
  1.1 Seed natal charts (any new tickers with IPO dates)
  1.2 Compute daily planetary transits (Swiss Ephemeris, all 13 bodies)
  1.3 Compute astro scores + generate horoscope readings for all tickers
  1.4 Compute astro rankings: Top 5 Favorable + Bottom 5 Misaligned
      -> Log the rankings to console with box-drawing characters

PHASE 2: TARGETED FINANCIAL VERIFICATION (guided by astro rankings)
  2.1 Watchlist price data (Alpha Vantage)
  2.2 Sentiment (Alpha Vantage)
  2.3 News, earnings, ratings (Finnhub)
  2.4 Macroeconomic data (FRED)
  2.5 Short interest (FINRA)
  2.6 Options flow (Polygon.io)

PHASE 3: BULK DATA (universe + history + enrichment)
  3.1 Ticker universe seeding (Polygon)
  3.2 Ticker universe seeding (FMP)
  3.3 Bulk price history (Tiingo, up to 490/day)
  3.4 EDGAR insider trades + 8-K filings
  3.5 13F institutional holdings
  3.6-3.9 IPO date enrichment (AV, FMP, Wikidata, SEC EDGAR)

PHASE 4: COMPOSITE SCORING
  4.1 Lagrange Score computation (astro-informed)
```

**AstroRanking struct:** After Phase 1, the `compute_astro_ranking()` function queries the DB for today's scores, returns the top 5 and bottom 5 tickers with their scores and dominant themes, and logs them in a formatted box to the console. The `priority_tickers()` method returns a combined list for future use by Phase 2 functions (when they are enhanced to accept priority ordering).

**Console output during Phase 1:**
```
══════════════════════════════════════════════════════════
  ASTRO RANKINGS — 2026-04-22
══════════════════════════════════════════════════════════
  TOP 5 FAVORABLE:
    1. NVDA   87  Optimal Growth & Expansion
    2. AAPL   82  Growth & Expansion
    ...
  BOTTOM 5 MISALIGNED:
    1. META   22  Extreme Caution & Restructuring
    2. TSLA   31  Caution & Restructuring
    ...
  Total scored: 487
══════════════════════════════════════════════════════════
```

**Modified:** `src/scraper/main.rs` (major pipeline restructure), `src/scraper/astrology.rs` (AstroRanking + compute_astro_ranking)
**Tests:** 38 passing (no new tests needed for pipeline ordering, existing tests still pass)

---

### v2.0.3 — Horoscope Reading Engine *(completed 2026-04-22)*

**Theme:** Generate written "horoscope readings" for each ticker. Not just a number, but a narrative interpretation that explains what the stars say and what it means for the company. This is the killer feature that turns raw astrological data into actionable financial intelligence.

**How it works:** The engine is entirely template-based. No AI, no API calls, no network requests. Every planet-aspect-planet combination maps to a pre-written financial interpretation stored in Rust match arms. When you ask "what does Jupiter trine natal Venus mean for AAPL?", the engine looks up:

1. **Planet-aspect-planet template:** Jupiter (expansion) + trine (harmonious flow) + Venus (value/partnerships) = "Partnership expansion, revenue growth, favorable M&A conditions."
2. **Dignity enrichment:** If Jupiter is in Sagittarius (domicile), append "Jupiter is in domicile (strongest expression), amplifying this transit."
3. **Strength assessment:** Combine orb tightness (exact = "Very strong", 5+ degrees = "Mild") with applying/separating status.

The reading generation pipeline:

```
TransitScore (from v2.0.2)
  -> Pick top 5 aspects by |score_delta|
  -> Map each to TransitInterpretation (meaning + financial implication + strength)
  -> Classify dominant theme (Growth/Caution/Transformation/Innovation)
  -> Interpret moon phase in financial context (8 phases)
  -> Mercury Rx warning if active (3 combined scenarios)
  -> Compute timing window (applying = energy building vs separating = energy fading)
  -> Calculate confidence (strong aspect count + directional agreement + score extremity)
  -> Synthesize overall_outlook narrative (2-3 sentences combining all signals)
  -> Serialize to JSONB and store in horoscope_readings table
```

**Dominant theme classifier:** Categorizes the overall reading into one of 8 refined themes by weighting each planet's contribution to growth, caution, transformation, and innovation scores. Jupiter aspects feed "Growth & Expansion", Saturn feeds "Caution & Restructuring", Pluto feeds "Deep Transformation", Uranus feeds "Innovation & Disruption". The theme is further refined by the overall score (e.g., "Growth & Expansion" at score 80+ becomes "Optimal Growth & Expansion").

**Timing window logic:** Counts applying vs separating aspects and checks for tight-orb (< 2 degrees) favorable aspects. A score of 70+ with 2+ tight favorable applying aspects = "Strong favorable window active NOW." A score below 45 with many applying aspects = "Challenging energy intensifying. Defensive positioning recommended."

**Confidence scoring:** Three factors, weighted 0-100:
- Strong aspect count (|delta| > 5.0): up to 50 points
- Directional agreement (do most aspects point the same way?): up to 30 points
- Score extremity (distance from 50): up to 20 points

**Moon phase interpretations (8 phases, financial context):**
| Phase | Angle | Financial Meaning |
|-------|-------|-------------------|
| New Moon | 0-29 degrees | Initiation window. Plant seeds. Bold beginnings supported. |
| Waxing Crescent | 30-89 degrees | Momentum building. Commit resources. Market rewards forward movement. |
| First Quarter | 90-134 degrees | Decision point. Cut losers, double down on winners. |
| Waxing Gibbous | 135-179 degrees | Refine and perfect. Optimization phase. |
| Full Moon | 180-209 degrees | Peak volatility. Reversals common. Take profits. |
| Disseminating | 210-269 degrees | Harvest phase. Share results. Reduce momentum exposure. |
| Last Quarter | 270-314 degrees | Re-evaluate and release. Defensive positioning. |
| Balsamic | 315-359 degrees | Endings and clearing. Close losing positions. Prepare for renewal. |

**Mercury Rx interpretation:** When Mercury is retrograde, the engine produces a warning that combines the retrograde caution ("NOT a time to sign major contracts") with the current moon phase for nuanced timing. Full Moon + Mercury Rx = "volatility and confusion peak. Maximum caution advised."

**New file:** `src/astrology/interpretation.rs` (~600 lines)
**New migration:** `migrations/0022_horoscope_readings.sql`
**Modified:** `src/astrology/mod.rs` (export), `src/scraper/astrology.rs` (generate + store after scoring)
**Tests:** 8 new tests (total: 38 passing)

---

### v2.0.2 — Extended Aspect System *(completed 2026-04-22)*
**Theme:** Expand the aspect detection engine from 5 major aspects to 9 (adding 4 minor aspects), implement applying/separating distinction, and add planetary dignity/detriment scoring. This makes the astrological scoring engine production-grade.

**Why this matters:** The v0.5.0-v2.0.1 engine only detected 5 major aspects. This missed important astrological signals. A Quincunx (150 degrees) between transiting Saturn and natal Jupiter often signals "forced restructuring" in financial charts, but our engine was blind to it. The applying/separating distinction is even more impactful: two charts with the same aspects can have completely different readings depending on whether the energy is building or fading. And a transit planet's dignity in its current sign tells you how strong that planet's influence actually is.

**How applying/separating detection works:**

In astrology, an "applying" aspect is one where the transit planet is moving toward the exact angle with the natal planet. A "separating" aspect is one where the transit planet has passed the exact angle and is moving away. Think of it like a wave: applying = the wave is still building, separating = the wave has crested and is receding.

Our implementation uses `longitude_speed` (degrees/day) from Swiss Ephemeris. We project the transit planet's position 0.1 days forward and check whether the angular distance to the exact aspect angle is decreasing (applying) or increasing (separating). This is more accurate than the common "compare yesterday vs today" approach because it uses instantaneous velocity rather than a 24-hour average.

Score multipliers: applying aspects get 1.5x their base score (building energy), separating aspects get 0.7x (fading energy). An applying Jupiter trine scores 50% more than a separating one.

**How planetary dignity works:**

Each planet has signs where it's strongest (Domicile, Exaltation) and weakest (Detriment, Fall). This is one of the oldest systems in astrology, dating back to Hellenistic-era practitioners. The dignity table:

| Planet | Domicile | Exaltation | Detriment | Fall |
|--------|----------|------------|-----------|------|
| Sun | Leo | Aries | Aquarius | Libra |
| Moon | Cancer | Taurus | Capricorn | Scorpio |
| Mercury | Gemini, Virgo | Virgo | Sagittarius | Pisces |
| Venus | Taurus, Libra | Pisces | Aries, Scorpio | Virgo |
| Mars | Aries, Scorpio | Capricorn | Taurus, Libra | Cancer |
| Jupiter | Sagittarius | Cancer | Gemini | Capricorn |
| Saturn | Capricorn, Aquarius | Libra | Cancer | Aries |

A dignified planet (Domicile or Exaltation) gets a +20% score modifier. A debilitated planet (Detriment or Fall) gets a -20% modifier. Outer planets (Uranus, Neptune, Pluto) and nodes/Chiron have no traditional dignity, so they always score at baseline (Peregrine, 1.0x).

**New aspect types (4 minor, bringing total to 9):**

| Aspect | Angle | Orb | Nature | Financial meaning |
|--------|-------|-----|--------|-------------------|
| Semi-sextile | 30 degrees | 2 degrees | Mildly harmonious | Slight opportunity, incremental growth |
| Semi-square | 45 degrees | 2 degrees | Challenging | Internal friction, organizational tension |
| Sesquiquadrate | 135 degrees | 2 degrees | Challenging | External agitation, market pressure |
| Quincunx (Inconjunct) | 150 degrees | 3 degrees | Stressful adjustment | Forced pivots, uncomfortable restructuring |

Minor aspects score at 0.5x the base magnitude of major aspects. This reflects their subtler energy. A minor semi-square from Saturn carries about half the weight of a major square from Saturn.

**Scoring pipeline (4 layers, up from 2):**

```
score = base_magnitude                    [planet natures: 5-15 points]
      x direction                         [harmonious +1, challenging -1, conjunction depends]
      x orb_modifier                      [1.0 at exact, 0.25 at max orb]
      x minor_aspect_modifier             [1.0 for major, 0.5 for minor]
      x dignity_modifier                  [1.2 dignified, 0.8 debilitated, 1.0 peregrine]
      x applying_separating_modifier      [1.5 applying, 0.7 separating]
```

**Changes to `src/astrology/aspects.rs` (major expansion, ~510 lines total):**

- `AspectType` enum expanded: 5 variants to 9
- New methods: `is_major()`, `all()` returning all 9 types
- `find_aspect()` now checks all 9 types, returns tightest orb on overlap
- `aspect_nature()` updated for minor aspects (SemiSextile = harmonious, others = challenging)
- New function `is_applying()` using longitude_speed projection
- New enum `DignityState` (Domicile, Exaltation, Detriment, Fall, Peregrine) with `name()` and `symbol()`
- New function `planetary_dignity(planet, sign)` -- full lookup table for Sun through Saturn
- New function `dignity_modifier(state)` -- 1.2 / 0.8 / 1.0
- New function `score_aspect_full()` -- full scoring with dignity + minor aspect reduction
- `score_aspect()` retained as backward-compatible wrapper (delegates to `score_aspect_full` with None defaults)
- `ActiveAspect` struct expanded: added `applying: bool` and `dignity: DignityState` fields

**Changes to `src/astrology/natal.rs`:**

- `compute_transit_score()` now collects `longitude_speed` for all transit planets via Swiss Eph bridge
- Computes `is_applying()` for each aspect using instantaneous speed
- Computes `planetary_dignity()` for each transit planet in its current sign
- Applies `APPLYING_MULTIPLIER` (1.5) or `SEPARATING_MULTIPLIER` (0.7) to each aspect score
- `aspects_to_json()` now serializes `applying` and `dignity` fields
- `aspects_from_json()` parses them back (backward-compatible: defaults to applying=true, dignity=Peregrine for old data)

**Changes to `src/dashboard/astrology.rs`:**

- Transits table header expanded: added "A/S" column (Applying/Separating indicator)
- Each row now shows "A" or "S" for applying/separating
- Transit planet name shows dignity suffix: "+" for Domicile/Exalted, "-" for Detriment/Fall

**Test results (30 tests, all passing):**

New tests added:
- `test_all_9_aspects_detectable` -- every aspect detected at exact angle
- `test_applying_detection` -- applying/separating correctly identified from speed direction
- `test_dignity_venus_pisces_exalted` -- Venus in Pisces = Exaltation, 1.2x modifier
- `test_dignity_venus_virgo_fall` -- Venus in Virgo = Fall, 0.8x modifier
- `test_dignity_affects_score` -- exalted Venus scores higher than debilitated Venus
- `test_find_minor_aspect_quincunx` -- Quincunx detected at 150 degrees
- `test_find_minor_aspect_semisextile` -- Semi-sextile detected at 30 degrees
- `test_minor_aspect_scores_less_than_major` -- minor aspects score ~50% of major

**Files modified:** `src/astrology/aspects.rs` (major expansion), `src/astrology/natal.rs`, `src/dashboard/astrology.rs`

---

### v2.0.1 — Swiss Ephemeris Accuracy Upgrade *(completed 2026-04-22)*
**Theme:** Replace the Jean Meeus approximate ephemeris with the Swiss Ephemeris (sub-arcsecond accuracy), expand from 10 to 13 celestial bodies, and lay the foundation for production-grade financial astrology.

**Why this matters:** The Meeus engine we shipped in v0.5.0 was a pure-Rust implementation of simplified astronomical formulas from Jean Meeus' *Astronomical Algorithms*. It was good enough to get the astrology layer working, but testing against Swiss Ephemeris revealed a critical problem: **the Meeus heliocentric-to-geocentric conversion was producing errors of 20-150 degrees for most planets.** Only the Sun and Moon (which use dedicated formulas, not the generic conversion) were accurate. Every planet position displayed in the dashboard from Mercury through Pluto was significantly wrong. Swiss Ephemeris fixes all of them to sub-arcsecond precision.

**How Swiss Ephemeris works (technical explanation):**

Swiss Ephemeris is NOT a web API. It makes zero network calls. It is a C library (written by Astrodienst, Switzerland) that gets compiled directly into our Rust binary via FFI (Foreign Function Interface). When you run `cargo build`, the C compiler on your machine compiles the Swiss Ephemeris C source alongside our Rust code into a single executable. No downloads, no servers, no API keys at runtime.

The library implements VSOP87 planetary theory (a set of mathematical series with thousands of terms) to compute where any planet is at any given Julian Day. A single call to `swe::calc(jdn, Planet::Sun, flags)` executes in microseconds, returning longitude, latitude, distance, and speed. All pure math, computed locally.

There are three accuracy tiers, all running locally on the user's machine:

| Tier | What it is | Accuracy | Data source |
|------|-----------|----------|-------------|
| **Swiss Ephemeris** | Interpolation from embedded coefficient files | Sub-arcsecond (~0.001 degrees) | `.se1` files compiled into binary via `embedded-ephe` feature |
| **Moshier** | Analytical math formulas in C code | Sub-arcminute (~0.01 degrees) | Pure code, no external files needed |
| **Meeus (old)** | Simplified Rust formulas | 1-150 degree error for planets, < 1 degree for Sun/Moon | Pure code |

The `embedded-ephe` Cargo feature bakes the Swiss Ephemeris data files directly into the compiled binary. These files contain pre-computed coefficients (not raw positions, but the mathematical parameters that let the library interpolate positions for any date from 3000 BC to 3000 AD). Without this feature, you'd need to download `.se1` files separately.

The one important caveat: Swiss Ephemeris is a C library with global internal buffers, so it is not thread-safe. Two threads calling it simultaneously can corrupt results. We protect against this with a `SWE_LOCK` mutex in `natal.rs`. The scraper runs sequentially anyway, so this only affects parallel unit tests (which we run with `--test-threads=1`).

**New celestial bodies (13 total, up from 10):**

| Body | Symbol | Glyph | Swiss Eph constant | Financial astrology meaning |
|------|--------|-------|--------------------|-----------------------------|
| North Node (Rahu) | NN | ☊ | `TrueNode` (SE_TRUE_NODE = 11) | Destiny/growth direction. Where karmic expansion aligns with a company's trajectory. |
| South Node (Ketu) | SN | ☋ | Computed as NorthNode + 180 degrees | What's being released or left behind. Past patterns the company is outgrowing. |
| Chiron | Ch | ⚷ | `Chiron` (SE_CHIRON = 15) | The "wounded healer." Areas of vulnerability that, when addressed, become the company's greatest strength. |

Chiron note: Chiron is classified as an asteroid in Swiss Ephemeris and requires the asteroid ephemeris file (`seas_18.se1`), which is not included in the `embedded-ephe` feature. We fall back to the Moshier analytical ephemeris for Chiron (still sub-arcminute accuracy). If neither works, Chiron is gracefully skipped, giving 12 bodies instead of 13.

**New module -- `src/astrology/swisseph_bridge.rs` (~200 lines):**

Adapter layer between our `Planet`/`PlanetSnapshot` types and the `swiss-eph` crate's safe API. Key functions:

- `snapshot_all_precise(jdn)` -- compute all 13 bodies with sub-arcsecond accuracy. Falls back to Meeus for any planet where Swiss Eph fails.
- `longitude_speed(planet, jdn)` -- degrees/day speed for a planet. Positive = direct, negative = retrograde. Critical for v2.0.2's applying/separating aspect detection (no need to compute yesterday's positions separately).
- `compute_houses(jdn, lat, lon)` -- Whole Sign house cusps + Ascendant + Midheaven for a given location. Defaults to NYSE (40.7128 degrees N, 74.0060 degrees W) for US equity charts.
- `compute_houses_nyse(jdn)` -- convenience wrapper for NYSE location.
- `swe_julday(year, month, day, hour)` -- Swiss Eph's own Julian Day computation (verified to agree with our `date_to_jdn()` to within milliseconds).

**Planet enum expansion in `ephemeris.rs`:**

- Added `NorthNode`, `SouthNode`, `Chiron` variants to `Planet` enum
- `Planet::all()` returns all 13 bodies; `Planet::all_classical()` returns the original 10
- `Planet::needs_swiss_eph()` returns true for the 3 new bodies (prevents accidental use with Meeus)
- `planet_longitude()` and `is_retrograde()` panic with helpful messages if called for nodes/Chiron (forces use of Swiss Eph bridge)
- `norm360()` made public for use by the bridge module

**NatalChart + TransitScore wiring (`natal.rs`):**

- `NatalChart::compute()` now uses `snapshot_all_precise()` instead of `snapshot_all()`. Charts now contain 12-13 bodies at sub-arcsecond accuracy.
- `compute_transit_score()` uses Swiss Eph for all transit positions. Moon phase computed from Swiss Eph Moon/Sun longitudes (not Meeus formulas). Mercury retrograde detected from snapshot speed (not separate `is_retrograde()` call).
- Global `SWE_LOCK` mutex protects all Swiss Eph calls from concurrent access.
- Fallback: if Swiss Eph returns fewer than 10 bodies (catastrophic failure), falls back to Meeus `snapshot_all()`.

**Scraper updates (`src/scraper/astrology.rs`):**

- `compute_daily_transits()` uses `snapshot_all_precise()` and stores `longitude_speed` (degrees/day) for each planet. This enables the v2.0.2 applying/separating distinction without requiring a separate "yesterday's positions" query.
- `seed_natal_charts()` now also computes and stores natal angles (Ascendant + MC) in the new `natal_angles` table using NYSE location. Logs body count per chart.

**Dashboard updates (`src/dashboard/astrology.rs`):**

- `planet_abbrev()` and `planet_glyph()` extended for NorthNode (NN / ☊), SouthNode (SN / ☋), and Chiron (Ch / ⚷).

**Migration 0021 -- `natal_extended.sql`:**

- New table `natal_angles`: stores Ascendant and MC longitude per ticker (keyed on ticker, references `company_metadata`)
- New column `daily_transits.longitude_speed`: degrees/day for applying/separating detection
- No existing data modified. All new columns are nullable for backward compatibility.
- Optional: uncomment `DELETE FROM natal_positions` to force a full re-seed with Swiss Eph accuracy.

**New dependency:**
```toml
swiss-eph = { version = "0.2", features = ["embedded-ephe"] }
```
The `embedded-ephe` feature compiles the main planetary ephemeris data into the binary (adds ~2MB to binary size). No runtime file management needed.

**Test results (22 tests, all passing):**

- `test_swe_sun_position` -- Sun at J2000 (Jan 1, 2000) returns ~280 degrees (Capricorn). Correct.
- `test_swe_vs_meeus_sun_moon` -- Sun and Moon agree between Meeus and Swiss Eph to < 1 degree.
- `test_swe_nodes` -- North Node and South Node are exactly 180 degrees apart.
- `test_swe_chiron_optional` -- Chiron gracefully handled whether or not asteroid files are available.
- `test_swe_julday_agreement` -- Our JDN and Swiss Eph's JDN agree to < 0.001.
- `test_houses_nyse` -- Valid Ascendant, MC, and 12 house cusps at NYSE location.
- `test_snapshot_all_precise_count` -- Returns 12-13 bodies, all classical planets and nodes present.
- All pre-existing natal chart and aspect tests continue to pass.

Note: Tests must run single-threaded (`--test-threads=1`) due to Swiss Ephemeris C library thread-safety limitation. The scraper runs sequentially, so this only affects the test harness.

**Files created:** `src/astrology/swisseph_bridge.rs`, `migrations/0021_natal_extended.sql`
**Files modified:** `src/astrology/ephemeris.rs`, `src/astrology/mod.rs`, `src/astrology/natal.rs`, `src/scraper/astrology.rs`, `src/dashboard/astrology.rs`, `Cargo.toml`

---

### v1.0.0 — Enrichment Pipeline + Tiingo + Codebase Refactor *(completed 2026-04-20)*
**Theme:** Scale the scoring engine beyond the 10-ticker watchlist. Multi-source IPO date enrichment, bulk price history via Tiingo, DRY codebase cleanup, and critical data fixes that unlock Lagrange scores at scale.

**New scraper modules:**

- [x] `src/scraper/edgar_enrich.rs` — SEC EDGAR first-filing date enrichment
  - Fetches `company_tickers.json` (one call → full CIK lookup table)
  - For each ticker with null `ipo_date`: queries `CIK{padded}.json` for earliest 10-K / S-1 / 20-F / F-1
  - Handles paginated archive batches (`files[]` in submissions JSON)
  - **CIK deduplication cache**: `HashMap<u64, NaiveDate>` — share-class variants (ADAMG/ADAMH/ADAMI) reuse the date fetched for the primary CIK without additional API calls
  - Rate: 200ms between calls (≤5 req/sec, well under EDGAR's 10 req/sec limit)
  - Budget: 50 tickers per daily run, watchlist-first ordering

- [x] `src/scraper/wikidata_enrich.rs` — Wikidata SPARQL founding/inception dates
  - Single HTTP call fetches up to 10,000 companies with ticker + inception date
  - SPARQL query uses UNION of `wdt:P249` (direct property) and `p:P414 → pq:P249` (P414 qualifier, where 90%+ of ticker data lives) — filtered to NYSE / NASDAQ / AMEX exchange QIDs
  - POST form method (avoids URL length limits for complex SPARQL)
  - Runs once per day (guarded via `fetch_log` check)
  - First run result: 1,974 bindings → 1,064 inception dates filled

- [x] `src/scraper/fmp_enrich.rs` — Financial Modeling Prep integration
  - `seed_ticker_universe()`: calls `/v3/stock/list`, inserts all US common stocks + ETFs into `company_metadata` — gives full Robinhood / Bloomberg tradeable universe
  - `enrich_ipo_dates()`: calls `/v3/profile/{ticker}` for up to 240 tickers/day with null `ipo_date` — fills date and seeds natal chart
  - Budget tracking via `fetch_log` (250 req/day free tier; 1 for stock list + 240 for profiles)
  - Graceful 403 handling: stops enrichment loop immediately and explains paid-plan requirement

- [x] `src/scraper/company_enrich.rs` — Alpha Vantage OVERVIEW IPO date enrichment
  - Calls `function=OVERVIEW` for up to 10 tickers/day (watchlist-first)
  - Budget-aware: counts AV calls used today, leaves 1 as safety margin
  - Handles AV rate limit messages (`"Note"` / `"Information"` JSON keys)

- [x] `src/scraper/enrich_common.rs` — Shared enrichment utilities (NEW, ~44 lines)
  - `seed_one_natal_chart()` — single canonical implementation, called by all 4 enrichment modules
  - `watchlist_priority_sql()` — generates `CASE WHEN ticker IN (...) THEN 0 ELSE 1 END, ticker` ORDER BY clause
  - Eliminated ~83 lines of copy-paste duplication across `company_enrich`, `fmp_enrich`, `edgar_enrich`, `wikidata_enrich`

- [x] `src/scraper/tiingo.rs` — Tiingo bulk price history feed
  - `fetch_all_prices_tiingo()`: fetches up to 490 tickers/day (500 free tier − 10 margin)
  - Priority: watchlist first, then all tickers with natal chart but fewer than 26 price rows (Lagrange threshold)
  - Per-ticker: uses `MAX(date) + 1` as start date if rows exist, otherwise 5 years back
  - Handles 404 silently (ticker not in Tiingo), budgets via `fetch_log`
  - **This is the primary unlock for Lagrange scores** — each ticker needs 26+ rows for the financial component (35% weight)

**Data fixes:**

- [x] T. Rowe Price removed from `INSTITUTION_MAP` in `main.rs` — T. Rowe files 13F under subsidiary CIKs, not top-level CIK `0001113169`. Was causing a persistent "No 13F-HR found" error every run.
- [x] EDGAR CIK deduplication cache — AGNCL / AGNCO / AGNCP (preferred share classes) reuse AGNC's cached date instead of burning 3 additional daily API slots
- [x] Migration 0017: `ipo_date DATE` made nullable in `company_metadata` — required for the incremental enrichment pipeline where dates arrive from multiple sources over multiple days

**Enrichment pipeline order in `run_all_fetches`:**
```
1. Polygon ticker universe seed      (new listings, list_date available)
2. FMP ticker universe seed          (full Robinhood/Bloomberg universe)
3. AV price fetch                    (watchlist, 5 calls/min)
4. Tiingo bulk price history         (490 tickers/day, watchlist-priority)
5. EDGAR Form 4 + 8-K
6. 13F institutional holdings
7. Finnhub (news, earnings, ratings)
8. AV sentiment
9. FRED macro data
10. FINRA short interest
11. Polygon options flow
12. AV OVERVIEW IPO enrichment       (10/day, watchlist-first)
13. FMP profile IPO enrichment       (up to 240/day)
14. Wikidata SPARQL founding dates   (once/day, bulk)
15. SEC EDGAR first-filing dates     (50/day, watchlist-first)
16. Astrology: seed natal charts
17. Astrology: daily transits
18. Astrology: compute astro scores
19. Lagrange: compute all scores
```

**Scale after v1.0.0 first run:**
| Table | Approximate rows |
|-------|-----------------|
| `company_metadata` | ~10,000 tickers |
| `natal_positions` | ~100,000 (10 planets × tickers with ipo_date) |
| `price_data` | ~2,500 (10 watchlist × ~250 AV rows) |
| `astro_scores` | ~1,200+ (tickers with ipo_date) |
| `lagrange_history` | 7 tickers (price_data FK limit — resolved in v1.1.0) |

---

### v1.1.0-design — Design System + Visual Polish *(completed 2026-04-22)*
**Theme:** Centralized color token architecture, systematic type scale, theme-aware canvas widgets, Apple-grade empty states. Source-code design audit: 10 findings, all resolved.

**New module — `src/dashboard/theme.rs`:**
- [x] Central color token module (single source of truth for all UI colors)
- [x] Theme-aware functions: `canvas_bg()`, `fg()`, `fg_dim()`, `fg_muted()`, `label_color()`, `grid_line()`, `sign_color()`, `ring_dim()`
- [x] Type scale constants (1.2x minor third): `TEXT_XS=8`, `TEXT_SM=10`, `TEXT_BASE=12`, `TEXT_MD=14`, `TEXT_LG=17`, `TEXT_XL=20`, `TEXT_2XL=24`
- [x] Chart accents: `ACCENT_BLUE`, `SMA20_ORANGE`, `SMA50_YELLOW`, `BB_BLUE`, `SPARKLINE_BLUE`
- [x] Natal wheel: `NATAL_GOLD`, `TRANSIT_BLUE`, `RETROGRADE_RED`, aspect colors
- [x] Score zones: `ZONE_MISALIGNED` through `ZONE_OPTIMAL`
- [x] Gauge zones: `GAUGE_EXTREME_FEAR` through `GAUGE_EXTREME_GREED`
- [x] Sparkline zone bands: `SPARK_ZONE_MIS` through `SPARK_ZONE_OPT`

**Charts/canvas theme awareness (FINDING-001 through FINDING-004):**
- [x] `charts.rs` — PriceChart: replaced hardcoded dark background with `theme::canvas_bg()`, all grid/label/crosshair colors theme-aware
- [x] `charts.rs` — LagrangeSparkline: added background fill, zone bands/lines/text use theme tokens
- [x] `gauges.rs` — FearGreedGauge: replaced inline `is_dark` checks with theme functions
- [x] `astrology.rs` — NatalWheel: replaced inline theme checks with `theme::canvas_bg()`, `theme::ring_dim()`, `theme::sign_color()`

**Table & layout normalization (FINDING-005 through FINDING-008):**
- [x] `view.rs` — all `.spacing()` on table rows normalized to 4px
- [x] `view.rs` — volume column uses `format_shares()` with comma formatting
- [x] `view.rs` — all numeric `.size()` calls replaced with `theme::TEXT_*` constants
- [x] `view.rs` — section heading hierarchy: primary sections at `TEXT_LG` (17px), secondary at `TEXT_MD` (14px)

**Empty states & scrollable heights (FINDING-009 through FINDING-010):**
- [x] `view.rs` — 10 empty state sections restructured: heading + explanation + data source hint
- [x] `view.rs` — scrollable heights increased: Universe 200->240, Alerts 140->160, Earnings 110->130, Holdings 100->120
- [x] `view.rs` — removed direct `Color` import (all colors via `theme::` module)

**Commits:**
- `d2dcd58` — theme.rs color tokens + chart/sparkline theme awareness
- `b502273` — table spacing + volume formatting
- `717abbd` — type scale + section hierarchy
- `d827415` — warm empty states
- `19c322e` — scrollable heights

---

### v0.9.0 — Alert Threshold System *(completed 2026-04-18)*
**Theme:** Real-time OS toast notifications when a ticker enters Optimal or Misaligned zone; full alert history panel

**New migration 0016:**
- [x] `lagrange_alerts` table: `id SERIAL, ticker, alert_date, score, label, prev_label, alert_type, is_read, created_at`
- [x] `UNIQUE (ticker, alert_date, alert_type)` — crossing is recorded once per day per direction
- [x] `alert_type` values: `entered_optimal`, `entered_misaligned`

**Scraper changes — lagrange.rs:**
- [x] `check_alert_crossing()` runs after each daily Lagrange upsert
- [x] Compares today's label to most-recent prior row (`ORDER BY score_date DESC LIMIT 1`) — weekend-safe
- [x] Only fires on threshold entry (not exit); ignores sideways stay in same zone
- [x] `ON CONFLICT DO NOTHING` — idempotent, safe on scraper re-runs

**New model:**
- [x] `LagrangeAlert { id, ticker, alert_date, score, label, prev_label, alert_type, is_read }` in `src/models.rs`

**Dashboard — db.rs:**
- [x] `fetch_alerts()` — SELECT last 50 alerts ordered by date DESC
- [x] `mark_alert_read()` — fire-and-forget UPDATE, logs to stderr on failure

**Dashboard — state.rs:**
- [x] New fields: `alerts: Vec<LagrangeAlert>`, `unread_alert_count: usize`, `notifications_fired: bool`
- [x] New messages: `AlertsLoaded`, `MarkAlertRead(i32)`, `NotifyAlerts`

**Dashboard — update.rs:**
- [x] `fetch_alerts` added to startup batch (`TickersLoaded`) and 30-second `Tick` handler
- [x] `AlertsLoaded`: stores alerts, counts unread, fires toast once per session via `notifications_fired` spam guard
- [x] `MarkAlertRead`: optimistic in-memory flip + fire-and-forget DB write
- [x] `fire_toast()` async fn using `notify-rust` — summary + up to 3 tickers in body, "+N more" overflow

**Dashboard — view.rs:**
- [x] Alerts panel below Watchlist Ranking: date, ticker, score, zone (color-coded), was, Mark Read button
- [x] Watchlist zone score text now color-coded: Misaligned=red, Unfavorable=orange, Neutral=gray, Favorable=blue, Optimal=green
- [x] `unread_alert_count` shown in alerts panel header when > 0

---

### v0.8.0 — Universal Birth Chart Database + Dynamic Ticker Search *(completed 2026-04-18)*
**Theme:** Scale the astrology engine from 10 hardcoded tickers to the full US equity market (~10k stocks)

**New scraper module — ticker_seed.rs:**
- [x] `src/scraper/ticker_seed.rs` — paginate Polygon.io `/v3/reference/tickers?market=stocks&active=true&locale=us`
- [x] Upsert all active US common stocks into `company_metadata` using `list_date` as `ipo_date`
- [x] Exchange MIC → human-readable name mapping: `XNAS` → NASDAQ, `XNYS` → NYSE, `ARCX` → NYSE Arca, etc.
- [x] Dedup guard: Polygon may return same ticker on multiple exchanges — take first occurrence
- [x] After bulk seed, run natal chart computation for every new `company_metadata` row → `natal_positions`
- [x] `ON CONFLICT (ticker) DO NOTHING` on both tables — fully idempotent, safe to re-run
- [x] Runs once at startup if `natal_positions` count < 100 (cold start), then daily incremental

**Migration 0014:**
- [x] Add `data_source TEXT NOT NULL DEFAULT 'manual'` to `company_metadata`
- [x] Add `seeded_at TIMESTAMPTZ` to `company_metadata`

**Dashboard — dynamic ticker search:**
- [x] Replace hardcoded 10-button ticker row with a text input + search button
- [x] On submit: look up ticker in DB, display full analysis if data exists
- [x] Graceful "No data yet — run scraper for this ticker" placeholder state
- [x] Pinned watchlist row preserved below search bar (user's 10 favorites, still one-click)
- [x] Recently viewed ring (last 10 tickers, stored in `recently_viewed` local DB table)

**Migration 0015:**
- [x] `recently_viewed` table: `ticker TEXT, viewed_at TIMESTAMPTZ`

---

### v0.7.0 — Lagrange History, Portfolio, CPI YoY%, Color-Coding *(completed 2026-04-16)*
**Theme:** Daily Lagrange history accumulation, portfolio positions, display polish

- [x] `src/indicators.rs` moved to lib crate — shared between scraper and dashboard binaries
- [x] `compute_lagrange_score` now returns `(f32, String, LagrangeComponents)` with component breakdown
- [x] `LagrangeComponents { fin_score, astro_score, macro_score, short_score }` stored for debugging
- [x] `src/scraper/lagrange.rs` — daily Lagrange Score computation for all 10 tickers
- [x] Migration 0013: `lagrange_history` + `portfolio_positions` tables
- [x] `LagrangeSparkline` canvas widget: 90-day score history strip below price chart
- [x] Portfolio panel: reads `portfolio_positions` table
- [x] Macro strip: CPI raw index replaced with YoY% via SQL CTE window calculation
- [x] Short interest symbol guard fix (was summing all records regardless of ticker)

---

### v0.6.0 — Expanded Data Sources + Actionable Intelligence *(completed 2026-04-15)*
**Theme:** FRED macro data, FINRA short interest, Lagrange Score, signal synthesis

- [x] Split 1,495-line `src/scraper/main.rs` monolith into 9 focused modules
- [x] FRED macroeconomic data (10 indicators: Fed Funds, CPI, Unemployment, yields, VIX, WTI, M2)
- [x] FINRA Developer API: per-ticker short sale volume
- [x] Polygon.io: options flow put/call ratio (free-tier probe pattern)
- [x] Lagrange Score: Financial(35%) + Astrology(25%) + Macro(25%) + Short Squeeze(15%)
- [x] Signal Intelligence panel: plain-English bullet synthesis per ticker
- [x] Watchlist Ranking panel: all 10 tickers sorted by quick composite score

---

### v0.5.0 — Astrology Layer *(completed 2026-04-14)*
**Theme:** Company birth charts, planetary transit scoring, astrological Fear & Greed

- [x] Pure-Rust ephemeris (Jean Meeus formulas) — Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto
- [x] Natal chart seeder + daily transit computation + aspect scoring
- [x] Migrations 0008–0011: `company_metadata`, `natal_positions`, `daily_transits`, `astro_scores`
- [x] Dashboard: natal chart wheel canvas (inner natal ring, outer transit ring, aspect lines)
- [x] Dashboard: active transits table, moon phase display, Mercury Rx flag

---

### v0.4.0 — Module Refactor *(completed 2026-04-14)*
**Theme:** Split 1,435-line monolith into maintainable modules (10 dashboard files)

---

### v0.3.0 — UI / UX Pass *(completed 2026-04-13)*
**Theme:** Usability, layout polish, copy/open buttons, Fear & Greed gauges (3 → 5 total by v0.6.0)

---

### v0.2.0 — Data Enrichment *(completed 2026-04-12)*
**Theme:** News, earnings, analyst ratings, sentiment pipeline from Finnhub + Alpha Vantage

---

### v0.1.0 — Foundation *(completed 2026-04-10)*
**Theme:** Core scraper + dashboard skeleton — Alpha Vantage prices, EDGAR feeds, Iced UI

---

## Architecture

### Binary layout

```
pursuit_week4_automation/
├── src/
│   ├── lib.rs                        shared types (models, astrology, indicators)
│   ├── models.rs                     SQLx FromRow structs
│   ├── indicators.rs                 SMA/EMA/RSI/MACD/BB/SMA200 + Lagrange Score     ← MOD v2.4.3
│   ├── astrology/                    planetary calculation engine
│   │   ├── mod.rs
│   │   ├── ephemeris.rs              Planet enum (13 bodies), Meeus fallback, JDN conversion
│   │   ├── aspects.rs                aspect detection + scoring (9 types, dignity, applying/separating)
│   │   ├── interpretation.rs         horoscope reading engine + theme-score reconciliation  ← MOD v2.0.8
│   │   ├── natal.rs                  NatalChart + TransitScore (sigmoid scoring)  ← MOD v2.0.8
│   │   └── swisseph_bridge.rs        Swiss Ephemeris adapter: sub-arcsecond positions, houses  ← NEW v2.0.1
│   ├── scraper/
│   │   ├── main.rs                   entry point + WATCHLIST + scheduler
│   │   ├── prices.rs                 Alpha Vantage daily OHLCV (watchlist)
│   │   ├── tiingo.rs                 Tiingo bulk price history (all scored tickers)  ← NEW v1.0
│   │   ├── edgar.rs                  SEC EDGAR Form 4 + 8-K
│   │   ├── edgar_enrich.rs           SEC first-filing date enrichment                ← NEW v1.0
│   │   ├── holdings.rs               SEC EDGAR 13F institutional
│   │   ├── finnhub.rs                news, earnings, analyst ratings
│   │   ├── sentiment.rs              Alpha Vantage NEWS_SENTIMENT
│   │   ├── astrology.rs              natal seeding + transit scoring
│   │   ├── macro_data.rs             FRED macroeconomic series
│   │   ├── short_interest.rs         FINRA Developer API short volume
│   │   ├── options.rs                Polygon.io put/call ratio
│   │   ├── lagrange.rs               daily Lagrange Score computation
│   │   ├── ticker_seed.rs            Polygon ticker universe seed
│   │   ├── fmp_enrich.rs             FMP ticker universe + IPO date enrichment       ← NEW v1.0
│   │   ├── company_enrich.rs         Alpha Vantage OVERVIEW IPO enrichment
│   │   ├── wikidata_enrich.rs        Wikidata SPARQL founding dates                  ← NEW v1.0
│   │   ├── fundamentals.rs            FMP key-metrics + ratios scraper              ← NEW v2.2.1
│   │   └── enrich_common.rs          shared: seed_one_natal_chart + watchlist SQL    ← NEW v1.0
│   └── dashboard/
│       ├── main.rs                   entry point + mod declarations
│       ├── state.rs                  Dashboard struct + Message enum + all features   ← MOD v3.0
│       ├── indicators.rs             local indicator helpers
│       ├── signals.rs                plain-English signal bullet generator
│       ├── helpers.rs                formatting utilities
│       ├── agents.rs                 AI agent personas (Buffett/Graham/Lynch/Munger) ← NEW v2.4.1
│       ├── patterns.rs               technical pattern recognition (6 patterns)      ← NEW v2.4.3
│       ├── backtest.rs                astro-driven backtesting engine                 ← NEW v2.8.1
│       ├── strategy.rs               composable condition chains + strategy backtest  ← NEW v3.0.2
│       ├── calendar.rs               astro calendar canvas widget (monthly heat map)  ← NEW v3.0.5
│       ├── heatmap.rs                sector heat map canvas widget                   ← NEW v2.6.2
│       ├── db.rs                     async DB + transactions + settings + calendar  ← MOD v3.0
│       ├── dcf.rs                    DCF intrinsic value calculator                ← NEW v2.2.2
│       ├── gauges.rs                 FearGreedGauge canvas widget
│       ├── charts.rs                 PriceChart canvas (volume bars, astro markers)  ← MOD v3.0.1
│       ├── tabs.rs                   Tab enum (7 tabs, Astrology first)              ← MOD v3.0.4
│       ├── theme.rs                  color tokens + circadian phase + ThemeMode     ← MOD v2.2.3
│       ├── update.rs                 update() + strategy + calendar + settings + tx   ← MOD v3.0
│       ├── view.rs                   view() strategy + calendar + settings + tx log  ← MOD v3.0
│       └── astrology.rs              natal wheel canvas + transits table
├── migrations/
│   ├── 0001_initial_schema.sql
│   ├── 0002_seed_watchlist.sql
│   ├── 0003_add_items_to_filings.sql
│   ├── 0004_news_articles.sql
│   ├── 0005_earnings_dates.sql
│   ├── 0006_analyst_ratings.sql
│   ├── 0007_sentiment_scores.sql
│   ├── 0008_company_metadata.sql
│   ├── 0009_natal_positions.sql
│   ├── 0010_daily_transits.sql
│   ├── 0011_astro_scores.sql
│   ├── 0012_macro_indicators.sql
│   ├── 0013_lagrange_and_portfolio.sql
│   ├── 0014_company_metadata_source.sql
│   ├── 0015_recently_viewed.sql
│   ├── 0016_lagrange_alerts.sql
│   ├── 0017_nullable_ipo_date.sql
│   ├── 0018_drop_price_data_fk.sql
│   ├── 0019_scoring_active.sql
│   ├── 0020_sector_industry.sql
│   ├── 0021_natal_extended.sql           natal_angles table + longitude_speed column  ← NEW v2.0.1
│   ├── 0022_horoscope_readings.sql      horoscope readings JSONB storage             ← NEW v2.0.3
│   ├── 0023_concordance.sql             concordance column on lagrange_history        ← NEW v2.0.5
│   ├── 0024_fundamentals.sql           fundamental_metrics table (22 columns)        ← NEW v2.2.1
│   ├── 0025_named_watchlists.sql      watchlists + watchlist_members tables          ← NEW v2.6.3
│   ├── 0026_transactions.sql         buy/sell transaction log with CHECK constraints  ← NEW v3.0.3
│   └── 0027_settings.sql             key-value settings store with seeded defaults    ← NEW v3.0.4
├── Cargo.toml
├── .env                              secrets (never committed)
├── .gitignore
├── CLAUDE.md
└── DESIGN.md                         this file
```

### Data flow

```
Alpha Vantage API   ──┐
SEC EDGAR API       ──┤
Finnhub API         ──┤
FRED API            ──┤
FINRA API           ──┤  scraper binary (startup + cron 6AM UTC)  ──►  PostgreSQL
Polygon.io API      ──┤
FMP API             ──┤
Tiingo API          ──┤
Wikidata SPARQL     ──┤
alternative.me      ──┘

PostgreSQL  ──►  dashboard binary (SQLx async)  ──►  Iced 0.13 UI
```

### Lagrange Score formula (v2.0.5)

```
Lagrange Score = Astrology(40%) + Financial(25%) + Macro(20%) + Short Squeeze(15%)

Astrology (lead signal, 40%):
  Today's astro_score from Swiss Ephemeris transit scoring (0-100)

Financial (verification, 25%):
  RSI(14) normalized 0-100                   x 0.30
  Price vs SMA50 momentum (+/-10% -> 0-100)  x 0.30
  MACD histogram (+/-0.2% of price -> 0-100) x 0.20
  AV sentiment (-1..+1 -> 0-100)             x 0.20

Macro (context, 20%):
  VIX score: (90 - (vix - 10) x 1.4) clamped 0-100   x 0.60
  Yield spread T10Y2Y: ((spread+1)x20+30) clamped     x 0.40

Short Squeeze (15%):
  base: pct>30% -> 75, pct>20% -> 65, pct>10% -> 50, else 40
  bonus: +15 if price rising AND short% > 15%

Concordance = compare(astro_score, fin_score):
  Strong Confirm (++) = astro > 60 AND fin > 60
  Mild Confirm (+)    = astro > 60 AND fin 40-60
  Divergence (!)      = astro and fin point opposite directions
  Mild Deny (-)       = astro < 40 AND fin 40-60
  Strong Deny (--)    = astro < 40 AND fin < 40

Labels: Misaligned (0-24) / Unfavorable (25-44) / Neutral (45-55) /
        Favorable (56-75) / Optimal (76-100)
```

---

## v1.1.0 Implementation Plan — Active Scoring Universe + Search Autocomplete

**Target:** 2026-04-21 or later (pending user review)

### Goal

Unlock Lagrange scoring for a meaningful universe of tickers beyond the 10-ticker watchlist — without bloating the UI or breaking any existing behavior. Simultaneously upgrade the search bar from "type and submit" to live autocomplete, so discovery of the expanded universe feels instant and natural.

This is the combination of **Option B** (configurable active scoring universe, ~50–100 tickers) and **Option D** (search autocomplete from `company_metadata`).

### The core architectural problem to solve

`price_data.ticker` currently has a **foreign key to `tickers(ticker)`**. The `tickers` table has only 10 rows (the pinned watchlist). This means:
- Tiingo cannot insert price data for any non-watchlist ticker — Postgres rejects with FK violation
- Lagrange currently loops over `WATCHLIST` const — hardcoded to 10 tickers even if data existed

The fix is clean: **drop the FK constraint**. The `tickers` table becomes the UI concept ("pinned watchlist buttons"), not a data integrity gate. `company_metadata` is the canonical universe of valid tickers and serves as the implicit source of truth for scraper inserts.

### Apple HIG Principles Applied

**Clarity — show the user what they have, not what they might have.**
- The watchlist buttons at the top are the user's "Dock" — fast-access to 10 curated tickers. They never move.
- The ranking panel shows the scored universe with clear visual hierarchy: rank, ticker, score zone (color + label), supporting signals.
- Search suggestions show company name alongside ticker symbol — "AAPL — Apple Inc." — never just a bare symbol.

**Progressive Disclosure — reveal complexity only when requested.**
- Typing 1 character shows up to 8 autocomplete suggestions. Not the full 10,000.
- The ranked universe panel shows the top 20 by default. A "Show more" row expands to the full set.
- Score details (component breakdown) live in the per-ticker drill-down, not the ranking row.

**Feedback — the UI should feel alive.**
- Autocomplete results appear within one keystroke delay (async DB query is fast: prefix match on indexed TEXT column).
- While loading suggestions, show a subtle spinner or dim the input.
- When a ticker search lands on a ticker with no price data, show the birth chart and explain what's missing.

**Consistency — reuse existing patterns.**
- Suggestion dropdown uses the same row style as the "Recently viewed" section.
- Ranking rows use the same `FillPortion` column widths already established in the watchlist panel.
- Score zone colors are unchanged: red / orange / gray / blue / green.

**Accessibility — color is never the only signal.**
- Every color-coded zone also carries a text label (Mis / Unf / Neu / Fav / Opt).
- Rank numbers provide a secondary ordering cue independent of color.

---

### Schema Changes (v1.1.0)

**Migration 0018 — Drop price_data FK, add performance index**
```sql
-- Remove the constraint that prevented Tiingo from inserting non-watchlist rows
ALTER TABLE price_data DROP CONSTRAINT IF EXISTS price_data_ticker_fkey;

-- Composite index for all the per-ticker price queries (Tiingo, Lagrange, dashboard)
CREATE INDEX IF NOT EXISTS idx_price_data_ticker_date ON price_data (ticker, date DESC);
```

**Migration 0019 — Add scoring_active flag to tickers table**
```sql
-- Separates "pinned watchlist button" from "active scoring universe"
-- watchlist (active=true, scoring_active defaults to true for existing rows)
-- additional scoring tickers: active=false, scoring_active=true
ALTER TABLE tickers ADD COLUMN IF NOT EXISTS scoring_active BOOLEAN NOT NULL DEFAULT true;

-- Future: INSERT INTO tickers (ticker, name, scoring_active) VALUES ('NVDA', ..., true)
-- to add tickers to the scoring universe without making them pinned watchlist buttons.
-- For now, the 10 existing rows all have scoring_active = true automatically.
```

> **Note:** v1.1.0 starts with the 10 existing watchlist tickers as the scoring universe. The migration establishes the concept cleanly. v1.2.0 will add a UI panel for managing the active set.

---

### Scraper Changes (v1.1.0)

**tiingo.rs — unblock non-watchlist inserts**
After migration 0018 drops the FK, Tiingo can insert for any `company_metadata` ticker. No code change needed — the query already targets `company_metadata WHERE ipo_date IS NOT NULL`.

**lagrange.rs — expand scoring universe**
Replace the hardcoded `WATCHLIST` loop with a DB-driven query:
```rust
// Was:
for ticker in WATCHLIST { ... }

// Becomes:
let scoreable: Vec<String> = sqlx::query_scalar(
    "SELECT DISTINCT pd.ticker
     FROM price_data pd
     JOIN tickers t ON t.ticker = pd.ticker
     WHERE t.scoring_active = true
     GROUP BY pd.ticker
     HAVING COUNT(*) >= 26
     ORDER BY pd.ticker"
).fetch_all(pool.as_ref()).await.unwrap_or_default();

for ticker in &scoreable { ... }
```

This means any ticker added to `tickers` with `scoring_active = true` that accumulates 26+ days of Tiingo price data automatically enters the Lagrange scoring loop the next morning.

---

### Dashboard Changes (v1.1.0)

**Search autocomplete flow**

New state field:
```rust
autocomplete_suggestions: Vec<(String, String)>,  // (ticker, company_name)
```

New DB function in `db.rs`:
```rust
pub async fn search_tickers(pool: Arc<PgPool>, prefix: String) -> Result<Vec<(String, String)>, String> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT ticker, COALESCE(company_name, ticker) FROM company_metadata
         WHERE ticker ILIKE $1 OR company_name ILIKE $2
         ORDER BY
           CASE WHEN ticker ILIKE $1 THEN 0 ELSE 1 END,
           ticker
         LIMIT 8"
    )
    .bind(format!("{}%", prefix.to_uppercase()))
    .bind(format!("%{}%", prefix))
    .fetch_all(pool.as_ref()).await.map_err(|e| e.to_string())
}
```

New messages:
```rust
AutocompleteResults(Vec<(String, String)>),
AutocompleteSelected(String),
```

**view.rs — search bar with suggestion dropdown**

Apple HIG pattern: the suggestions appear as a column directly below the search input, showing ticker + company name. Maximum 8 rows. Tapping any row loads that ticker exactly as if the user had submitted the search.

```
[ AAPL ] [ MSFT ] [ GOOGL ] ...  ← pinned watchlist (unchanged)
Search: [ APPL________________ ] [Go]
┌─────────────────────────────┐
│ AAPL  — Apple Inc.          │  ← suggestion row (click to select)
│ AAPB  — GraniteShares 2x... │
│ AAPU  — Direxion Daily...   │
└─────────────────────────────┘
Recently viewed: NVDA · JPM · V
```

**view.rs — ranked universe panel upgrade**

The existing Watchlist Ranking panel shows the active scoring universe (not just the 10 watchlist buttons). Add a scrollable container with fixed height and a "Scored universe" header that shows total count:

```
Scored Universe — 10 tickers  [ ▼ Sort: Score ]
────────────────────────────────────────────────
#   Ticker  Score         Astro    Sentiment  Short%
1   AAPL    82 Opt ■     73 Greed  Bull (0.3)  0.7%
2   MSFT    71 Fav ■     68 Greed  Neut (0.1)  0.8%
...
```

Scrollable container fixed at 200px height. Sort toggle (by Score desc, by Ticker asc) is a button row at the header.

---

### File Changes (v1.1.0)

| File | Change |
|------|--------|
| `migrations/0018_drop_price_data_fk.sql` | New — drops FK, adds composite index |
| `migrations/0019_scoring_active.sql` | New — adds `scoring_active` column to `tickers` |
| `src/scraper/lagrange.rs` | Replace `WATCHLIST` loop with DB query on `scoring_active = true` |
| `src/dashboard/state.rs` | Add `autocomplete_suggestions: Vec<(String, String)>` |
| `src/dashboard/db.rs` | Add `search_tickers()` function |
| `src/dashboard/update.rs` | Handle `TickerSearchInput` → fire autocomplete query; handle `AutocompleteResults`, `AutocompleteSelected` |
| `src/dashboard/view.rs` | Add suggestion dropdown below search bar; upgrade ranking panel header |

**Estimated new code:** ~180 lines across 6 files.

---

### Scale Estimates (v1.1.0)

| Concern | Assessment |
|---------|-----------|
| Price table in UI | Already `LIMIT 100` in `db.rs` — safe at any scale |
| Ranking panel | Scrollable `Length::Fixed(200.0)` — handles 100+ rows fine |
| Autocomplete query | Indexed TEXT prefix match — sub-10ms even at 10k rows |
| Market breadth query | `price_data` CTEs will be heavier with 490+ tickers; `idx_price_data_ticker_date` index from migration 0018 handles it |
| Lagrange loop | DB-driven, not hardcoded — if 0 new tickers qualify, loop body never runs |
| Alert flood risk | `check_alert_crossing` returns early on first-ever score — no flood on Tiingo first run |

---

## v1.2.0 Preview — Universe Explorer (Option C Foundation)

**Target:** Future sprint (after v1.1.0 is validated)

### Goal

A dedicated "Universe Explorer" panel that gives the user a sortable, filterable view of Lagrange scores across the full population of scored tickers — potentially 500–1,000+ tickers after several weeks of Tiingo accumulation. This is the full realization of Option C.

### What v1.1.0 prepares for v1.2.0

- FK dropped: Tiingo can score any ticker
- `scoring_active` column: the concept of "active universe" exists in the DB
- Composite index on `price_data(ticker, date)`: bulk queries are fast
- Lagrange loop is DB-driven: adding tickers automatically enters them into scoring

### New features in v1.2.0

**Universe Explorer panel** (new tab or expandable section):
- Sortable columns: Ticker, Lagrange Score, Zone, Financial, Astro, Macro, Short%
- Filter controls: Zone filter (show only Optimal / Favorable), Sector filter (requires sector data in `company_metadata`)
- Pagination: 50 rows per page, page controls
- "Add to Scoring Universe" button on each row (sets `scoring_active = true`)
- Export to CSV button (for integration with other tools)

**DB query driving the Explorer:**
```sql
SELECT lh.ticker, cm.company_name, lh.score, lh.label,
       lh.fin_score, lh.astro_score, lh.macro_score, lh.short_score
FROM lagrange_history lh
JOIN company_metadata cm ON cm.ticker = lh.ticker
WHERE lh.score_date = (SELECT MAX(score_date) FROM lagrange_history WHERE ticker = lh.ticker)
ORDER BY lh.score DESC
LIMIT 50 OFFSET $1
```

**Sector data source:** FMP `/v3/profile/{ticker}` returns `sector` and `industry` fields — add to `company_metadata` during enrichment pass.

**Managing the scoring universe (v1.2.0 UI):**
```
Scoring Universe Settings
─────────────────────────
Currently scoring: 47 tickers  [ + Add tickers ]  [ Import from CSV ]
Tiingo budget: 490/day  ·  Today's usage: 312/490

[ AAPL × ] [ MSFT × ] [ NVDA × ] ... → click × to remove from scoring
```

### Apple HIG principles for v1.2.0

**Deference:** The Explorer is a secondary surface — it defers to the per-ticker drill-down. Row tap → load that ticker in the main view.

**Depth:** Three levels of information density:
1. Universe Explorer row: ticker, score, zone (summary)
2. Ranked list in main view: all signal components (intermediate)
3. Per-ticker drill-down: full chart, astrology, news, filings (full depth)

**Information hierarchy:** Score zone color is the primary signal. Numeric score is secondary. Company name is tertiary (truncated to fit). Users scan by color band, then read numbers.

---

## Plan of Action — Versioned Checklist

### v1.0.0 — COMPLETE *(2026-04-20)*

- [x] `edgar_enrich.rs` — SEC first-filing date enrichment
- [x] `wikidata_enrich.rs` — Wikidata SPARQL founding dates (UNION query fix, POST form method)
- [x] `fmp_enrich.rs` — FMP ticker universe seed + IPO date enrichment
- [x] `company_enrich.rs` — AV OVERVIEW IPO date enrichment
- [x] `enrich_common.rs` — DRY extraction: `seed_one_natal_chart` + `watchlist_priority_sql`
- [x] `tiingo.rs` — Tiingo bulk price history (490/day, watchlist-priority, 5-year history)
- [x] `main.rs` — wire all new modules + `TIINGO_API_KEY` env var
- [x] Migration 0017 — `ipo_date` nullable in `company_metadata`
- [x] T. Rowe Price removed from `INSTITUTION_MAP` (subsidiary CIK issue)
- [x] EDGAR CIK deduplication cache (share-class variants share one API call)
- [x] `Cargo.toml` — added `"form"` feature to reqwest for Wikidata SPARQL POST
- [x] `.env` — `TIINGO_API_KEY` added

---

### v1.1.0 — COMPLETE *(2026-04-20)*

**Database**
- [x] Write `migrations/0018_drop_price_data_fk.sql` — drop FK constraint, add composite index
- [x] Write `migrations/0019_scoring_active.sql` — add `scoring_active BOOLEAN` to `tickers`
- [x] Migrations applied (confirmed "Migrations OK" on 2026-04-20 scraper run)
- [x] Tiingo insert verified for non-watchlist tickers (scraper run 2026-04-20, no FK violations)

**Scraper**
- [x] `lagrange.rs` — replace `WATCHLIST` const loop with DB-driven `scoring_active = true` query
- [x] `tiingo.rs` — lower `MAX_PER_RUN` 490 → 45 (burst limit fix), sleep 200ms → 1000ms, graceful 429 stop
- [x] Verify: run scraper, confirm Lagrange scores for 10 tickers — no regression (confirmed 2026-04-20)

**Dashboard — autocomplete**
- [x] `db.rs` — add `search_tickers(prefix: String)` function (ILIKE prefix + company name fuzzy)
- [x] `state.rs` — add `autocomplete_suggestions: Vec<(String, String)>`
- [x] `state.rs` — add `Message::AutocompleteResults`, `Message::AutocompleteSelected`, `Message::ToggleWatchlistSort`
- [x] `update.rs` — `TickerSearchInput` fires `search_tickers` on each keystroke, clears on empty
- [x] `update.rs` — `AutocompleteSelected` clears suggestions and dispatches `TickerSelected`
- [x] `view.rs` — suggestion dropdown column below search bar (max 8 rows, "TICKER — Company Name")

**Dashboard — ranking panel**
- [x] `view.rs` — "Scored Universe — N tickers" header
- [x] `view.rs` — `scrollable(...).height(Length::Fixed(200.0))` on ranking rows
- [x] `view.rs` — Sort toggle button (Score desc / Ticker asc) in panel header
- [x] `update.rs` — `ToggleWatchlistSort` flips bool, re-sorts watchlist in place
- [x] `update.rs` — `RefreshNow` expanded to reload all panels (lagrange history, market FG, watchlist, macro, alerts)

**Dashboard — no-data states**
- [x] Chart: placeholder text when `self.rows.is_empty()` instead of blank canvas
- [x] Signal Intelligence: "No price data yet" guard on outer `if self.rows.is_empty()`
- [x] Indicator row: "Indicators: —" when rows empty
- [x] Astrology: conditional render — informational message if `natal_positions.is_empty()`

**Regression testing checklist**
- [x] All 10 existing watchlist buttons still work
- [x] Recently viewed still populates
- [x] Lagrange sparkline still renders for watchlist tickers
- [x] Alert panel still shows / marks read
- [x] Price chart still renders `LIMIT 100` rows
- [x] No FK violations in scraper output

---

### v1.2.0 — PREP ITEMS *(completed during v1.1.0, 2026-04-20)*

- [x] `company_metadata` — add `sector TEXT`, `industry TEXT` columns (migration 0020)
- [x] `fmp_enrich.rs` — capture `sector` / `industry` from `/v3/profile` response during enrichment; `FmpProfile` struct expanded; `fetch_profile_ipo_date` returns `(Option<NaiveDate>, Option<String>, Option<String>)`; sector/industry written via `COALESCE` upsert
- [x] `lagrange_history` — add `ticker_count INTEGER` column (migration 0020)
- [x] `state.rs` — add `universe_page: usize`, `universe_filter_zone: Option<String>` stubs
- [ ] Plan the Universe Explorer panel layout (wireframe in comments or separate WIREFRAME.md)

---

### v2.0 Roadmap — "The Oracle's Desk"

**Theme:** Astrology is THE product differentiator. Everything else verifies the astrological signal.

- [x] v2.0.1 — Swiss Ephemeris accuracy upgrade (13 bodies, sub-arcsecond, houses)
- [x] v2.0.2 — Extended aspect system (9 types, applying/separating, planetary dignity)
- [x] v2.0.3 — Horoscope reading engine (narrative interpretation per ticker)
- [x] v2.0.4 — Astro-first scraper pipeline (astro Phase 1, financial Phase 2)
- [x] v2.0.5 — Lagrange Score rebalance (Astro 40%, Financial 25%, Macro 20%, Short 15%)
- [x] v2.0.6 — Tab navigation system (Astrology as first tab, 6 tabs total)
- [x] v2.0.7 — Keyboard navigation (Ctrl+1..6 tabs, Ctrl+T search, Ctrl+R refresh, Escape clear)
- [x] v2.0.8 — Score calibration (sigmoid normalization, theme-score reconciliation)

### v2.2 Roadmap — "The Owner's Lens"

**Theme:** Fundamentals panel, DCF calculator, circadian theming. Financial data to verify the astro signal.

- [x] v2.2.1 — Fundamental analysis panel + FMP scraper (22 metrics, two-column grid)
- [x] v2.2.2 — DCF intrinsic value calculator (two-stage model, margin of safety)
- [x] v2.2.3 — Circadian theme system (4 phases, 3 modes: Auto/Light/Dark)
- [x] v2.2.4 — Two-column Overview layout (FillPortion 3:2 split)

### v2.4 Roadmap — "The Council"

**Theme:** AI agent personas that review the astrological reading. Comparative analysis. Technical pattern detection. Three product pillars converge.

- [x] v2.4.1 — AI agent personas (Buffett/Graham/Lynch/Munger template analysis)
- [x] v2.4.2 — Comparative analysis (up to 4 tickers, LATERAL JOIN, astro + fundamentals)
- [x] v2.4.3 — Technical pattern recognition (Golden/Death Cross, Double Top/Bottom, Support/Resistance, SMA200)

### v2.6 Roadmap — "The Explorer"

**Theme:** Universe Explorer, sector heat map, multiple watchlists. Navigate the full scored universe with astro rankings front and center.

- [x] v2.6.1 — Universe Explorer panel (paginated table, zone/sector filters, 11 columns, astro-first)
- [x] v2.6.2 — Sector heat map (canvas widget, proportional cells, two-phase color interpolation)
- [x] v2.6.3 — Multiple watchlists + CSV export (CRUD, named watchlists, rfd file dialog)

### v2.8 Roadmap — "The Strategist"

**Theme:** Backtesting (astro-driven strategies), portfolio gain/loss tracking. Empirical validation of the astro signal.

- [x] v2.8.1 — Astro-driven backtesting engine (configurable thresholds, signal accuracy metric, trade log)
- [x] v2.8.2 — Portfolio gain/loss tracking (unrealized P&L, color coding, astro score per holding)

### v3.0 Roadmap — "The Terminal"

**Theme:** Terminal-grade polish. Advanced charting, strategy builder, transaction log, settings persistence, astro calendar. All 7 Berkshire principles in final form.

- [x] v3.0.1 — Advanced charting (volume bars, timeframe selector 1M/3M/6M/1Y/ALL, astro event markers)
- [x] v3.0.2 — Strategy builder (8 condition types, AND/OR logic, quick-add buttons, backtest integration)
- [x] v3.0.3 — Portfolio transaction log (BUY/SELL CRUD, CHECK constraints, color-coded action labels)
- [x] v3.0.4 — Settings panel (DB-backed key-value store, theme/refresh persistence, dashboard info)
- [x] v3.0.5 — Astro calendar (monthly canvas heat map, score-colored days, prev/next navigation)
- [x] v3.0.6 — Post-release bugfix patch (3 bugs + 16 compiler warnings)
- [x] v3.0.7 — Dashboard UI review (6 bugs + 6 UX improvements from video review)

### v3.1 Roadmap — "The Network"

**Theme:** New data integrations ported from FinceptTerminal (C++20/Qt6). Three free, no-API-key data sources that expand our coverage: international economics, 60+ news feeds, and prediction market sentiment. Each integration follows the existing scraper module pattern (`src/scraper/*.rs`).

**Reference:** `reference/fincept_terminal_api_catalog.md` (full API catalog), `reference/fincept_src/` (C++ source files for translation)

- [x] v3.1.0 — DBnomics international economics (free REST API, 70+ data providers, supplements FRED)
- [x] v3.1.1 — RSS news aggregation (25 curated feeds, `feed-rs` parser, parallel fetch)
- [x] v3.1.2 — Polymarket prediction markets (Gamma API discovery, category fallback, volume-ranked)
- [x] v3.1.3 — Dashboard wiring (international macro strip, RSS in Research tab, Polymarket in Overview)
- [x] v3.1.3b — Font scale setting + astro priority full scrape (AtomicU32 runtime scaling, 4 presets)
- [x] v3.1.4 — Video review bug fixes (6 fixes: Y-axis precision, Polymarket categories, calendar nav, macro strip, current price label)

See master plan: `.claude/plans/delegated-dazzling-globe.md` for full v2.0-v3.1 roadmap with dependency chain.

#### v3.0.6 Bugfix Patch (Post-Release Review)

Three bugs identified from scraper/dashboard output review, plus 16 compiler warnings cleaned.

**Bug 1 (CRITICAL): Score Polarization**
Astro scores clustered bimodally at 0-5 and 95-100 across 1642 tickers. Root cause: the logistic sigmoid in `natal.rs` fed raw `delta_sum` (range +/-300 with 50-70 aspects per ticker) into a sigmoid with k=0.04, which saturated to extremes. Fix: normalize `delta_sum` by `sqrt(aspect_count)` before the sigmoid, and increase k from 0.04 to 0.10. The sqrt normalization preserves the signal that "more aligned aspects = stronger" while preventing linear scaling from saturating. With sqrt(50) = 7.07, raw +/-300 becomes +/-42, and k=0.10 maps that through a properly-shaped bell curve centered at 50.

**Bug 2 (MODERATE): Theme-Score Mismatch**
Tickers like AG scored 64 ("Greed") but had theme "Mild Caution & Restructuring". Root cause: the theme classifier in `interpretation.rs` counts aspect frequencies by planet type (how many Saturn aspects), while the score sums signed deltas (magnitude). These independent pathways can disagree. The reconciliation logic only overrode themes at extremes (score 65+ or 0-35), leaving scores 36-64 unreconciled. Fix: expand reconciliation window to 56+/0-44, aligning with the score label boundaries (Neutral = 45-55).

**Bug 3 (MINOR): AstroRanking Not Consumed**
The `AstroRanking` struct was built in Phase 1 but stored as `_ranking` (underscore = intentionally unused). Phase 2 functions used the hardcoded `WATCHLIST` constant, ignoring the astro-prioritized tickers. Fix: renamed to `ranking`, added `fetch_priority_prices()` to `prices.rs` that fetches price data for top/bottom astro tickers before the watchlist. Priority tickers already in the watchlist are skipped to avoid wasting API calls.

**Compiler Warnings (16 total, all resolved):**
- 5 true dead code removed: unused `Theme` import, unused `Logic` import, unused variable `n`, unused `shortcut()` method, unused `TEXT_XL` constant
- 9 incomplete feature fields suppressed with `#[allow(dead_code)]`: AgentContext (6 fields), CalendarDay.label, DcfResult (3 fields), SectorSummary.avg_lagrange, PortfolioPnlRow.notes, TransactionRow.notes, AstroRanking.total_scored, FmpKeyMetrics.earnings_yield_ttm
- 2 partially wired strategy features suppressed: MacdCrossUp/MacdCrossDown variants, all_options(), Strategy.name

---

#### v3.0.7 Implementation Plan

**Phase 1 (highest impact):**
1. Bug 5+6: Rewrite `fetch_universe_page()` in `db.rs:575-615` to use `astro_scores` as primary table with LEFT JOIN on `lagrange_history`. Universe goes from 3 tickers to ~1675. Astro scores come from source table.
2. Bug 4: Fix search autocomplete race condition. Add `autocomplete_just_selected` guard in `state.rs`, check in `TickerSearchInput` handler. Add click-outside dismiss via mouse_area in `view.rs`.

**Phase 2 (features):**
3. Bug 7: Wire horoscope reading display. Add `fetch_horoscope()` to `db.rs`, load in `update.rs` on ticker selection, render in `view.rs` astrology section.
4. Bug 9: Add `fetch_ticker_earnings()` to `db.rs` filtered by current ticker. Relabel watchlist section.

**Phase 3 (scraper):**
5. Bug 8: Replace AV `bail!` on "Information" response with sleep(60s) + retry in `prices.rs:92-94`.

**Phase 4 (UX polish):**
6. UX-1: Natal wheel 240px to 300px (`view.rs:665`). UX-2: Astro calendar color legend. UX-3: Deduplicate institutional holders. UX-4: Tab-contextual title bar. UX-5: Agent graceful empty state. UX-6: Verify recently viewed overflow.

---

### v3.1 Implementation Plan — "The Network" (FinceptTerminal Integrations)

**Source material:** FinceptTerminal C++20/Qt6 codebase, cataloged at `reference/fincept_terminal_api_catalog.md`. Key source files saved locally at `reference/fincept_src/` for direct Rust translation.

**Design principle:** Each integration follows the existing scraper pattern: an async function in `src/scraper/*.rs` that fetches data via `reqwest`, stores in PostgreSQL via `sqlx`, and logs via `log_fetch()`. No new architectural patterns needed for the scraper side. Dashboard wiring follows the standard state/update/view/db cycle.

#### v3.1.0 — DBnomics International Economics

**What:** DBnomics (`https://api.dbnomics.org`) aggregates 70+ economic data providers (FRED, ECB, BIS, Eurostat, World Bank, OECD, etc.) into a single free REST API. No API key required. This supplements our existing FRED integration with international economics data.

**Why this first:** Simplest integration. Pure REST + JSON. Their C++ implementation (`reference/fincept_src/services/dbnomics/DBnomicsService.cpp`, 372 lines) maps 1:1 to our `reqwest` + `serde_json` pattern. No new crate dependencies.

**Porting from FinceptTerminal:**
- `DBnomicsService::fetch_providers()` -> `fetch_dbnomics_providers()` in Rust
- `DBnomicsService::fetch_datasets()` -> `fetch_dbnomics_datasets()` in Rust
- `DBnomicsService::fetch_series()` -> `fetch_dbnomics_series()` in Rust
- `DBnomicsService::fetch_observations()` -> `fetch_dbnomics_observations()` in Rust
- `DBnomicsService::global_search()` -> `search_dbnomics()` in Rust
- Their pagination pattern (offset-based, 50 per page) translates directly
- Their debounce timer (300ms) maps to a simple `tokio::time::sleep` guard

**REST Endpoints (from their C++ source):**
```
GET /providers                                    -- list all 70+ data providers
GET /datasets/{provider}?limit=50&offset=N        -- datasets per provider
GET /series/{provider}/{dataset}?limit=50&offset=N&observations=false  -- series listing
GET /series/{provider}/{dataset}/{series}?observations=1&format=json   -- actual data points
GET /search?q={query}&limit=20&offset=N           -- global search across all providers
```

**Pre-selected series for our macro dashboard (replacing/supplementing FRED):**
| Series ID | Provider | Description | Replaces |
|-----------|----------|-------------|----------|
| `FRED/DGS10` | FRED | 10-Year Treasury Yield | Already have via FRED |
| `ECB/FM/M.U2.EUR.RT.MM.EURIBOR3MD_.HSTA` | ECB | Euribor 3M | New: EU rates |
| `BIS/WS_CBPOL/M.CN.CNY.N.O` | BIS | PBoC policy rate | New: China rates |
| `IMF/WEO/USA.NGDP_RPCH` | IMF | US GDP growth forecast | New: IMF projections |
| `Eurostat/prc_hicp_manr/M.I15.CP00.EA` | Eurostat | Eurozone CPI | New: EU inflation |
| `OECD/MEI_CLI/LOLITOAA.USA.M` | OECD | US Leading Indicator | New: OECD CLI |

**New files:**
- `src/scraper/dbnomics.rs` -- scraper module (~150 lines)
- `migrations/0021_dbnomics_series.sql` -- storage table

**Migration:**
```sql
CREATE TABLE IF NOT EXISTS dbnomics_series (
    id SERIAL PRIMARY KEY,
    provider TEXT NOT NULL,       -- e.g., 'ECB', 'BIS', 'OECD'
    dataset TEXT NOT NULL,        -- e.g., 'FM', 'WS_CBPOL'
    series_code TEXT NOT NULL,    -- e.g., 'M.U2.EUR.RT.MM.EURIBOR3MD_.HSTA'
    obs_date DATE NOT NULL,
    value DOUBLE PRECISION,
    label TEXT,                   -- human-readable name
    fetched_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (provider, dataset, series_code, obs_date)
);
```

**Scraper implementation checklist:**
- [x] Create `src/scraper/dbnomics.rs` with `fetch_all_dbnomics()` entry point
- [x] Define `DbnomicsResponse` and `DbnomicsObservation` serde structs
- [x] Implement `fetch_series_observations(client, provider, dataset, series)` for 6 pre-selected series
- [ ] Rate limit: 3 req/s (not implemented; sequential fetch suffices for 6 series)
- [x] Store observations in `macro_indicators` table with `DBNOMICS:` prefix (reused existing table instead of separate `dbnomics_series`)
- [x] Wire into `src/scraper/main.rs` Phase 3 (step 3.6)
- [x] Add `mod dbnomics;` to main.rs
- [x] Log via `log_fetch(pool, "dbnomics", None, series_id, "ok", None)`

**Dashboard wiring:**
- [x] Reuses existing `fetch_macro_data(pool)` -- `DBNOMICS:` prefixed series appear alongside FRED data
- [x] Reuses existing `macro_data: Vec<MacroIndicator>` state field (zero new types needed)
- [x] Loaded on startup alongside existing macro_data (same query)
- [x] `view.rs`: International macro strip row added (Euribor 3M, PBoC Rate, EU CPI, OECD CLI, US Credit/GDP)

---

#### v3.1.1 — RSS News Aggregation

**What:** Parallel fetching of 60+ financial RSS/Atom feeds, parsed natively with no API key. Supplements our existing Finnhub news (which is limited to ~50 articles/day on free tier).

**Why:** FinceptTerminal's `NewsService.cpp` (1,327 lines) fetches 80+ feeds in parallel with XML parsing, tiered priority, and deduplication. We'll port the top 60 financial feeds (skipping regional/niche ones). This gives us always-fresh news from Reuters, WSJ, CNBC, Bloomberg, SEC, Federal Reserve, and more.

**Porting from FinceptTerminal:**
- `NewsService::default_feeds()` -> hardcoded `Vec<RssFeed>` constant in Rust
- `NewsService::fetch_all_news()` -> `fetch_rss_news()` with `tokio::join!` parallelism
- `NewsService::parse_rss_items()` -> `parse_feed()` using `feed-rs` crate
- Their parallel fetch + shared state + atomic counter pattern maps to `futures::join_all` in Rust
- Their tier system (1-4) for feed priority, we'll keep as a simple integer field
- Their deduplication (by link URL) maps to a SQL UNIQUE constraint

**Feed selection (60 feeds from their 80+, organized by tier):**

**Tier 1 (Wire services, regulatory -- always fetch first):**
```
Reuters Business:    https://feeds.reuters.com/reuters/businessNews
Reuters Markets:     https://feeds.reuters.com/reuters/financialsNews
AP Top News:         https://rsshub.app/apnews/topics/ap-top-news
SEC Press Releases:  https://www.sec.gov/news/pressreleases.rss
Federal Reserve:     https://www.federalreserve.gov/feeds/press_all.xml
IMF News:            https://www.imf.org/en/News/rss?language=eng
ECB Press:           https://www.ecb.europa.eu/rss/press.html
Bank of England:     https://www.bankofengland.co.uk/rss/news
```

**Tier 2 (Major financial media):**
```
Bloomberg Markets:   https://feeds.bloomberg.com/markets/news.rss
WSJ Markets:         https://feeds.a.dj.com/rss/RSSMarketsMain.xml
MarketWatch:         https://feeds.marketwatch.com/marketwatch/topstories/
CNBC Top News:       https://search.cnbc.com/rs/search/combinedcms/view.xml?partnerId=wrss01&id=100003114
Seeking Alpha:       https://seekingalpha.com/market_currents.xml
BBC Business:        http://feeds.bbci.co.uk/news/business/rss.xml
Benzinga:            https://www.benzinga.com/feed
Investing.com:       https://www.investing.com/rss/news.rss
FXStreet:            https://www.fxstreet.com/rss/news
OilPrice:            https://oilprice.com/rss/main
Kitco Gold:          https://www.kitco.com/rss/news/
CoinDesk:            https://www.coindesk.com/arc/outboundfeeds/rss/
```

**Tier 3 (Specialized/analysis):**
```
ZeroHedge:           https://feeds.feedburner.com/zerohedge/feed
Calculated Risk:     https://feeds.feedburner.com/CalculatedRisk
Wolf Street:         https://wolfstreet.com/feed/
TechCrunch:          https://techcrunch.com/feed/
Ars Technica:        https://feeds.arstechnica.com/arstechnica/index
(+35 more from FinceptTerminal's feed list)
```

**New crate dependency:** `feed-rs = "2"` (unified RSS 2.0 / Atom / JSON Feed parser)

**New files:**
- `src/scraper/rss_news.rs` -- scraper module (~200 lines)
- `migrations/0022_rss_articles.sql` -- storage table

**Migration:**
```sql
CREATE TABLE IF NOT EXISTS rss_articles (
    id SERIAL PRIMARY KEY,
    feed_id TEXT NOT NULL,          -- e.g., 'reuters-biz', 'wsj-markets'
    title TEXT NOT NULL,
    link TEXT NOT NULL UNIQUE,      -- dedup key
    summary TEXT,
    published_at TIMESTAMPTZ,
    source_name TEXT NOT NULL,      -- e.g., 'Reuters', 'WSJ'
    category TEXT,                  -- 'MARKETS', 'REGULATORY', 'ECONOMIC', 'TECH', 'CRYPTO'
    region TEXT DEFAULT 'GLOBAL',   -- 'US', 'EU', 'ASIA', 'GLOBAL'
    tier INT DEFAULT 2,
    fetched_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_rss_articles_published ON rss_articles(published_at DESC);
CREATE INDEX idx_rss_articles_category ON rss_articles(category);
```

**Scraper implementation checklist:**
- [x] Add `feed-rs = "2"` to `Cargo.toml`
- [x] Create `src/scraper/rss_news.rs` with `fetch_all_rss()` entry point
- [x] Define `RssFeed` struct with id, name, url, category, source fields
- [x] Hardcode 25 curated feeds (top sources from FinceptTerminal's 80+, organized by wire/financial/analysis/crypto)
- [x] Implement `fetch_single_feed(client, feed)` with 5s per-feed timeout
- [x] Parallel fetch all feeds using `tokio::spawn` + `join_all`
- [x] Parse via `feed_rs::parser::parse()`, extract title/link/summary/date
- [x] Dedup by link URL (SQL `UNIQUE` constraint on `link` column)
- [x] HTML stripping via state-machine parser, truncated to 300 chars
- [x] Wire into `src/scraper/main.rs` Phase 3 (step 3.7)
- [ ] Prune articles older than 30 days on each run (not yet implemented)
- [x] Log via `log_fetch(pool, "rss", None, "rss_articles", "ok", None)`

**Dashboard wiring:**
- [x] `db.rs`: `fetch_rss_articles(pool)` -- latest 50 articles across all feeds
- [x] `models.rs`: `RssArticle` struct with source_name, category, published_at
- [x] `view.rs`: "Market News (RSS)" section in Research tab with date, source, category badge, headline, Open button
- [ ] Category filter buttons (Markets / Regulatory / Economic / Tech / Crypto) — not yet implemented

---

#### v3.1.2 — Polymarket Prediction Markets

**What:** Polymarket is a prediction market where users bet on real-world outcomes (Fed rate decisions, elections, corporate events). The probability prices are a crowd-sourced sentiment signal. No API key required for read-only access.

**Why:** Novel signal that no other financial terminal integrates as a market indicator. "What does the market think the probability of a rate cut is?" directly informs macro analysis. FinceptTerminal's `PolymarketService.cpp` (843 lines) implements all three Polymarket APIs.

**Porting from FinceptTerminal:**
- `PolymarketService::fetch_markets()` -> `fetch_polymarkets()` in Rust
- `PolymarketService::fetch_events()` -> `fetch_poly_events()` in Rust
- `PolymarketService::search_markets()` -> `search_polymarkets()` in Rust
- `PolymarketService::fetch_price_history()` -> `fetch_poly_prices()` in Rust
- Their three-base-URL pattern (Gamma for discovery, CLOB for pricing, Data for analytics)
- Their `num_or_str()` JSON helper (CLOB returns numbers as strings)

**Three API bases (from their C++ source):**
```
Gamma (discovery): https://gamma-api.polymarket.com
  GET /markets?closed=false&limit=50&offset=N&order=volume  -- list markets
  GET /markets/{id}                                          -- market detail
  GET /markets?_q={query}&limit=20                          -- search
  GET /events?closed=false&limit=50&offset=N                -- events
  GET /tags                                                  -- categories

CLOB (pricing):    https://clob.polymarket.com
  GET /book?token_id={id}                                   -- order book
  GET /prices-history?market={id}&interval=1d&fidelity=100  -- price history
  GET /midpoint?token_id={id}                               -- current probability

Data (analytics):  https://data-api.polymarket.com
  GET /trades?market={id}&limit=100                         -- recent trades
  GET /v1/leaderboard?category=OVERALL&orderBy=PNL&limit=20 -- top traders
```

**Pre-selected markets for macro sentiment:**
| Market Question | Signal Type | How We Use It |
|----------------|-------------|---------------|
| "Will the Fed cut rates in [month]?" | Monetary policy probability | Macro indicator: rate cut odds |
| "US Recession in 2026?" | Recession probability | Macro fear/greed input |
| "Will S&P 500 reach [level] by [date]?" | Market direction | Bull/bear sentiment |
| "Will inflation be above 3% in [month]?" | Inflation expectations | Macro indicator |

**New files:**
- `src/scraper/polymarket.rs` -- scraper module (~250 lines)
- `migrations/0023_polymarket.sql` -- storage tables

**Migration:**
```sql
CREATE TABLE IF NOT EXISTS polymarket_events (
    id SERIAL PRIMARY KEY,
    poly_id INT NOT NULL UNIQUE,
    question TEXT NOT NULL,
    slug TEXT,
    volume DOUBLE PRECISION,
    liquidity DOUBLE PRECISION,
    end_date TIMESTAMPTZ,
    active BOOLEAN DEFAULT TRUE,
    category TEXT,              -- matched to our signal types
    fetched_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS polymarket_prices (
    id SERIAL PRIMARY KEY,
    poly_market_id INT NOT NULL,
    token_id TEXT NOT NULL,
    probability DOUBLE PRECISION NOT NULL,  -- 0.0 to 1.0
    price_date DATE NOT NULL,
    fetched_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (poly_market_id, token_id, price_date)
);
CREATE INDEX idx_polymarket_prices_date ON polymarket_prices(price_date DESC);
```

**Scraper implementation checklist:**
- [x] Create `src/scraper/polymarket.rs` with `fetch_all_polymarket()` entry point
- [x] Define `GammaMarket` serde struct (Gamma API response format)
- [x] Implement Gamma API: `fetch_markets_by_tag()` for 6 financial tags + `fetch_top_markets()` by volume
- [ ] Implement CLOB API: `fetch_midpoint(client, token_id)` for current probability — not yet ported
- [ ] Implement CLOB API: `fetch_price_history(client, token_id)` for trend — not yet ported
- [x] Fetch top markets by volume across financially relevant tags on each run
- [x] Store markets + probabilities in `polymarket_markets` table via upsert
- [ ] Rate limit: not implemented (Gamma API is generous; CLOB will need it when ported)
- [x] Wire into `src/scraper/main.rs` Phase 3 (step 3.8)
- [x] `num_or_str()` + `parse_str_or_array()` helpers for Gamma's mixed JSON types
- [x] Category fallback: market.category -> first tag -> query tag (capitalized)

**Dashboard wiring:**
- [x] `db.rs`: `fetch_polymarket(pool)` -- top 20 active markets by volume
- [x] `state.rs`: `pub polymarket: Vec<PolymarketMarket>` + `PolymarketLoaded` message
- [x] `view.rs`: "Prediction Markets" section in Overview tab with Yes%, category badge, question, volume
- [ ] Optional: integrate rate-cut probability into Lagrange macro sub-score — not yet implemented

---

#### v3.1.3 — Dashboard Integration & New Tab Sections

**What:** Wire all three new data sources into the dashboard UI. Add an "International Economics" subsection to the macro strip, expand the news section with RSS feeds, and add a Polymarket sentiment gauge.

**Checklist:**
- [x] Macro strip: international indicators row added (Euribor 3M, PBoC Rate, EU CPI, OECD CLI, US Credit/GDP)
- [x] News section: RSS articles displayed in Research tab with source badge and category
- [ ] Research tab: category filter buttons (Markets / Regulatory / Economic / Tech / Crypto) — not yet implemented
- [x] Overview tab: Polymarket prediction markets section with Yes%, category, question, volume
- [ ] Settings tab: toggles for enabling/disabling each new data source — not yet implemented
- [ ] Consider: Polymarket rate-cut probability as input to Lagrange macro sub-score — not yet implemented

---

#### v3.1 Dependency Chain

```
v3.0.7 (bug fixes) -- must complete first
 +-- v3.1.0 (DBnomics) -- standalone, no dependencies on other v3.1.x
 +-- v3.1.1 (RSS News) -- standalone, no dependencies on other v3.1.x
 +-- v3.1.2 (Polymarket) -- standalone, no dependencies on other v3.1.x
 +-- v3.1.3 (Dashboard wiring) -- requires all three above
```

v3.1.0, v3.1.1, and v3.1.2 can be implemented in parallel. Each is an independent scraper module. v3.1.3 wires them all into the dashboard.

#### v3.1 Estimated Effort

| Version | New Files | New Crates | Migration | Lines |
|---------|-----------|------------|-----------|-------|
| v3.1.0 DBnomics | `scraper/dbnomics.rs` | None | 0021 | ~150 |
| v3.1.1 RSS News | `scraper/rss_news.rs` | `feed-rs` | 0022 | ~200 |
| v3.1.2 Polymarket | `scraper/polymarket.rs` | None | 0023 | ~250 |
| v3.1.3 Dashboard | -- | -- | -- | ~100 |
| **Total** | **3 new files** | **1 crate** | **3 migrations** | **~700 lines** |

---

### External Reference: FinceptTerminal

**Source:** https://github.com/Fincept-Corporation/FinceptTerminal (C++20/Qt6, ~13k stars)
**Local catalog:** `reference/fincept_terminal_api_catalog.md`
**Local source:** `reference/fincept_src/` (key files fetched for Rust porting reference)

FinceptTerminal integrates 13 API providers via a DataHub pub/sub architecture. Key integrations relevant to our project:

| Provider | API Key | Cost | Priority | What We'd Gain |
|----------|---------|------|----------|----------------|
| DBnomics | None | Free | HIGH | 70+ economic data providers, supplements FRED |
| RSS Feeds (80+) | None | Free | MEDIUM | Massive news coverage, supplements Finnhub |
| Polymarket | None | Free | MEDIUM | Prediction market probabilities as sentiment signal |
| Databento | Required | Paid | LOW | Institutional derivatives data (v4.x) |
| WebSockets | Varies | Varies | LOW | Real-time streaming (v4.x) |
| Broker APIs | Required | Varies | LOW | Trading execution (v5.x) |

**Architecture pattern:** Their DataHub pub/sub with per-topic TTL, rate limits, and request coalescing is worth porting. Maps to `tokio::broadcast` + `governor` crate in Rust.

---

### Backlog (no version assigned yet)

- [ ] `docker-compose.yml` — reproducible local PostgreSQL setup
- [ ] FINRA API token refresh — current key expires when session does; need OAuth2 refresh flow
- [ ] Polygon.io Starter plan ($29/mo) — full options snapshot endpoint
- [ ] One-click installer / packaged binary for distribution
- [ ] Databento derivatives data — institutional-grade options vol surfaces, Greeks (paid API, v4.x)
- [ ] WebSocket real-time streaming — Alpaca/Polygon live price feeds via `tokio-tungstenite` (v4.x)
- [ ] Broker trading integration — Alpaca REST API for order execution (v5.x)
- [ ] DataHub pub/sub architecture — FinceptTerminal-style topic routing with per-producer rate limits (v4.x)

### Completed

- [x] Post-release bugfix — score polarization fix, theme reconciliation, AstroRanking wiring, 16 warnings cleaned — v3.0.6
- [x] Advanced charting — volume bars, timeframe selector, astro event markers on chart — v3.0.1
- [x] Strategy builder — composable condition chains with AND/OR logic, backtest integration — v3.0.2
- [x] Transaction log — BUY/SELL CRUD with CHECK constraints, color-coded display — v3.0.3
- [x] Settings panel — DB-persisted key-value store, theme/refresh persistence — v3.0.4
- [x] Astro calendar — monthly canvas heat map colored by daily astro score — v3.0.5
- [x] Astro-driven backtesting — configurable thresholds, signal accuracy, trade log — v2.8.1
- [x] Portfolio gain/loss tracking — unrealized P&L with astro scores per holding — v2.8.2
- [x] Universe Explorer — full scored universe with pagination, zone/sector filters, 11 columns — v2.6.1
- [x] Sector heat map — canvas-rendered proportional heat map by average astro score — v2.6.2
- [x] Multiple watchlists + CSV export — named watchlists CRUD + native file dialog export — v2.6.3
- [x] AI agent personas (Buffett/Graham/Lynch/Munger) — template analysis through 4 investment lenses — v2.4.1
- [x] Comparative analysis — side-by-side multi-ticker comparison with astro scores — v2.4.2
- [x] Technical pattern recognition — Golden/Death Cross, Double Top/Bottom, Support/Resistance — v2.4.3
- [x] Horoscope reading engine — template-based narrative interpretations per ticker — v2.0.3
- [x] Swiss Ephemeris (`swiss-eph` FFI) — sub-arcsecond planetary accuracy — v2.0.1
- [x] CPI display: YoY% via SQL CTE — v0.7.0
- [x] Lagrange Score history accumulation — v0.7.0
- [x] Universal ticker seed + birth chart database — v0.8.0
- [x] Dynamic ticker search + recently viewed — v0.8.0
- [x] Alert threshold system — v0.9.0
- [x] IPO date enrichment pipeline (4 sources) — v1.0.0
- [x] Tiingo bulk price history — v1.0.0
- [x] Codebase DRY refactor (`enrich_common.rs`) — v1.0.0

---

## Appendix: Astrology Engine Implementation Detail

### Calculation Engine

**v2.0.1+:** Swiss Ephemeris (C library compiled into binary via FFI). Sub-arcsecond accuracy. No external API, no network calls. The `embedded-ephe` feature bakes coefficient files into the binary.
**Planets (13 bodies):** Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, North Node, South Node, Chiron.
**Houses:** Whole Sign house system computed at NYSE location (40.7128 degrees N, 74.0060 degrees W). Ascendant + Midheaven stored in `natal_angles` table.

**Legacy (v0.5.0-v1.1.0):** Pure Rust math using Jean Meeus *Astronomical Algorithms* formulas. 10 planets only (no nodes or Chiron). Accurate for Sun and Moon, but 20-150 degree errors for all other planets due to simplified heliocentric-to-geocentric conversion. Retained as fallback if Swiss Eph fails.

### Aspect Scoring (v2.0.2)

**9 aspect types (5 major + 4 minor):**

| Aspect | Angle | Orb | Nature | Major/Minor |
|--------|-------|-----|--------|-------------|
| Conjunction | 0 degrees | 8 degrees | Neutral (depends on planets) | Major |
| Semi-sextile | 30 degrees | 2 degrees | Harmonious | Minor |
| Semi-square | 45 degrees | 2 degrees | Challenging | Minor |
| Sextile | 60 degrees | 6 degrees | Harmonious | Major |
| Square | 90 degrees | 8 degrees | Challenging | Major |
| Trine | 120 degrees | 8 degrees | Harmonious | Major |
| Sesquiquadrate | 135 degrees | 2 degrees | Challenging | Minor |
| Quincunx | 150 degrees | 3 degrees | Stressful adjustment | Minor |
| Opposition | 180 degrees | 8 degrees | Challenging | Major |

**Score formula (v3.0.6, sqrt-normalized sigmoid):**
```
aspect_delta  = base_magnitude x direction x orb_mod x minor_mod x dignity_mod
final_delta   = aspect_delta x applying_mod
raw_x         = sum(final_deltas) + moon_modifier
normalized_x  = raw_x / sqrt(aspect_count)
astro_score   = 100 / (1 + e^(-0.10 x normalized_x))
if mercury_rx: astro_score = min(astro_score, 65)

Where:
  base_magnitude: 5-15 based on planet natures (Benefic/Malefic/Neutral)
  direction: +1 (harmonious), -1 (challenging), depends on planets (conjunction)
  orb_mod: 1.0 (exact) to 0.25 (at max orb), linear
  minor_mod: 1.0 (major), 0.5 (minor)
  dignity_mod: 1.2 (Domicile/Exalted), 0.8 (Detriment/Fall), 1.0 (Peregrine)
  applying_mod: 1.5 (applying), 0.7 (separating)
  aspect_count: number of active transit-to-natal aspects (min 1)

Score distribution: bell-shaped centered at 50.
  normalized_x = 0 -> score 50
  normalized_x = +/-3 -> score ~57/~43
  normalized_x = +/-6 -> score ~65/~35
  normalized_x = +/-10 -> score ~73/~27
```

### IPO Seed Data

| Ticker | Company | IPO Date | Exchange |
|--------|---------|----------|----------|
| AAPL | Apple | 1980-12-12 | NASDAQ |
| MSFT | Microsoft | 1986-03-13 | NASDAQ |
| GOOGL | Alphabet | 2004-08-19 | NASDAQ |
| AMZN | Amazon | 1997-05-15 | NASDAQ |
| META | Meta | 2012-05-18 | NASDAQ |
| TSLA | Tesla | 2010-06-29 | NASDAQ |
| NVDA | NVIDIA | 1999-01-22 | NASDAQ |
| JPM | JPMorgan Chase | 1969-05-01 | NYSE |
| V | Visa | 2008-03-19 | NYSE |
| UNH | UnitedHealth | 1984-10-17 | NYSE |

All at 09:30 EST. Coordinates: New York (40.7128°N, 74.0060°W).
