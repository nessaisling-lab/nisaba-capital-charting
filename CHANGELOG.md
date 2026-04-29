# Changelog — Pursuit Week 4 Financial Dashboard

**Author:** Aisling Leiva
**Stack:** Rust, Iced 0.13, SQLx, PostgreSQL
**Development:** 2026-04-07 to 2026-04-29

---

## v9.0.0 — "The Performance" (2026-04-29)

- **Planet pulse/breathe:** Natal planets sinusoidally modulate radius + halo intensity with per-planet phase offset (1.7 rad stagger) for organic non-synchronized breathing
- **Orbital transit trails:** 5 ghost dots per transit planet at progressively earlier angular positions with fading alpha (0.08→0.60), creating comet-tail effect behind drifting transits
- **Aspect line shimmer wave:** Traveling alpha pulse along each aspect line, speed varies by type (conjunction 1.0, sextile 1.5, trine 2.0, square 4.0) — red squares shimmer fastest
- **Zodiac segment glow:** Active sign (containing current Sun transit) gets 30% brightness boost + subtle 1.5Hz pulse. Computed from Sun's ecliptic longitude / 30
- **Dust mote cursor interaction:** Vignette dust motes gently push away from mouse cursor within 0.15 UV radius. Cursor position passed via VignetteUniforms mouse_pos field
- **Candlestick chart draw-in:** On ticker switch, candles grow from price midpoint with left-to-right stagger (60% stagger / 40% growth over 500ms). Uses ease_out_cubic per candle
- **Layered page transitions:** Background settles in first ~100ms (3× alpha speed), content follows over full 300ms. Gold glow fires during fast background phase
- **Tab sparkle tuning:** 8 particles (up from 5), varied sizes 1.5-4.0px, faster burst (0.08 stagger), downward gravity drift during fade
- **60fps astrology tab:** Tick model fix — `still_animating |= active_tab == Astrology` keeps shader_time advancing at 60fps when astrology tab is visible
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
- **Directional lighting:** Top-bright/bottom-dark gradient simulates overhead illumination on the tilted disc
- **Rim glow:** Pulsing gold shimmer on outer ring edge (0.22 intensity, 0.10 radius), sinusoidal time modulation
- **Star field:** Twinkling procedural stars outside the zodiac ring using hash-based noise + sinusoidal twinkle
- **Perspective tuning:** 32% Y-foreshortening (camera_tilt=0.32) for pronounced 3D depth
- **496-byte uniform buffer:** 13 natal + 13 transit planets packed as `[[f32; 4]; 13]` arrays with longitude, retrograde flag, and planet index

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
- **Dark atmospheric outer frame:** Deep circadian-aware background behind the book (grimoire_outer_bg)
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
- **Gauge sweep:** Fear/Greed needle sweeps from old→new score over 600ms (ease_out_cubic)
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
