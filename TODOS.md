# TODOS

## Open — Wave 6: Data Reliability + Astro Depth (~900 lines, medium risk)

**Theme:** Pair Track A (financial data) + Track B (astrology engine) so Concordance metric strengthens on both sides. 8 items, 4 sub-waves.

### Wave 6.0 — "The Reliability" (highest impact pair)

- [ ] **6.A1 Multi-source price fallback** — `PriceSource` trait, AV→Tiingo→Finnhub→Yahoo→Stooq cascade. `data_source` provenance column. Migration `0026`.
- [ ] **6.B1 Aspect pattern recognition** — Grand Trine, T-Square, Grand Cross, Yod, Mystic Rectangle, Stellium. `aspect_patterns` table. Migration `0030`.

### Wave 6.1 — "The Precision"

- [ ] **6.A2 Multi-source fundamentals fallback** — FMP→Finnhub `metric/all`→AV `OVERVIEW`. Migration `0027`.
- [ ] **6.B2 Aspect strength model** — orb tightness, applying/separating, body weight, essential dignity, mutual reception, out-of-sign flag.

### Wave 6.2 — "The Depth"

- [ ] **6.A3 Earnings calendar + analyst targets** — FMP `/earning_calendar` + Finnhub `/stock/price-target`. Migration `0028`.
- [ ] **6.B3 Fixed stars + Arabic Parts** — Regulus/Algol/Spica/Antares/Sirius/Vega/Aldebaran/Fomalhaut + Part of Fortune/Spirit/Commerce.

### Wave 6.3 — "The Trust"

- [ ] **6.A4 Data freshness UI badges** — `data_freshness` SQL view. Universe table badge column. Verdict confidence annotation. Migration `0029`.
- [ ] **6.B4 Eclipse cycles + lunar nodes** — `swe_sol_eclipse_when_loc` + Saros series. New `eclipses` table. Forecast tab timeline. Migration `0031`.

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

### OpenBB Platform integration
**What:** Replace some scraper modules with OpenBB Platform aggregator (wraps many sources).
**Why:** ~5 of our 20 scrapers could collapse into one OpenBB call. Trade-off: external dep vs less code.

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
