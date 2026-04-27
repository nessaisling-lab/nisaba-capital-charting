---
name: v3.1.6+ Dashboard & Scraper Review
description: 6 bugs + 8 UX improvements + 7 feature additions from video review (158 frames, 8min recording) and scraper output analysis (2026-04-23)
type: project
---

## v3.1.6-v3.1.9 Backlog (2026-04-23) — COMPLETE (v3.1.6-v3.1.9 all shipped)

Found via video frame extraction (158 frames from 8-min screen recording at 1 frame/3s) + scraper stdout analysis.
Covers all 7 tabs across multiple tickers (AAPL, BMRA, CABR, TOYO, VVX, BAER) in both Light and Dark themes.
Previous v3.0.7 bugs (4-9) and UX items (1-6) were fixed in v3.1.3-v3.1.5.

---

### v3.1.6 — "Fix the Pipes" (Critical Fixes)

1. **DBnomics DNS failure** — `api.db.nomics.world` fails DNS. All 5 intl macro indicators show "---".
2. **Polymarket irrelevant markets** — Sports/politics markets displayed instead of financial (rate cuts, recession, S&P).
3. **Dead RSS feeds** — Reuters, NYT, Defense News 404/403. 3 of 60 feeds broken.
4. **Empty intl macro row** — Hide row when all 5 DBnomics values are dashes.

### v3.1.7 — "The Verification Layer"

1. **Free sector data** — Populate sector/industry from Polygon (not FMP paid).
2. **Sub-scores all "---"** — Fin/Macro/Short/Concordance never computed. Lagrange = Astro exactly.
3. **EDGAR not fetching for watchlist** — 8-K + Form 4 empty for all tickers.
4. **Agents too quick to "Insufficient Data"** — Should use price/astro/sentiment when FMP missing.

### v3.1.8 — "Polish the Glass"

1. Universe search box (1739 tickers, 35 pages, no search)
2. Backtest minimum-data guard (shows 0 trades instead of explaining why)
3. Collapsible price table (100 rows dominate Fundamentals)
4. RSS ticker relevance (generic news, not ticker-filtered)
5. Recently Viewed cap at 8
6. Lagrange sparkline with sparse data

### v3.1.9 — "New Tools" ✅ COMPLETE

1. ~~"Fetch this ticker" button~~ (deferred — needs scraper-side work)
2. ✅ Alert management UI (dismiss/ack/mark-all-read + threshold display)
3. ✅ Chart astro event overlay (retrograde stations + astro score extremes)
4. ✅ News sentiment coloring
5. ✅ Export Universe CSV
6. ✅ Portfolio import from watchlist
7. ✅ Comparative auto-suggest peers
