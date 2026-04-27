# TODOS

## Open

### docker-compose.yml for local PostgreSQL
**What:** `docker-compose.yml` to start PostgreSQL. 10-line file, `POSTGRES_PASSWORD=dev`, port 5432.
**Blocked by:** User wants to explore Docker first.

### Sub-score computation (Fin/Macro/Short/Concordance)
**What:** Lagrange composite currently equals astro score. Fin, Macro, Short, and Concordance sub-scores are never computed.
**Impact:** Universe Explorer, alerts, and paper engine all score on astrology alone.

### "Fetch this ticker" button
**What:** Single-ticker on-demand scraper fetch from the dashboard UI.
**Blocked by:** Needs scraper-side work (currently batch-only pipeline).

## Completed (v6.0-v7.0)
- v7.0: Renaissance book UI/UX overhaul (Parchment/Leather themes, 24-stage circadian, Fraunces/Source Serif typography)
- v6.2: Paper engine priority pipeline (all data sources cover paper tickers)
- v6.1: Equity curve, NYSE holidays, rebalancing, stop-losses
- v6.0: Paper trading engine (Lagrange signal-driven simulation)
