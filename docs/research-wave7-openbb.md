# Wave 7 + 8 Research: Pure-Rust OpenBB Alternative

**Date:** 2026-05-04 (revised from earlier Python-sidecar plan)
**Path chosen:** Path C — Wave 7 ships 10 native Rust providers; Wave 8 is conditional Rust sidecar mimicking OpenBB Workspace contract.

---

## Decision Trail

### Earlier proposal (rejected)
First Wave 7 plan was to install `pip install openbb`, run `openbb-api` as Python sidecar, and have Rust scraper call `localhost:6900`. Pros: 350+ datasets immediately. Cons: Python operational footprint, two-language maintenance, ~2GB pip deps, off-brand for pure-Rust project.

### User feedback
"Lean toward 10 providers but want to know what's worth picking" → analyzed gap between OpenBB's 50-60 unique providers and our existing 20-module scraper. **Real gap is ~10 providers**, not 350.

"Want to talk through Approach 2 more" → analyzed Rust sidecar that mimics OpenBB Workspace's HTTP contract. Workspace is just an HTTP client; doesn't care if backend is Python OpenBB or Rust axum.

### Final decision (Path C)
Two phases. Phase 1 ships data. Phase 2 (conditional) ships UI access via Workspace.
- **Wave 7** — 10 native Rust providers. Pure additive, no Python, ~8-10 days.
- **Wave 8** — Rust axum sidecar speaking OpenBB Workspace contract. Conditional. ~7 days.

---

## Why Pure-Rust Provider Ports Beat OpenBB Python For This Project

**OpenBB's "350+ datasets" is misleading**: 350 = providers × endpoints. Real provider count is ~50-60. Our existing scraper already covers ~25 of those (FRED, FMP, Finnhub, AV, Yahoo, Stooq, Tiingo, EDGAR, GDELT, Polymarket, RSS, Wikidata, etc.). True gap is ~10 providers.

**Per-provider Rust port cost: ~half-day to 1.5 days.** 10 providers = ~8-10 days. Same effort as installing/learning OpenBB Python platform, but stays in Rust.

**Architectural fit**: existing `src/scraper/sources/` (Wave 6.A1/A2) already established the pattern. New providers slot in directly. No new operational concepts.

**Quality control**: each provider isolated. We control rate limits, retries, schema mapping, error handling. OpenBB community-maintained adapters have variable quality.

---

## Wave 7: The 10 Providers

### Selection criteria
1. **Free** (no paid API key required)
2. **Closes a real gap** not in our existing 20 modules
3. **Plausibly affects scoring** (Lagrange composite or astro signals)
4. **Low integration effort** (public docs, simple JSON/CSV)

### Provider matrix

| # | Provider | Domain | Endpoint | Effort | Wave |
|---|----------|--------|----------|--------|------|
| 1 | **World Bank** | Intl macro (200+ countries, 1500+ indicators) | `api.worldbank.org/v2/...` | 1 day | 7.0 |
| 2 | **IMF DataMapper** | Sovereign macro, IMF forecasts | `imf.org/external/datamapper/api/v1/` | 1 day | 7.0 |
| 3 | **ECB Statistical Warehouse** | EU monetary policy, EUR FX | `data-api.ecb.europa.eu/service/data/` | 1.5 days | 7.0 |
| 4 | **CFTC Commitment of Traders** | Futures positioning sentiment | weekly CSV from `cftc.gov` | 1.5 days | 7.1 |
| 5 | **BLS** | Detailed US labor (CPI components, sector employment) | `api.bls.gov/publicAPI/v2/` | 1 day | 7.2 |
| 6 | **EIA** | US energy data (oil, gas, electricity) | `api.eia.gov/v2/` | 1 day | 7.2 |
| 7 | **OFR Financial Stress** | Single composite stress index | `financialresearch.gov/data/` | 0.5 day | 7.3 |
| 8 | **Treasury Direct** | US Treasury auctions, yield curve | `api.fiscaldata.treasury.gov/` | 1 day | 7.4 |
| 9 | **CoinGecko** | Crypto prices/market data (30/min free) | `api.coingecko.com/api/v3/` | 0.5 day | 7.4 |
| 10 | **(buffer)** | TBD based on need surfaced from prior 9 | — | 1 day | — |

**Total: ~9-10 days** including dashboard surfacing + cross-check work.

### Why these 10 specifically

**Skipped (and why):**
- **OECD Stats** — overlap with World Bank, more complex schema. Deferred.
- **Eurostat** — overlap with ECB for our needs. Deferred.
- **DefiLlama** — only matters when DeFi tickers added. Out of current scope.
- **NASA Eclipse computational** — current 17-eclipse hardcoded catalog (Wave 6.B4) covers 2025-2028. Sufficient.
- **JPL Horizons** — Swiss Eph Moshier already sub-arcminute accurate for our orbs.
- **NOAA weather** — diminishing returns vs core financial signal.
- **TradingEconomics** — overlap with FRED + planned cross-check uses native FRED only.

**Top picks rationale:**
- World Bank/IMF/ECB = international macro coverage we entirely lack. FRED is US-only.
- CFTC = qualitatively new signal type (smart-money positioning).
- BLS = sector-level labor breakdowns enable sector-aware Lagrange tweaks.
- EIA = energy/utility tickers currently scored without sector-specific signal.
- OFR = single high-signal composite. Best signal-per-LOC ratio.
- Treasury + CoinGecko = round out coverage.

### Implementation pattern (per provider)

Each provider follows existing scraper module structure:

```rust
// src/scraper/{provider}.rs
use anyhow::{Context, Result};
use std::sync::Arc;

pub async fn fetch_all(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
) {
    // 1. Determine what to fetch (priority list, staleness check)
    // 2. Loop with rate-limit-respecting sleeps
    // 3. Per-ticker/series: HTTP GET, parse, upsert
    // 4. Log to fetch_log
}

pub async fn fetch_one(
    pool: Arc<sqlx::PgPool>,
    client: Arc<reqwest::Client>,
    ticker_or_series: &str,
) -> Result<()> {
    // Single-fetch variant for FetchThisTicker flow
}
```

Same pattern as `analyst_targets.rs` (Wave 6.A3), `world_bank.rs` mirrors that closely.

### Data architecture additions

Migrations 0043-0047 add 5 new tables:

```sql
-- 0043_intl_macro.sql
CREATE TABLE intl_macro_indicators (
    country_code  TEXT NOT NULL,
    indicator_code TEXT NOT NULL,
    indicator_name TEXT,
    date          DATE NOT NULL,
    value         DOUBLE PRECISION,
    data_source   TEXT NOT NULL,  -- 'worldbank' | 'imf' | 'ecb'
    PRIMARY KEY (country_code, indicator_code, date, data_source)
);

-- 0044_cftc_cot.sql
CREATE TABLE cftc_positioning (
    commodity        TEXT NOT NULL,
    report_date      DATE NOT NULL,
    commercial_long  BIGINT,
    commercial_short BIGINT,
    large_spec_long  BIGINT,
    large_spec_short BIGINT,
    small_spec_long  BIGINT,
    small_spec_short BIGINT,
    PRIMARY KEY (commodity, report_date)
);

-- 0045_bls_eia.sql
CREATE TABLE bls_series (
    series_id   TEXT NOT NULL,
    period_date DATE NOT NULL,
    value       DOUBLE PRECISION,
    PRIMARY KEY (series_id, period_date)
);
CREATE TABLE eia_series (
    series_id   TEXT NOT NULL,
    period_date DATE NOT NULL,
    value       DOUBLE PRECISION,
    units       TEXT,
    PRIMARY KEY (series_id, period_date)
);

-- 0046_ofr_stress.sql
CREATE TABLE ofr_financial_stress (
    fetch_date    DATE NOT NULL PRIMARY KEY,
    stress_index  DOUBLE PRECISION,
    components    JSONB
);

-- 0047_treasury_crypto.sql
CREATE TABLE treasury_yields (
    auction_date  DATE NOT NULL,
    maturity      TEXT NOT NULL,
    yield         DOUBLE PRECISION,
    PRIMARY KEY (auction_date, maturity)
);
CREATE TABLE crypto_prices (
    coin_id     TEXT NOT NULL,
    date        DATE NOT NULL,
    usd_price   DOUBLE PRECISION,
    market_cap  BIGINT,
    volume_24h  BIGINT,
    PRIMARY KEY (coin_id, date)
);
```

---

## Wave 8: The Rust Sidecar (Conditional)

### Goal
Build axum HTTP server in Rust that speaks OpenBB Workspace's API contract. Workspace cloud UI connects to it via ngrok tunnel. Gives us:
- Polished dashboards (Workspace's UI handles charts/tables/filters)
- Shareable URLs for Pursuit demo day
- Multi-user collaboration (Workspace teams)
- Mobile-responsive UI free
- PDF/CSV export free

**Critically**: this is read-only. Sidecar reads from Postgres (populated by main scraper) and serves OpenBB-compatible JSON. Doesn't fetch external APIs, doesn't write to DB.

### Architecture

```
┌─────────────────────────────────────────────────────┐
│   Pursuit Dashboard (Rust/Iced) — UNCHANGED         │
│   Existing 20 + Wave 7's 10 = 30 native scrapers    │
└─────────────────────────────────────────────────────┘
                       ↓ writes to
              ┌────────────────────┐
              │  PostgreSQL        │
              │  (shared)          │
              └────────────────────┘
                       ↑ reads (read-only role)
┌─────────────────────────────────────────────────────┐
│   Wave 8: Rust sidecar (axum, NEW binary)           │
│   src/workspace/main.rs                             │
│   Serves /widgets.json + /apps.json + per-widget    │
│   data endpoints. CORS allow pro.openbb.co.         │
│   PAT auth via Authorization: Bearer header.        │
│   Listens on localhost:7100.                        │
└─────────────────────────────────────────────────────┘
                       ↑ HTTPS
              ┌────────────────────┐
              │      ngrok         │
              │ public tunnel URL  │
              └────────────────────┘
                       ↑
              ┌────────────────────┐
              │ OpenBB Workspace   │
              │  pro.openbb.co     │
              └────────────────────┘
```

### Tech stack

| Component | Library | Already in tree? |
|-----------|---------|------------------|
| HTTP server | **axum** | No (add) |
| Tower middleware | **tower-http** | No (add) |
| DB access | **sqlx** | Yes |
| JSON | **serde / serde_json** | Yes |
| Async runtime | **tokio** | Yes |
| OpenAPI (optional) | **utoipa** | No (add if needed) |

3-4 new deps. Cargo.toml gets a new `[[bin]]` target:
```toml
[[bin]]
name = "workspace"
path = "src/workspace/main.rs"
```

Run: `cargo run --bin workspace`. Doesn't affect dashboard or scraper builds.

### OpenBB Workspace contract

Workspace expects:
- `GET /widgets.json` → catalog of available widgets
- `GET /apps.json` → app catalog (groups widgets)
- `GET /<widget-endpoint>?<params>` → array of objects matching widget's column definitions
- Bearer auth via `Authorization: Bearer <PAT>` header
- CORS allow `https://pro.openbb.co`

Widget catalog example:
```json
{
  "widgets": [{
    "name": "Lagrange Composite Score",
    "category": "Pursuit",
    "endpoint": "/lagrange-scores",
    "type": "table",
    "params": [{"paramName": "ticker", "type": "string", "default": "AAPL"}],
    "data": {
      "columns": [
        {"field": "ticker", "headerName": "Ticker"},
        {"field": "score_date", "headerName": "Date", "cellDataType": "date"},
        {"field": "score", "headerName": "Lagrange", "cellDataType": "number"},
        {"field": "label", "headerName": "Zone"}
      ]
    }
  }]
}
```

Per-widget data endpoint returns:
```json
[
  {"ticker": "AAPL", "score_date": "2026-05-04", "score": 67.2, "label": "Favorable"},
  ...
]
```

### Sub-wave breakdown

**8.0 — Scaffold (1 day)**
- Cargo `[[bin]]` workspace target
- axum on localhost:7100
- CORS allow Workspace origin
- PAT bearer auth middleware (env var `WORKSPACE_PAT`)
- Read-only Postgres role + sqlx pool

**8.1 — Contract (1 day)**
- `GET /widgets.json` returning catalog
- `GET /apps.json`
- First widget: `GET /lagrange-scores?ticker=X`
- End-to-end Workspace render verification

**8.2 — Widget set (2 days)**
- `/aspect-patterns?ticker=X`
- `/eclipse-activations?ticker=X`
- `/fixed-stars?date=D`
- `/universe?zone=X&sector=Y`
- `/data-freshness?ticker=X`
- `/lagrange-history?ticker=X`

**8.3 — Visuals (1 day)**
- Line chart, heatmap, gantt-style timeline widgets

**8.4 — Tunnel (1 day)**
- ngrok install + auth + tunnel
- Workspace account + PAT
- Header `ngrok-skip-browser-warning: true`
- Connection verification

**8.5 — Dashboard (1 day)**
- Build polished Workspace view
- Screenshot for portfolio
- Document setup

**Total: ~7 days.**

### Decision gate (when to actually start Wave 8)

After Wave 7 ships, evaluate:
1. **Is the new data signal-bearing?** Do Wave 7's 10 providers actually shift Lagrange scores in interesting ways? Run backtest comparison vs v11.4 baseline.
2. **Does Pursuit Fellowship require shareable dashboards?** Demo day, mentor presentations, portfolio links.
3. **Is local + ngrok acceptable for the use case?** Or do we need cloud-hosted always-on?

If yes to all three → ship Wave 8.
If not signal-bearing → Wave 7 might need refinement before Wave 8.
If no demo need → defer Wave 8 indefinitely.

### Risks (Wave 8)

- **Workspace contract is partially undocumented** — `widgets.json` shape published at high level but edge cases (chart types, custom renderers) need browser dev-tools observation. Budget extra time for iteration.
- **OpenBB could change Workspace's frontend contract** — they'd alienate every custom-backend user, so probably stable. Not guaranteed.
- **ngrok free tier has rotating URL** — fine for dev. Static domain costs $10/mo.
- **Eventually need cloud hosting for 24/7 access** — Fly.io free tier or Railway $5/mo would suffice. Defer until needed.

### Alternative: skip Wave 8, build Iced-native polished views instead

If Workspace integration risk seems too high, alternative is to invest the same ~7 days in making the Iced dashboard's research views polished enough to screenshot directly. Trade-offs:
- Iced screenshots vs Workspace shareable URLs (links beat static images for sharing)
- Already-installed Iced views vs new Workspace dependency
- Custom design control vs Workspace's pre-built patterns

Defer this trade-off until Wave 7 ships.

---

## Implementation Order

1. **Wave 7.0** — World Bank first (closest schema parallel to existing FRED). ~1 day. Locks in pattern.
2. **Wave 7.0 cont.** — IMF + ECB. ~2 days. Validates pattern scales.
3. **Wave 7.1** — CFTC COT. ~1.5 days. Tests CSV-source variant.
4. **Wave 7.2** — BLS + EIA. ~2 days.
5. **Wave 7.3** — OFR. ~0.5 day.
6. **Wave 7.4** — Treasury + CoinGecko. ~1.5 days.
7. **Wave 7.5** (buffer) — buffer slot for whichever provider surfaces as needed during build.
8. **Decision gate** — backtest comparison + Pursuit needs review.
9. **Wave 8** (conditional) — full sidecar + Workspace integration if gate passes.

---

## Won't Do

- ❌ Install OpenBB Python package or run `openbb-api`
- ❌ Replace existing scrapers with OpenBB equivalents
- ❌ Embed Workspace inside Iced dashboard
- ❌ Use OpenBB for time-sensitive data (not relevant — we don't depend on OpenBB)
- ❌ Add paid-tier providers in Wave 7 (those stay in API key backlog)

## Open Questions

1. **What buffer provider to slot at #10?** Decide during Wave 7.0/7.1 based on what gaps surface.
2. **Should Wave 8 deployment be local-only (ngrok) or cloud (Fly.io)?** Defer until Wave 8 actually starts.
3. **Privacy review for Workspace** — Lagrange scores are derivative work, fine to expose. Astro patterns + eclipses are derivative public ephemeris, fine. Anything from FMP/Finnhub commercial license? Audit before exposing via Wave 8.
4. **Can Workspace handle 30-second-stale data?** Our scraper runs daily-ish. If Workspace expects real-time, manage user expectations via widget descriptions.
