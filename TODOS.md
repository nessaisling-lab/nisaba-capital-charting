# TODOS

## Open — Wave 6: Data Reliability + Astro Depth (~900 lines, medium risk)

**Theme:** Pair Track A (financial data) + Track B (astrology engine) so Concordance metric strengthens on both sides. 8 items, 4 sub-waves.

### Wave 6.0 — "The Reliability" (highest impact pair) — SHIPPED 2026-05-04

- [x] **6.A1 Multi-source price fallback** — Yahoo Finance v8 chart API + Stooq CSV cascade. `data_source` column on `price_data`. AV→Yahoo→Stooq order. Migration `0038`.
- [x] **6.B1 Aspect pattern recognition** — Grand Trine, T-Square, Grand Cross, Yod, Mystic Rectangle, Stellium, Kite (7 patterns). `aspect_patterns` JSONB column on `astro_scores`. Cross-chart detection (natal+transit mix). 7/7 unit tests passing. Migration `0037`.

### Wave 6.1 — "The Precision" — SHIPPED 2026-05-04

- [x] **6.A2 Multi-source fundamentals fallback** — `SourcedFundamentals` normalized struct, FMP→Finnhub `/stock/metric?metric=all`→AV `OVERVIEW`. `data_source` column on `fundamental_metrics`. Migration `0039`.
- [x] **6.B2 Aspect strength model** — `body_weight()` table (Sun/Moon=1.5, Jup/Sat=1.3, outers=1.4, inners=1.0, nodes/Chiron=0.8), `mutual_reception_bonus()` (1.15× when bodies in each other's domiciles), `out_of_sign_modifier()` (0.75× when sign-distance mismatches aspect-distance). New `score_aspect_v2()` integrates all multipliers; orb-tightness, applying/separating, dignity already existed pre-Wave 6. 18/18 aspect tests passing.

### Wave 6.2 — "The Depth" — SHIPPED 2026-05-04

- [x] **6.A3 Analyst price targets** — Finnhub `/stock/price-target` endpoint, new `analyst_targets` table (low/median/high/n_analysts), 7-day staleness check, single-ticker + universe-wide variants. Migration `0040`. Earnings calendar already existed via existing finnhub.rs.
- [x] **6.B3 Fixed stars + Arabic Parts** — 8 stars (Regulus/Spica/Antares/Aldebaran/Sirius/Vega/Fomalhaut/Algol) with hardcoded J2000 longitudes + linear precession (~0.014°/year). 1° conjunction orb. Algol carries negative strength (-14). Arabic Parts: Fortune, Spirit, Commerce, Substance computed from ASC + Sun + Moon + Mercury. NatalChart gains `ascendant: Option<f64>` field. Both add to delta_sum pre-sigmoid. 5 new tests passing (precession, conjunction detection, orb edge, Moon skip, Algol negative, Fortune formula, transit-conjunct-Fortune).

### Wave 6.3 — "The Trust" — SHIPPED 2026-05-04

- [x] **6.A4 Data freshness UI badges** — `data_freshness` SQL view aggregating 5-source completeness (prices/fundamentals/news/sentiment/astro) per ticker. `fresh_count: Option<i32>` added to UniverseRow. Universe table badge column rendering ●●●●○ with zone-color tinting (5=optimal, 3-4=favorable, 2=neutral, ≤1=misaligned). Migration `0042`.
- [x] **6.B4 Eclipse cycles + Saros series** — Hardcoded NASA eclipse catalog 2025-2028 (17 eclipses) in both `eclipses.rs::upcoming_eclipses()` const + migration `0041` seed. Activations: natal planets within 6° of any upcoming (12mo) or past (6mo echo) eclipse. Solar negative-strength (-10), lunar (-6), tightness + time-fade scaling. Saros series number stored. 5/5 unit tests passing.

## Open — Future

### House numbers on natal chart
**What:** Add house numbers (1-12) to the 3D natal chart.
**Approach:** Iced 0.14 `pin` overlay (same as planet symbols). No shader text needed.

### Real-time fetch progress bar
**What:** Parse scraper stdout line-by-line to show real progress %. Currently time-based estimate (capped at 0.85).
**Approach:** Iced 0.14 Animation API + subscription stream from subprocess stdout.

### Harmonic charts (H4, H5, H7, H9)
**What:** Compute `(longitude × N) mod 360` derivative charts, run aspect detection on harmonic positions.
**Why:** Reveals patterns invisible in natal chart. H4 = manifestation, H7 = inspiration.
**Deferred from Wave 6** — pure math, no new dependencies, but UI surface unclear.

### Sidereal vs Tropical concordance
**What:** Run aspect detection in both Tropical (current) and Sidereal (`SE_SIDM_LAHIRI`). Surface agreement as new metric.
**Why:** Two-system confirmation = stronger signal. Disagreement = ambiguity, smaller position size.
**Deferred from Wave 6** — depends on B2 strength model first.

### Progressed charts (Secondary + Solar Arc)
**What:** "1 day = 1 year" progressions for long-horizon forecasts.
**Why:** Current 90-day forecast is purely transit-based. Progressions add slow-unfolding layer.

## Open — Infrastructure

### docker-compose.yml for local PostgreSQL
**What:** `docker-compose.yml` to start PostgreSQL. 10-line file, `POSTGRES_PASSWORD=dev`, port 5432.
**Blocked by:** User wants to explore Docker first.

### OpenBB Platform integration — PROMOTED to Wave 7 (2026-05-04)
**Decision (revised 2026-05-04):** Pure-Rust path (Path C from approach analysis). Two phases:
- **Phase 1 (Wave 7) — 10 native Rust provider scrapers**, ~8-10 days
- **Phase 2 (Wave 8, conditional) — Rust sidecar mimicking OpenBB Workspace contract**, ~7 days

No Python dependency. No `openbb` package install. We pick datasets we want, hand-write Rust HTTP wrappers in `src/scraper/sources/` matching existing pattern. Phase 2 only happens if Phase 1 data proves worth presenting via Workspace cloud UI.

See `docs/research-wave7-openbb.md` for full plan.

## Open — Wave 7: Native Rust Provider Library ("The Library")

**Theme:** Add 10 free, high-value data providers as native Rust scrapers. Cherry-pick the datasets that close real gaps in our 20-module existing scraper. 5 sub-waves, ~8-10 days total.

**Selection criteria:** (1) free, (2) closes real gap, (3) plausibly affects scoring, (4) low integration effort.

### Wave 7.0 — "The Macro Foundation" (~2-3 days)
International + sovereign macro. Closes our biggest data gap (FRED is US-only).
- [ ] **7.0.1** `src/scraper/world_bank.rs` — `https://api.worldbank.org/v2/country/{code}/indicator/{id}?format=json`. 1500+ indicators, 200+ countries.
- [ ] **7.0.2** `src/scraper/imf.rs` — `https://www.imf.org/external/datamapper/api/v1/...`. Sovereign macro + IMF forecasts.
- [ ] **7.0.3** `src/scraper/ecb.rs` — `https://data-api.ecb.europa.eu/service/data/...`. EU monetary policy + EUR FX.
- [ ] **7.0.4** Migration `0043_intl_macro.sql` — unified `intl_macro_indicators` table (country_code, indicator_code, date, value, data_source).
- [ ] **7.0.5** Wire into scraper pipeline phase 2.4b (after FRED).
- [ ] **7.0.6** Dashboard surface — extend Research tab with international macro section.

### Wave 7.1 — "The Sentiment Layer" (~2 days)
Futures positioning sentiment. Qualitatively new signal type.
- [ ] **7.1.1** `src/scraper/cftc_cot.rs` — weekly CSV download from `cftc.gov`. Parse commercial vs speculator net positions.
- [ ] **7.1.2** Migration `0044_cftc_cot.sql` — `cftc_positioning` table (commodity, report_date, commercial_net, large_spec_net, small_spec_net).
- [ ] **7.1.3** Wire into scraper pipeline phase 2.5b.
- [ ] **7.1.4** New Research tab section "Futures Positioning" — gold, oil, SPX, treasuries.

### Wave 7.2 — "The Labor + Energy" (~2 days)
US sector-level signal sources.
- [ ] **7.2.1** `src/scraper/bls.rs` — `https://api.bls.gov/publicAPI/v2/timeseries/data/`. Free tier 25 queries/day, 50 series/query. Detailed CPI components, sector employment, productivity.
- [ ] **7.2.2** `src/scraper/eia.rs` — `https://api.eia.gov/v2/`. Energy data: oil/gas prices, production, inventories. Drives energy/utility tickers.
- [ ] **7.2.3** Migration `0045_bls_eia.sql` — `bls_series` + `eia_series` tables.
- [ ] **7.2.4** Wire into pipeline + Research tab.

### Wave 7.3 — "The Stress Index" (~1 day)
Single high-signal composite metric.
- [ ] **7.3.1** `src/scraper/ofr.rs` — Office of Financial Research. Financial Stress Index, money market data. Single endpoint, single number.
- [ ] **7.3.2** Migration `0046_ofr_stress.sql`.
- [ ] **7.3.3** Surface in Universe tab as new column or in macro overview.

### Wave 7.4 — "The Treasury + Crypto" (~2 days)
Round out coverage with two more high-value providers.
- [ ] **7.4.1** `src/scraper/treasury_direct.rs` — `https://api.fiscaldata.treasury.gov/services/api/...`. US Treasury auctions, debt-to-the-penny, yield curve raw.
- [ ] **7.4.2** `src/scraper/coingecko.rs` — `https://api.coingecko.com/api/v3/...`. Crypto prices/market data, 30/min free. Optional but adds asset class.
- [ ] **7.4.3** Migration `0047_treasury_crypto.sql`.
- [ ] **7.4.4** Pipeline + display.

### Wave 7 totals
**10 providers**: World Bank, IMF, ECB, CFTC, BLS, EIA, OFR, Treasury Direct, CoinGecko, plus 1 buffer slot.
**5 migrations**: 0043-0047.
**Estimate**: 8-10 days.

---

## Open — Wave 8 (CONDITIONAL): Rust Sidecar + OpenBB Workspace ("The Showcase")

**Theme:** Build Rust HTTP server (axum) mimicking OpenBB Platform's API contract. OpenBB Workspace cloud UI connects to it as if it were OpenBB. Gives us polished dashboards + sharing for Pursuit demo day. ~7 days.

**Decision gate:** Only ship Wave 8 IF Wave 7 data is worth presenting via Workspace AND Pursuit Fellowship needs polished shareable dashboards. Validate before committing.

### Wave 8.0 — "The Scaffold" (~1 day)
- [ ] **8.0.1** Cargo workspace `[[bin]]` target for `workspace` binary at `src/workspace/main.rs`.
- [ ] **8.0.2** axum scaffold serving `localhost:7100`.
- [ ] **8.0.3** `tower-http` CORS layer allowing `https://pro.openbb.co`.
- [ ] **8.0.4** PAT bearer auth middleware (env var `WORKSPACE_PAT`).
- [ ] **8.0.5** Read-only Postgres role + sqlx connection pool.

### Wave 8.1 — "The Contract" (~1 day)
- [ ] **8.1.1** Implement `GET /widgets.json` returning widget catalog.
- [ ] **8.1.2** Implement `GET /apps.json` for app definitions.
- [ ] **8.1.3** First widget endpoint: `GET /lagrange-scores?ticker=X` returning array of `{ticker, score_date, score, label}`.
- [ ] **8.1.4** Verify Workspace can render the widget end-to-end.

### Wave 8.2 — "The Widget Set" (~2 days)
Add core widgets exposing our proprietary data.
- [ ] **8.2.1** `/aspect-patterns?ticker=X` (Wave 6.B1)
- [ ] **8.2.2** `/eclipse-activations?ticker=X` (Wave 6.B4)
- [ ] **8.2.3** `/fixed-stars?date=D` (Wave 6.B3)
- [ ] **8.2.4** `/universe?zone=X&sector=Y` (paginated universe table)
- [ ] **8.2.5** `/data-freshness?ticker=X` (Wave 6.A4)
- [ ] **8.2.6** `/lagrange-history?ticker=X` (line chart series)

### Wave 8.3 — "The Visuals" (~1 day)
Chart-type widgets, not just tables.
- [ ] **8.3.1** Lagrange-over-time line chart widget.
- [ ] **8.3.2** Sector heatmap widget.
- [ ] **8.3.3** Aspect pattern timeline gantt-style.

### Wave 8.4 — "The Tunnel" (~1 day)
- [ ] **8.4.1** ngrok install + auth token + tunnel `localhost:7100`.
- [ ] **8.4.2** OpenBB Workspace account + PAT generation.
- [ ] **8.4.3** Workspace settings → backend URL = ngrok URL → header `ngrok-skip-browser-warning: true`.
- [ ] **8.4.4** Verify all widgets load in Workspace.

### Wave 8.5 — "The Dashboard" (~1 day)
- [ ] **8.5.1** Build one polished Workspace dashboard combining 6+ widgets.
- [ ] **8.5.2** Screenshot for Pursuit portfolio.
- [ ] **8.5.3** Document in `docs/openbb-workspace-rust.md` — setup, deployment, architecture.

### Wave 8 totals
**~7 days**, ~3000 LOC new Rust, 0 new external dependencies (axum/sqlx/tokio already in tree).
**Conditional ship gate**: validate after Wave 7 ships.

## Open — API Keys Backlog (Wave 6 deferred sources)

Decision 2026-05-04: Wave 6 ships with **free-tier only** (existing AV/FMP/Finnhub/Tiingo + scraping Yahoo/Stooq). Paid keys deferred until validated need.

### Polygon.io — real-time + options
**What:** Real-time prices, options chains, intraday data, news.
**Free tier:** 5 req/min — too tight for daily scraper.
**Paid:** $29/mo Stocks Starter (unlimited 15-min delayed). $79/mo Real-time.
**Why deferred:** Would unlock options-data feature surface (currently zero options data). Reconsider when we want intraday or options chains in the dashboard.

### EODHD — fundamentals (EU + US)
**What:** Different fundamentals universe than FMP, especially strong on EU/Asia tickers.
**Free tier:** 20 req/day — fundamentals universe-wide impossible.
**Paid:** $19.99/mo (100k req/day).
**Why deferred:** FMP already covers our US watchlist. EODHD only matters when we expand to international tickers.

### Quandl/Nasdaq Data Link — alt data
**What:** High-quality macro indicators, alternative datasets, sometimes free academic datasets.
**Free tier:** 50/day — workable for low-frequency macro pulls.
**Paid:** Per-dataset pricing, varies wildly.
**Why deferred:** FRED + DBnomics already cover our macro needs. Reconsider for niche datasets (CFTC, commodity inventories).

### Marketstack — tertiary price fallback
**Free tier:** 1,000/mo — only ~33/day, not useful as primary or secondary.
**Paid:** $9.99/mo (10k/mo).
**Why deferred:** Tiingo + Yahoo + Stooq already give us 3 fallback layers. Marketstack only worth adding if those prove unreliable.

### Polygon News (separate from Polygon Stocks)
**What:** Aggregated news with full article bodies + sentiment scores.
**Why deferred:** Finnhub news + RSS scraping already cover headlines. Article-body access only matters if we add full-text NLP analysis.

## Completed (v6.0-v11.3)

- **v11.3: "The Refinement"** — All 22 video-review feedback items shipped across 5 waves:
  - Wave 1: aspect line thickness, section icons, compact horoscope, scrollbar gutter, galaxy mute
  - Wave 2: header price/H/L, search above ornament, astrology tab reorder, two-column compression
  - Wave 3: sector dropdown, column tooltips, rising sign backfill, per-ticker fetch, planet symbol overlay, hover tooltips
  - Wave 4: council template diversification, chart size enum, fetch progress bar
  - Wave 5: tooltip size setting, scraper retry helper, gauge compass-rose, parchment fiber texture
- **v11.2: "The Foundation"** — Iced 0.13→0.14 framework upgrade (13+ files, 19 breaking API changes: Pipeline trait, wgpu 27, canvas Action, widget renames, application boot). PowerShell gcc PATH fix.
- **v11.1: "The Craft"** — clickable entity links, tab glow bookmark, chart layer toggles, nav redesign
- **v11.0: "The Intelligence"** — 90-day forecast, Big Three summary, smart calculator defaults, zodiac legend, loading shimmer
- **v10.0: "The Signal"** — RSS tone sentiment, Lagrange adaptive weighting, richer agent verdicts
- **v9.3: "The Clarity"** — aspect line contrast, column widths, tab labels bold, scrollbar gutter
- **v9.2: "The Cosmos"** — galaxy background, active tab gold glow + sparkle
- **v9.1: "The Polish"** — backtest crash fix, broken icons, tooltip contrast, disable chart rotation
- **v9.0: "The Performance"** — 9 animation items: planet pulse, aspect shimmer, dust motes, candle draw-in
- **v8.0: "The Observatory"** — 3D natal chart GPU shader (procedural SDF, perspective tilt, 496-byte uniforms)
- **v7.6: "The Consistency"** — gold scrollbars, canvas sparkle, animated transit ring
- **v7.3: "The Grimoire"** — game-book UI, right-side tabs, Canvas ornaments, page transition fade
- **v7.2: "The Motion"** — Phosphor Icons, animation infrastructure, gauge sweep, responsive font scaling
- **v7.1: Spatial polish** — compact header, 1240px max-width, eyebrow labels, Inter font numerics
- **v7.0: Renaissance book UI/UX overhaul** (Parchment/Leather themes, 24-stage circadian)
- **v6.0-6.2: Paper trading engine** ($100K virtual capital, equity curve, NYSE holidays)
