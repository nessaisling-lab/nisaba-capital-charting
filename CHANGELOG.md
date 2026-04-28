# Changelog — Pursuit Week 4 Financial Dashboard

**Author:** Aisling Leiva
**Stack:** Rust, Iced 0.13, SQLx, PostgreSQL
**Development:** 2026-04-07 to 2026-04-27

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

## Project Stats (v7.3.0)

| Metric | Value |
|--------|-------|
| Commits | 42+ |
| Rust source | ~19,200 lines across 2 binaries |
| SQL migrations | 32 |
| Tests | 70 (48 lib + 17 dashboard + 5 scraper) |
| Compiler warnings | 0 |
| Crate deps | 26 |
| Font assets | ~2.7MB (Fraunces, Source Serif 4, Inter, JetBrains Mono, Phosphor, Phosphor Bold) |
| Git tags | 9 (v4.0.0 - v7.3.0) |
| Development | 20 days (Apr 7 - Apr 27, 2026) |
