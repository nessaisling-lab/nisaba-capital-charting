# Changelog — Pursuit Week 4 Financial Dashboard

**Author:** Aisling Leiva
**Stack:** Rust, Iced 0.14, SQLx, PostgreSQL
**Development:** 2026-04-07 to 2026-05-04

---

## v11.4.0-w6.0 — "The Reliability" (2026-05-04)

**Theme:** Wave 6.0 — paired financial-data + astrology-engine expansion. Track A removes single-points-of-failure in price data. Track B adds geometric pattern recognition the engine was missing.

### Track A — 6.A1 Multi-Source Price Fallback

New `src/scraper/sources/` module with two adapters and a cascade dispatcher:
- **Yahoo Finance** (`sources/yahoo.rs`) — v8 chart API at `query1.finance.yahoo.com`. JSON shape: `chart.result[0].timestamp[] + indicators.quote[0].{open,high,low,close,volume}`. Browser-style User-Agent required (Yahoo blocks default reqwest UA). 3-month default range, daily interval.
- **Stooq** (`sources/stooq.rs`) — CSV at `stooq.com/q/d/l/?s={ticker}.us&i=d`. Header parsing strict ("Date,Open,High,Low,Close,Volume" required). Returns "No data" for unknown tickers.
- **Cascade** (`sources/mod.rs`) — `fetch_fallback_chain(ticker, client)` tries Yahoo then Stooq, returns `(rows, source_name)`. Each failure logged with source identity for operator diagnostics.

`prices::fetch_and_store` refactored: AV primary path → on rate-limit/error → cascade fallback. Provenance tagged at insert via new `data_source` column. AV writes `'alpha_vantage'`, fallback writes `'yahoo'` or `'stooq'`.

Migration `0038_price_data_source.sql`: adds `data_source TEXT NOT NULL DEFAULT 'alpha_vantage'` column + GIN index on `price_data`.

### Track B — 6.B1 Aspect Pattern Recognition

New `src/astrology/patterns.rs` (~470 lines) detecting 7 geometric configurations from combined natal + transit positions:

| Pattern | Geometry | Strength |
|---------|----------|----------|
| Grand Trine | 3 planets, each pair 120° (orb 4°) | +15 |
| T-Square | opposition + 2 squares to apex (orb 6°) | -12 |
| Grand Cross | 4 planets, 2 oppositions + 4 squares (orb 6°) | -18 |
| Yod | 2 sextile + apex 150° from both (orb 3-4°) | -8 |
| Stellium | 3+ planets in same sign | +6 base, +4 per extra body |
| Mystic Rectangle | 4 planets, 2 sextiles + 2 trines + 2 oppositions (orb 4°) | +10 |
| Kite | Grand Trine + opposition from 4th planet | +18 |

Each pattern marked `is_cross` when bodies span both natal + transit (1.0× multiplier vs 0.6× for intra-chart). Tightness factor (0.5 + 0.5 × tightness) modulates final strength based on average orb across defining aspects.

`compute_transit_score` adds `pattern_score_total(&patterns)` to `delta_sum` BEFORE sigmoid normalization, so patterns meaningfully shift score without being washed out by aspect_count division.

`TransitScore` gains `patterns: Vec<AspectPattern>` field. New `patterns_to_json()` serializer in `natal.rs`. Scraper `compute_astro_scores` + `compute_astro_score_one` write JSON to new `aspect_patterns` JSONB column.

Migration `0037_aspect_patterns.sql`: adds `aspect_patterns JSONB DEFAULT '[]'` to `astro_scores` + GIN index for "show me all tickers with a Grand Trine" queries.

7/7 unit tests passing: exact Grand Trine, T-Square, Stellium, Yod (with corrected geometry — apex must be 150° from BOTH sextile ends, not 150° from the start), cross-chart marking, intra-chart marking, no false positives on random angles.

**Files modified:** 8 (`src/astrology/mod.rs`, `src/astrology/natal.rs`, `src/astrology/patterns.rs` new, `src/scraper/main.rs`, `src/scraper/astrology.rs`, `src/scraper/prices.rs`, `src/scraper/sources/mod.rs` new, `src/scraper/sources/yahoo.rs` new, `src/scraper/sources/stooq.rs` new, migrations `0037` + `0038`)

---

## v11.3.0 — "The Refinement" (2026-05-04)

**Theme:** All 22 video-review feedback items shipped across 5 waves. Polish pass on UI density, layout flow, chart overlays, council templates, gauges, and background atmosphere.

### Wave 1 — Quick Visual Wins
- **1a Aspect line thickness** — `ASPECT_W` 0.005 → 0.003 in `natal_wheel_3d.wgsl`. Reverts v9.3 thickening that produced "lines spitting at each other."
- **1b Section icons** — Phosphor icons (GLOBE/GRAPH_UP/CALENDAR/MOON_STARS/LIGHTNING) on Astrology tab section headers. New `icon_eyebrow()` helper in `shared.rs`.
- **1c Compact horoscope** — Moon/Mercury/Timing collapsed into single `row![]` at `text_xs`. "Mercury: Direct — clear communications" trimmed to "Mercury: Direct".
- **1d Scrollbar gutter audit** — New `gutter_scroll()` helper wraps content with 20px right padding + gold style. Applied to 13 sub-scrollables across 6 view files.
- **1e Galaxy background mute** — Desaturated nebula purples, fewer stars (threshold 0.92→0.93), slower twinkle (1.2→0.8). Less "screaming nebula."

### Wave 2 — Layout Restructure
- **2a Header price + H/L** — Ticker block now shows last close (gold) + day high/low pulled from `self.rows.last()`. No state changes needed.
- **2b Search above ornament** — `compact_nav` moved above `PageHeaderOrnament`. Ornament now divides header from tab bar.
- **2c Astrology tab reorder** — Horoscope extracted to standalone section. New flow: Natal → Calendar+Forecast (row) → Horoscope → Backtest → Strategy.
- **2d Two-column compression** — `row![calendar_col, forecast_col]` with `Length::FillPortion(1)` each. ~250px vertical compression.

### Wave 3 — UX + Chart Overlays (Iced 0.14 features)
- **3a Sector dropdown** — Button row → `pick_list` with "All" sentinel mapping to `None`.
- **3b Column header tooltips** — `tooltip()` widget on Fin/Mac/Sht/Conc with full names + descriptions. Custom `tip_style` container.
- **3c Rising sign backfill** — Seeder WHERE clause now matches tickers missing EITHER `natal_positions` OR `natal_angles`. Fixes blank Rising sign for tickers seeded pre-angles.
- **3d Per-ticker fetch scope** — New `seed_natal_chart_one` + `compute_astro_score_one` for `--ticker` mode. Fixes universe-wide loops on single-ticker fetches.
- **3e Planet symbols overlay** — `stack![shader, pin(text_glyph)]` with Unicode astrology glyphs (☉☽☿♀♂♃♄⛢♆♇☊☋⚷). Position math in `planet_pixel_pos()` accounts for camera tilt.
- **3f Chart hover tooltips** — Each glyph wrapped in `tooltip()` showing planet+sign+degree on hover.

### Wave 4 — Council Fix + Chart Polish
- **4a Council diversification** — Headline pools 3→6 variants per persona/verdict. Ticker-specific fundamental injection (ROE, P/E, PEG, op margin). New `headline_variant()` hashes char codepoints + score so AAPL/MSFT/META no longer share strings.
- **4b Chart size enum** — `ChartSize::{Compact 320, Default 400, Large 520}`. State field + pick_list in Astrology tab and Settings. `planet_pixel_pos` parametrized by `chart_px`.
- **4c Fetch progress bar** — `fetch_start_time: Option<Instant>` drives time-based fill (cap 0.85 over 30s). `FillPortion` split: gold filled / 15%-alpha track. Replaces indefinite pulse.

### Wave 5 — Deferred Polish
- **5a Tooltip size setting** — `TooltipSize::{Small/Default/Large}` enum. State field + Settings UI. Tuple `(font_px, box_w, box_h)` passed into `PriceChart` via new `tooltip_dims` field.
- **5b Scraper retry helper** — New `src/scraper/retry.rs` with `with_retry()` higher-order async helper (3 attempts, 2s/8s backoff). Wired into FMP fundamentals fetch (key-metrics + ratios endpoints).
- **5c Gauge reimagination** — Compass-rose detailing: outer gilt arc (45% gold), sundial tick marks (major every 25pt, minor every 5pt), gold-backed needle, 8-point star center cap.
- **5d Background texture** — Vignette shader gains 3 new layers: 8x60 horizontal "fiber" pattern (chain lines), 5x5 sepia-warm aging blotches, retains existing per-pixel grain. Renaissance parchment feel.

**Files modified:** 23 (`shaders/natal_wheel_3d.wgsl`, `shaders/vignette.wgsl`, `view/shared.rs`, `view/astrology_tab.rs`, `view/mod.rs`, `view/universe.rs`, `view/overview.rs`, `view/research.rs`, `view/fundamentals.rs`, `view/settings.rs`, `state.rs`, `agents.rs`, `gauges.rs`, `charts.rs`, `astrology.rs`, `update/astro.rs`, `update/data.rs`, `scraper/main.rs`, `scraper/astrology.rs`, `scraper/fundamentals.rs`, `scraper/retry.rs` new)

---

## v11.2.0 — "The Foundation" (2026-04-30)

**Iced 0.13 → 0.14 framework upgrade.** Major dependency migration touching 13+ source files across 19 breaking API changes.

### Breaking API Changes Resolved
| Category | Count | Change |
|----------|-------|--------|
| Shader system | 6 | `Storage` pattern → `Pipeline` trait (auto-creates on first frame) |
| wgpu 22 → 27 | 8 | `entry_point` now `Option`, new `compilation_options`/`cache`/`depth_slice` fields |
| Canvas events | 2 | `update()` returns `Option<Action<Message>>` instead of `(Status, Option<Message>)` |
| Widget renames | 17 | `horizontal_rule` → `rule::horizontal`, `Space::with_width` → `Space::new().width()` |
| Text alignment | 32 | `horizontal_alignment` → `align_x`, `vertical_alignment` → `align_y` |
| Application boot | 1 | First arg now boot fn, title via builder, `.run()` replaces `.run_with()` |
| Keyboard | 1 | `on_key_press(f)` → `keyboard::listen().filter_map()` |
| Palette | 1 | New `warning` field required |
| Scrollable | 1 | `Scroller.color` → `Scroller.background`, new `auto_scroll` field |
| Button style | 1 | New `snap` field required |

### Environment Fix
- Added `C:\msys64\ucrt64\bin` to Windows user PATH — `swiss-eph` C compilation now works from PowerShell (was only working from bash/MSYS2)

### Unblocked by Upgrade
- `pin` widget: absolute (x,y) positioning for planet labels over shader
- `float` widget: floating overlays with dynamic positioning
- `stack` improvements: `push_under` for shader-behind-UI layering
- Animation API: built-in animation primitives for hover transitions
- cosmic-text 0.15: better Unicode symbol rendering (zodiac glyphs)
- wgpu 27: modern GPU backend

**Files modified:** 19 (`Cargo.toml`, `shaders/mod.rs`, `charts.rs`, `main.rs`, `update/mod.rs`, `theme.rs`, `view/mod.rs`, `view/shared.rs`, `view/overview.rs`, `view/portfolio_tab.rs`, `view/universe.rs`, `view/paper_trail.rs`, `view/settings.rs`, `view/fundamentals.rs`, `view/astrology_tab.rs`, `calendar.rs`, `astrology.rs`, `gauges.rs`, `heatmap.rs`)

---

## v11.1.0 — "The Craft" (2026-04-30)

- **Clickable entity links:** Insider names and institutional holder names in Research tab now open Google search on click. Reusable `link_button()` helper in `shared.rs` with gold hover styling.
- **Tab glow rework:** Active tab now has 2px gold border with bookmark shape (rounded top corners, flat bottom) replacing the 15% alpha gold background fill. Hover tabs show faint gold border preview.
- **Chart layer visibility toggles:** 4 toggle buttons (Natal/Transit/Aspects/Retro) above the natal wheel. Eye/eye-slash Phosphor icons. State flows through Dashboard bools -> shader uniforms -> WGSL conditionals. Retrogrades handled in Rust packing (no extra shader uniform needed).
- **Nav layout redesign:** Two-row header: search bar (280px) left, ticker name centered, icon-only action buttons (refresh/fetch/theme) right. Second row: ticker DB buttons + recently viewed. Theme button changed from text to moon-stars icon.

**Files modified:** 8 (`view/shared.rs`, `view/research.rs`, `view/mod.rs`, `view/astrology_tab.rs`, `state.rs`, `update/astro.rs`, `shaders/mod.rs`, `shaders/natal_wheel_3d.wgsl`, `icons.rs`)

| Feature | Before | After |
|---------|--------|-------|
| Entity names | Plain text | Gold clickable links (Google search) |
| Active tab | Gold bg fill + sparkle | 2px gold border bookmark shape |
| Chart layers | Always all visible | 4 toggleable layers (eye/eye-slash) |
| Header layout | Single row, text buttons | Two-row, icon-only actions, search-left |

---

## v11.1 Video Review (2026-04-30)

15-minute screen recording review of v11.1 build. Audio transcribed via faster-whisper.

**Approved:** Tab bookmark borders, icon theme toggle, Universe legibility, Council verdict accuracy, overall layout direction.

**21 feedback items captured** across 5 categories:
- P0 Layout (5): Header price/high-low, search position, forecast-calendar merge, horoscope reposition, reduce dead space
- P0 Chart (5): Aspect lines too thick, galaxy bg rework, planet symbols, interactivity, chart size
- P1 UX (5): Sector dropdown, column tooltips, tooltip sizing, Rising sign bug, horoscope formatting
- P1 Visual/Data (5): Section icons, scrollbar gutter, gauge redesign, progress bar, data reliability
- P1 Bug (1): Council template responses too generic

Full feedback structured in TODOS.md with video timestamps.

---

## v11.0.0 — "The Intelligence" (2026-04-29)

- **90-day astro forecast:** Computed from natal positions + transit ephemeris, displayed as colored timeline events (favorable/unfavorable date ranges with aspect descriptions)
- **Big Three summary:** Sun/Moon/Rising signs displayed prominently above natal chart
- **Smart calculator defaults:** DCF growth% auto-fills from PEG ratio, Options Greeks vol% from historical volatility
- **Zodiac sign band + planet symbol legend:** Visual legend below natal chart showing planet glyphs with natal/transit/retro color coding
- **Pulsing loading bar:** Shimmer animation during data fetch operations
- **Icon-only nav buttons:** Action buttons converted to Phosphor icons

---

## v10.0.0 — "The Signal" (2026-04-29)

- **RSS tone sentiment:** Keyword-based sentiment scoring from 25 news feeds
- **Lagrange adaptive weighting:** Removed 50-default compression, signals now scale to their actual range
- **Richer agent verdicts:** Sector-aware, news-informed verdicts with 3 headline variants
- **Fetch-this-ticker button:** One-click data fetch for selected ticker

---

## v9.3.0 — "The Clarity" (2026-04-29)

- **Aspect lines overhaul:** Base width 0.003→0.005 (67% thicker). Alpha values boosted: conjunction 0.20→0.45, sextile 0.14→0.30, square 0.16→0.35, trine 0.20→0.40. Colors more saturated. Conjunctions now 2× base width (was 1.5×). Squares 1.3× width (new). Added outer glow halo (4× line width, 15% alpha) for luminous bleeding effect against galaxy background.
- **Universe table columns widened:** Astro/Score 56→64px, Fin/Macro/Short 44→52px, Concordance 90→100px. Headers no longer truncated ("Astr o" → "Astro").
- **Tab labels bolder:** Active tab label uses Fraunces Bold at 16px (was SemiBold at 14px). New `DISPLAY_BOLD` font constant. Active tabs visually heavier and more readable.
- **Scrollbar gutter:** Page content right padding 10→20px. Scrollbar no longer overlaps content text.

**Files modified:** 5

| Fix | Before | After |
|-----|--------|-------|
| Aspect lines | Thin (0.003), faint (0.14-0.20 alpha), no glow | Thick (0.005), bold (0.30-0.45 alpha), luminous glow halo |
| Universe headers | Truncated "Astr o", "Scor e" | Full "Astro", "Score" at proper width |
| Active tab label | Fraunces SemiBold 14px | Fraunces Bold 16px |
| Scrollbar | Overlaps content | Own gutter space (20px right padding) |

---

## v9.2.0 — "The Cosmos" (2026-04-29)

- **Galaxy background:** Natal chart background replaced from flat bg_color to procedural galaxy field. Deep space gradient (near-black center → dark purple edges) with nebula swirl (layered sine noise in purple/blue) and dense twinkling star field across entire chart. Stars have color variation: cool white (common), blue (medium), gold (rare). 60.0 grid density vs 45.0 for outer-only stars.
- **Active tab glow:** Active tab icon now renders in gold (was ink color), with warm gold background glow (15% alpha) and persistent subtle sparkle shimmer (2 particles). Tab feels "shining" when selected.

**Files modified:** 3

| Feature | Before | After |
|---------|--------|-------|
| Chart background | Flat bg_color (dark brown/cream) | Galaxy gradient + nebula swirl + dense colored star field |
| Active tab | Bold icon + gold underline | Gold icon + gold glow bg + persistent sparkle + gold underline |

---

## v9.1.0 — "The Polish" (2026-04-29)

- **[P0] Disable chart rotation:** Natal chart no longer spins, making planetary positions readable. Removed `u.time * 0.015` rotation transform from natal_wheel_3d.wgsl
- **[P0] Backtest crash fix:** Removed early `return` from backtest view that swallowed entire astrology tab. Added "Clear Results" button so users can dismiss backtest output
- **[P0] Broken watchlist icons:** Replaced Unicode "✕" with Phosphor `X_LG` icon for watchlist remove buttons. Previous character rendered as broken box
- **[P0] Tooltip contrast fix:** OHLCV hover tooltip now uses dark background card (0.12/0.10/0.08 RGB) with warm cream text + gold border accent. Readable in both Parchment and Leather themes. Font size 9→10px, tooltip wider (90→106px)

**Files modified:** 6

| Bug | Before | After |
|-----|--------|-------|
| Natal chart | Slowly rotating, hard to read | Static, readable positions |
| Backtest results | Swallows entire tab, no dismiss | Shows inline with Clear button |
| Watchlist icons | "✕" renders as broken box | Phosphor X_LG icon |
| Chart tooltip | White text on light bg, tiny | Dark card + cream text, gold border |

---

## v9.0.1 — Hotfix (2026-04-29)

- **WGSL array fix:** Changed orbital trail `trail_alphas` from `let` to `var` — WGSL `let` arrays cannot be indexed by loop variable, only `var` arrays support dynamic indexing. Caused shader validation crash on launch.
- **Roadmap update:** Added v9.1/v10.0/v11.0 milestones from video review (15min, 284 subtitle entries, 903 frames analyzed)
- **TODOS overhaul:** 4 P0 bugs + 16 items organized by milestone version

---

## v9.0.0 — "The Performance" (2026-04-29)

- **Planet pulse/breathe:** Natal planets sinusoidally modulate radius + halo intensity with per-planet phase offset (1.7 rad stagger) for organic non-synchronized breathing
- **Orbital transit trails:** 5 ghost dots per transit planet at progressively earlier angular positions with fading alpha (0.08→0.60), comet-tail effect behind drifting transits
- **Aspect line shimmer wave:** Traveling alpha pulse along each aspect line, speed varies by type (conjunction 1.0, sextile 1.5, trine 2.0, square 4.0) — red squares shimmer fastest
- **Zodiac segment glow:** Active sign (containing current Sun transit) gets 30% brightness boost + subtle 1.5Hz pulse. Computed from Sun's ecliptic longitude / 30
- **Dust mote cursor interaction:** Vignette dust motes push away from mouse cursor within 0.15 UV radius. Cursor position passed via VignetteUniforms mouse_pos field
- **Candlestick chart draw-in:** On ticker switch, candles grow from price midpoint with left-to-right stagger (60% stagger / 40% growth over 500ms). Uses ease_out_cubic per candle
- **Layered page transitions:** Background settles first ~100ms (3× alpha speed), content follows over full 300ms. Gold glow fires during fast background phase
- **Tab sparkle tuning:** 8 particles (up from 5), varied sizes 1.5-4.0px, faster burst (0.08 stagger), downward gravity drift during fade
- **60fps astrology tab:** Tick model fix — `still_animating |= active_tab == Astrology` keeps shader_time advancing at 60fps when astrology tab visible
- **Uniform buffer growth:** NatalWheel3DUniforms 496→512 bytes (active_sign + padding). VignetteUniforms field reuse (mouse_pos replaces 2 pad floats, stays 64 bytes)

**Files modified:** 12

| Feature | Before (v8.0.0) | After (v9.0.0) |
|---------|-----------------|----------------|
| Natal planets | Static gold dots | Breathing pulse (sinusoidal radius modulation) |
| Transit planets | Single dot per planet | Dot + 5-ghost orbital trail fade |
| Aspect lines | Solid colored segments | Shimmer wave (traveling alpha pulse) |
| Zodiac ring | Uniform brightness | Active sign highlighted + pulsing |
| Dust motes | Lissajous drift only | Cursor-reactive repulsion |
| Candlestick chart | Instant render | Grow-from-midpoint staggered draw-in |
| Page transitions | Flat 250ms fade | Layered 300ms (fast bg + delayed content) |
| Tab sparkle | 5 particles, fixed | 8 particles, gravity drift, varied sizes |
| Astrology tick | 30s when idle | 60fps when tab visible |

**Project stats:** ~20,800 Rust+WGSL source | 2 GPU shaders | 4 canvas ornaments | 70 tests | 0 warnings

---

## v8.0.0 — "The Observatory" (2026-04-28)

- **3D natal chart shader:** Replaced Canvas-based `NatalWheel` with GPU-rendered `NatalWheel3DProgram` using procedural SDF fragment shader
- **Perspective-tilted zodiac:** Y-axis foreshortening + slow rotation creates convincing 3D tilted disc without vertex buffers
- **12 colored zodiac segments:** Element-based sign colors (fire/earth/air/water) rendered via SDF arc regions with anti-aliased edges
- **Glowing planet dots:** Natal planets = gold halos + hot center core, transit planets = blue/red with 0.5°/sec animated drift
- **Aspect line computation in WGSL:** Natal×transit O(n²) loop computes conjunction/sextile/square/trine with correct orbs (8°/6°/8°/8°)
- **Directional lighting:** Top-bright/bottom-dark gradient simulates overhead illumination on tilted disc
- **Rim glow:** Pulsing gold shimmer on outer ring edge (0.22 intensity, 0.10 radius), sinusoidal time modulation
- **Star field:** Twinkling procedural stars outside zodiac ring using hash-based noise + sinusoidal twinkle
- **Perspective tuning:** 32% Y-foreshortening (camera_tilt=0.32) for pronounced 3D depth
- **496-byte uniform buffer:** 13 natal + 13 transit planets packed as `[[f32; 4]; 13]` arrays with longitude, retrograde flag, planet index

**Files modified:** 4 + 1 new

| Feature | Before (v7.6.0) | After (v8.0.0) |
|---------|-----------------|----------------|
| Natal chart renderer | Canvas 2D (`canvas::Program`) | GPU shader (`shader::Program`) |
| Ring perspective | Flat circle | Tilted ellipse (32% foreshortening) |
| Zodiac segments | Canvas arc paths | SDF-rendered anti-aliased arcs |
| Planet rendering | Canvas circles + text glyphs | SDF dots + glow halos (no text) |
| Aspect computation | Rust loop in `draw()` | WGSL loop in fragment shader |
| Background | Solid fill | Star field + vignette |
| Animation | Transit drift only | Drift + rotation + rim pulse + star twinkle |

---

## v7.6.0 — "The Consistency" (2026-04-28)

- **Gold sub-scrollbar styling:** Extracted `gold_scrollbar_style` helper, applied to all 15 data-table scrollables across 4 view files
- **Concordance column fix:** Universe table "Conc" column width 50→90px, "Strong Confirm" no longer truncated
- **Animated transit ring:** Transit planets drift 0.5°/sec on natal chart, driven by `shader_time` — "heavens in motion" effect
- **Canvas sparkle particles:** Replaced Unicode ✦ with canvas-rendered gold particle burst (5 dots, staggered fade-in per tab)
- **Fetch error guidance:** "Scraper not found" message now includes `cargo build --bin scraper` instruction
- **Ornament contrast (v7.5.1):** Boosted alpha on all 3 canvas ornaments for Parchment theme visibility
- **Ticker-specific empty states (v7.5.1):** 8 "for this ticker" messages now interpolate actual ticker name

**Files modified:** 12

| Feature | Before (v7.5.0) | After (v7.6.0) |
|---------|-----------------|----------------|
| Sub-scrollbars | Default gray | Gold scroller, translucent rail |
| Concordance | Truncated at 50px | Full text at 90px |
| Transit ring | Static positions | 0.5°/sec animated drift |
| Tab sparkle | Unicode ✦ character | Canvas particle burst |
| Ornaments | Low alpha (barely visible) | Boosted alpha (visible on cream) |
| Empty states | "for this ticker" | "for AAPL" / "for MSFT" |

---

## v7.5.0 — "The Polish" (2026-04-28)

- **Scrollbar styling:** Gold scroller on translucent rail, right padding prevents content overlap
- **Fetch error display:** Persistent orange warning banner for errors, pre-flight scraper check, gold loading bar
- **Gauge grid:** 5 gauges in 3+2 grid layout (two rows), no horizontal scrollbar
- **Leather vignette warmth:** `grimoire_outer_bg()` multipliers increased (0.15→0.25), shader center brightened (1.2→1.5)
- **Natal chart beautified:** Element-colored zodiac ring segments, gold glow halos on natal planets, planet glyphs, 300→400px canvas
- **Tab sparkle:** Gold ✦ character fades in during hover with delayed alpha ramp
- **Active tab visibility:** Gold-colored label always visible, 3px gold underline, surface background. Three-tier: active/hovered/default

**Files modified:** 8

| Feature | Before (v7.4.1) | After (v7.5.0) |
|---------|-----------------|----------------|
| Scrollbar | Default, overlaps | Gold scroller, right padding |
| Fetch errors | Toast only | Persistent banner + pre-check |
| Gauges | Horizontal scroll | 3+2 grid |
| Natal chart | 300px, flat | 400px, colored zodiac, glyphs |
| Active tab | Icon only, 2px | Gold label visible, 3px |

---

## v7.4.1 — "The Grimoire — Header Redesign" (2026-04-28)

- **Horizontal tab bar:** Moved 8 tabs from right-side vertical strip to horizontal bar under header ornament
- **Icon-only at rest:** Tabs show icon only, label fades in on hover via `tab_hover_progress` animation
- **Gold bottom underline:** Active tab gets 2px gold bottom border + surface background
- **Transparent button chrome:** Custom `button::Style` with `background: None` so container styling shows through
- **Layout simplification:** `row![spine, book_page]` — right-side dark strip removed entirely
- **Dead code cleanup:** `build_grimoire_tabs()` deleted (~110 lines), replaced by `build_tab_bar()`

**Files modified:** 1 (`src/dashboard/view/mod.rs`)

| Feature | Before (v7.3–7.4) | After (v7.4.1) |
|---------|-------------------|----------------|
| Tab position | Right-side vertical column | Horizontal bar, top of page |
| Tab shape | Square containers + stagger | Inline icons + gold underline |
| Layout | `row![spine, page, tabs]` | `row![spine, page]` (tabs inside page) |

---

## v7.4.0 — "The Atmosphere" (2026-04-28)

- **GPU vignette shader:** Radial darkening (lighter center, dark edges) via wgpu `Shader` widget
- **Noise grain:** Static hash-based texture, luminance-adaptive strength
- **Dust motes:** 12 procedural golden particles with Lissajous drift, frozen at idle
- **Gold edge glow:** Book border glows gold during page transitions
- **Stack compositing:** `stack![vignette_shader, padded_book]` replaces flat container
- **Power-efficient:** `shader_time` only advances during 16ms animation ticks
- **Bug fix:** Parchment vignette too dark (LCD purple distortion below RGB 0.05)
- **Bug fix:** Tab icon colors inverted in both themes

**Files modified:** 7 + 2 new (`shaders/mod.rs`, `shaders/vignette.wgsl`)

---

## v7.3.0 — "The Grimoire" (2026-04-27)

- **Right-side book tab dividers:** 8 tabs moved from horizontal top bar to right-side vertical column, styled as physical book dividers with staggered cascade
- **Hover-to-expand tabs:** `mouse_area` hover detection — icon-only (48px) expands to icon+label (168px) on hover with `ease_out_back` elastic overshoot animation
- **Dark atmospheric outer frame:** Deep circadian-aware background behind book (grimoire_outer_bg)
- **Book spine:** Canvas-rendered vertical binding strip with cross-stitch marks and diamond endcaps
- **Page header ornament:** Canvas Renaissance-style flourish with central lozenge, sine-wave scrollwork, extending rules
- **Page border corners:** Canvas decorative corner brackets with perpendicular arms and gold diamond vertices
- **Page transition:** 250ms "materializing from darkness" fade-in when switching tabs
- **Compact navigation:** Merged header + nav into single slim row, reduced chrome
- **New easing:** `ease_out_back` (elastic overshoot) for playful game-feel interactions

**Files modified:** 9 + 1 new (`ornaments.rs`)

| Feature | Before (v7.2) | After (v7.3) |
|---------|---------------|--------------|
| Tab position | Horizontal top bar | Right-side vertical dividers |
| Tab hover | None | Icon→icon+label expand animation |
| Layout | column![header, tabs, content] | row![spine, book_page, grimoire_tabs] |
| Outer frame | None | Dark atmospheric background |
| Decorations | None | Canvas spine, header ornament, corner brackets |
| Tab switch | Instant | 250ms page transition fade |

---

## v7.2.0 — "The Motion" (2026-04-27)

- **Phosphor Icons:** Replaced Bootstrap Icons with Phosphor (1530 icons, regular + bold weights)
- **Animation infrastructure:** Easing functions, adaptive tick (16ms/60fps during animation, 30s at rest)
- **Gauge sweep:** Fear/Greed needle sweeps old→new score over 600ms (ease_out_cubic)
- **Toast fade-out:** Opacity fades 1.0→0.0 over last 500ms of lifetime
- **Tab indicator crossfade:** Gold underline fades between tabs over 200ms
- **Responsive font scaling:** Viewport-aware auto-scale (<1024px: 0.85, 1440+: 1.05, 1920+: 1.1)
- **Bug fix:** TECHNICAL INDICATORS vertical text wrapping (6→2×3 grid layout)
- **Bug fix:** Recently-viewed overflow (capped at 6)
- **Version control:** Cargo.toml synced to actual version, git tags created, CHANGELOG.md added

**Files modified:** 13 + 3 new

---

## v7.1.0 — "The Ledger — Spatial Polish" (2026-04-27)

- Fix paper trail buy threshold text 75 → 65
- Spacing constants (`SPACE_XS/SM/MD/LG/XL`, `MAX_WIDTH`, `RADIUS_CARD`) + layout primitives (`max_container`, `eyebrow`, `section_rule`)
- Compact 2-row header (~200px → ~80px), remove status text
- 1240px max-width centered container
- 38 eyebrow labels + ~30 section rules across all 8 tabs
- ~65 `.font(font::INTER)` on numeric values across 7 files
- Overview restructure — vertical flow, full-width 300px hero chart

**Files modified:** 11 (`theme.rs`, `shared.rs`, `mod.rs`, + 8 view files)

---

## v7.0.0 — "The Ledger" (2026-04-26)

- LedgerPalette engine: 11-channel semantic palette with RwLock cache
- 8 anchor palettes (4 Parchment + 4 Leather) with 24-stage circadian lerp
- ThemeMode: Auto/Parchment/Leather (removed TokyoNight)
- Circadian preview slider in Settings (0-23 hour override)
- Four-role typography: Fraunces (display), Source Serif 4 (body), Inter (numerics), JetBrains Mono (tabular)
- Shared component restyling: card borders, gold tab indicator, toast overlay
- 43+ heading instances updated to `.font(font::DISPLAY)`

---

## v6.2.0 — "The Priority Queue" (2026-04-26)

- `collect_priority_tickers()` merges paper positions into fetch pipeline
- All Phase 2 data sources (sentiment, Finnhub, short, EDGAR) cover paper tickers
- Tiingo bulk SQL includes paper_portfolio at tier-0 priority

---

## v6.1.0 — "The Benchmark" (2026-04-26)

- SPY benchmark comparison (Sharpe, max drawdown, alpha)
- NYSE holiday calendar (2022-2030) — no trades on market holidays
- 25% position cap rebalancing
- 15% trailing stop-loss exits
- Win rate, avg holding days, closed trade statistics

---

## v6.0.0 — Paper Trading Engine (2026-04-26)

- Paper trading account with $100K initial capital
- Buy when Lagrange score > threshold, sell when < 35
- Equity curve chart with daily portfolio valuation
- Trade log, open positions, performance statistics
- Paper Trail tab in dashboard

---

## v5.0.1 — Polish (2026-04-25)

- Replace `.unwrap()` with safe extraction in SetAgentMode handler
- UTF-8 safe error truncation in LLM API path
- `fetch_single_ticker` now returns Result for proper exit codes
- Reset notification flag on mark-all-read
- Replace raw Color values with theme constants
- Log portfolio import errors instead of silently discarding

---

## v5.0.0 — "The Council" (2026-04-25)

- Eliminated all 47 compiler warnings
- "Fetch this ticker" button — dashboard spawns scraper subprocess with `--ticker` CLI mode
- LLM-backed agent analysis via Anthropic Claude API with Template/LLM mode toggle

---

## v4.2.0 — "The Expansion" (2026-04-24)

- OHLC candlestick charts replacing area fill
- Black-Scholes Options Greeks calculator + IV solver
- Server-side sortable Universe table (6 columns)
- In-app toast notifications
- GDELT geopolitical events in Research tab

---

## v4.1.0 — "The Glass" (2026-04-24)

- Catppuccin Mocha/Latte/Tokyo Night theme system
- Bootstrap Icons integration
- Card-based layout with section headings
- Fear & Greed gauge widgets
- Keyboard shortcuts (Ctrl+1..7 tabs, Ctrl+T search, Ctrl+R refresh)

---

## v4.0.0 — "The Forge" (2026-04-23)

- Modular update dispatcher (5 domain files)
- Extracted helpers, db modules, view modules
- Removed dead code, fixed clippy warnings
- Moshier ephemeris fix (NaN and state corruption)

---

## v3.1.x — "The Network" (2026-04-22 to 2026-04-23)

- Strategy builder, backtesting, transaction log
- Named watchlists, portfolio P&L
- Concordance detection, extended aspects
- Font scale setting, astro priority scrape
- Polymarket prediction markets integration
- RSS news aggregation from 25 sources
- DBnomics international economics scraper

---

## v3.0.x — Bug fixes and UX (2026-04-22)

- 6 bug fixes + 4 UX improvements
- Astrology engine (Swiss Ephemeris, natal charts, transits, horoscopes)
- Lagrange composite scoring system
- Universe Explorer with 1,700+ tickers
- Insider trades, filings, holdings, earnings, sentiment
- DCF intrinsic value calculator

---

## v1.0.0 — v1.1.0 (2026-04-22)

- Enrichment pipeline, Tiingo integration, alerts, recently-viewed
- Theme color tokens, chart/sparkline theming
- Type scale, section hierarchy, table spacing

---

## v0.6.0 — v0.7.0 (2026-04-08 to 2026-04-16)

- Lagrange history sparkline, portfolio tracker, CPI YoY%
- Lagrange Score, expanded data sources, signal intelligence

---

## v0.1.0 — Scaffold (2026-04-07 to 2026-04-08)

- Scaffold two-binary financial dashboard (scraper + Iced GUI)

---

## Project Stats (v8.0.0)

| Metric | Value |
|--------|-------|
| Commits | 50+ |
| Rust source | ~20,500 lines across 2 binaries |
| SQL migrations | 32 |
| Tests | 70 (48 lib + 17 dashboard + 5 scraper) |
| Compiler warnings | 0 |
| Crate deps | 26 |
| Font assets | ~2.7MB (Fraunces, Source Serif 4, Inter, JetBrains Mono, Phosphor, Phosphor Bold) |
| GPU shaders | 2 (vignette.wgsl, natal_wheel_3d.wgsl) |
| Canvas widgets | 4 (BookSpine, PageHeaderOrnament, PageBorderCorner, TabSparkle) |
| Git tags | 9 (v4.0.0 - v7.3.0) |
| Development | 22 days (Apr 7 - Apr 28, 2026) |