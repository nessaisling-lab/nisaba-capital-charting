# TODOS

## docker-compose.yml for local PostgreSQL

**What:** Add a `docker-compose.yml` to the repo root that starts a PostgreSQL instance.
**Why:** "Install PostgreSQL" is a 30-minute Windows adventure. `docker compose up -d` is 30 seconds and reproducible.
**Pros:** Reproducible setup, no system PostgreSQL needed, works on instructor's machine for demo.
**Cons:** Requires Docker Desktop installed.
**Context:** 10-line file. Would set `POSTGRES_PASSWORD=dev`, `POSTGRES_DB=financial_dashboard`, expose port 5432. The DATABASE_URL in `.env.example` already matches this config.
**Blocked by:** Nothing — can be added any time before the demo.

## ~~Paper engine: position sizing drift (rebalancing)~~ DONE (v6.1.0)

Implemented sell-only rebalancing with 25% drift threshold. Positions exceeding target weight are trimmed; freed cash funds next cycle's buys.

## ~~Data reliability: price source upgrade + priority queue~~ DONE (v6.2.0)

Priority queue implemented: paper engine positions + Lagrange buy candidates are merged into the pipeline's priority list before Phase 2. All 6 downstream data sources (AV price, Tiingo, sentiment, Finnhub, short interest, EDGAR) now cover paper engine tickers. Tiingo SQL upgraded to tier-0 priority for paper positions. Price source upgrade (Tiingo paid tier) deferred — the priority queue alone eliminates stale-price risk for the paper engine's ~30 tickers/day needs within the free-tier budget.

## ~~Paper engine: NYSE trading calendar / holiday awareness~~ DONE (v6.1.0)

Implemented `is_nyse_holiday()` covering 9 holidays with observed-date shifting + weekend guard. 5 unit tests.
