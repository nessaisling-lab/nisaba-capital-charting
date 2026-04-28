# TODOS

## Open

### docker-compose.yml for local PostgreSQL
**What:** `docker-compose.yml` to start PostgreSQL. 10-line file, `POSTGRES_PASSWORD=dev`, port 5432.
**Blocked by:** User wants to explore Docker first.

### Data sparsity in Lagrange sub-scores
**What:** All 4 sub-scores (Astro 40%, Fin 25%, Macro 20%, Short 15%) are computed, but sentiment and short-interest data are sparse. Most tickers default those inputs to neutral 50, compressing scores into the 45-73 range.
**Fix paths:** (a) Add alternative sentiment source (Finnhub, RSS tone), (b) find free short-interest feed, (c) buy threshold already lowered 75→65 as interim.
**Impact:** Paper engine now trades with 65 threshold; raising back to 75 once data coverage improves.

### "Fetch this ticker" button
**What:** Single-ticker on-demand scraper fetch from the dashboard UI.
**Blocked by:** Needs scraper-side work (currently batch-only pipeline).

## Completed (v6.0-v7.1)
- v7.1: Spatial polish — compact header, 1240px max-width, eyebrow labels, section rules, Inter font on numerics, vertical overview layout
- v7.0: Renaissance book UI/UX overhaul (Parchment/Leather themes, 24-stage circadian, Fraunces/Source Serif typography)
- v6.2: Paper engine priority pipeline (all data sources cover paper tickers)
- v6.1: Equity curve, NYSE holidays, rebalancing, stop-losses
- v6.0: Paper trading engine (Lagrange signal-driven simulation)
