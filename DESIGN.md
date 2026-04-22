# Financial Dashboard — Design Document

**Project:** Pursuit NYC Week 4 Fellowship — Native Rust Desktop Financial Dashboard
**Stack:** Rust, Iced 0.13, SQLx, PostgreSQL
**Author:** Aisling Leiva
**Current version:** v1.0.0

---

## Changelog

### v1.0.0 — Enrichment Pipeline + Tiingo + Codebase Refactor *(completed 2026-04-20)*
**Theme:** Scale the scoring engine beyond the 10-ticker watchlist. Multi-source IPO date enrichment, bulk price history via Tiingo, DRY codebase cleanup, and critical data fixes that unlock Lagrange scores at scale.

**New scraper modules:**

- [x] `src/scraper/edgar_enrich.rs` — SEC EDGAR first-filing date enrichment
  - Fetches `company_tickers.json` (one call → full CIK lookup table)
  - For each ticker with null `ipo_date`: queries `CIK{padded}.json` for earliest 10-K / S-1 / 20-F / F-1
  - Handles paginated archive batches (`files[]` in submissions JSON)
  - **CIK deduplication cache**: `HashMap<u64, NaiveDate>` — share-class variants (ADAMG/ADAMH/ADAMI) reuse the date fetched for the primary CIK without additional API calls
  - Rate: 200ms between calls (≤5 req/sec, well under EDGAR's 10 req/sec limit)
  - Budget: 50 tickers per daily run, watchlist-first ordering

- [x] `src/scraper/wikidata_enrich.rs` — Wikidata SPARQL founding/inception dates
  - Single HTTP call fetches up to 10,000 companies with ticker + inception date
  - SPARQL query uses UNION of `wdt:P249` (direct property) and `p:P414 → pq:P249` (P414 qualifier, where 90%+ of ticker data lives) — filtered to NYSE / NASDAQ / AMEX exchange QIDs
  - POST form method (avoids URL length limits for complex SPARQL)
  - Runs once per day (guarded via `fetch_log` check)
  - First run result: 1,974 bindings → 1,064 inception dates filled

- [x] `src/scraper/fmp_enrich.rs` — Financial Modeling Prep integration
  - `seed_ticker_universe()`: calls `/v3/stock/list`, inserts all US common stocks + ETFs into `company_metadata` — gives full Robinhood / Bloomberg tradeable universe
  - `enrich_ipo_dates()`: calls `/v3/profile/{ticker}` for up to 240 tickers/day with null `ipo_date` — fills date and seeds natal chart
  - Budget tracking via `fetch_log` (250 req/day free tier; 1 for stock list + 240 for profiles)
  - Graceful 403 handling: stops enrichment loop immediately and explains paid-plan requirement

- [x] `src/scraper/company_enrich.rs` — Alpha Vantage OVERVIEW IPO date enrichment
  - Calls `function=OVERVIEW` for up to 10 tickers/day (watchlist-first)
  - Budget-aware: counts AV calls used today, leaves 1 as safety margin
  - Handles AV rate limit messages (`"Note"` / `"Information"` JSON keys)

- [x] `src/scraper/enrich_common.rs` — Shared enrichment utilities (NEW, ~44 lines)
  - `seed_one_natal_chart()` — single canonical implementation, called by all 4 enrichment modules
  - `watchlist_priority_sql()` — generates `CASE WHEN ticker IN (...) THEN 0 ELSE 1 END, ticker` ORDER BY clause
  - Eliminated ~83 lines of copy-paste duplication across `company_enrich`, `fmp_enrich`, `edgar_enrich`, `wikidata_enrich`

- [x] `src/scraper/tiingo.rs` — Tiingo bulk price history feed
  - `fetch_all_prices_tiingo()`: fetches up to 490 tickers/day (500 free tier − 10 margin)
  - Priority: watchlist first, then all tickers with natal chart but fewer than 26 price rows (Lagrange threshold)
  - Per-ticker: uses `MAX(date) + 1` as start date if rows exist, otherwise 5 years back
  - Handles 404 silently (ticker not in Tiingo), budgets via `fetch_log`
  - **This is the primary unlock for Lagrange scores** — each ticker needs 26+ rows for the financial component (35% weight)

**Data fixes:**

- [x] T. Rowe Price removed from `INSTITUTION_MAP` in `main.rs` — T. Rowe files 13F under subsidiary CIKs, not top-level CIK `0001113169`. Was causing a persistent "No 13F-HR found" error every run.
- [x] EDGAR CIK deduplication cache — AGNCL / AGNCO / AGNCP (preferred share classes) reuse AGNC's cached date instead of burning 3 additional daily API slots
- [x] Migration 0017: `ipo_date DATE` made nullable in `company_metadata` — required for the incremental enrichment pipeline where dates arrive from multiple sources over multiple days

**Enrichment pipeline order in `run_all_fetches`:**
```
1. Polygon ticker universe seed      (new listings, list_date available)
2. FMP ticker universe seed          (full Robinhood/Bloomberg universe)
3. AV price fetch                    (watchlist, 5 calls/min)
4. Tiingo bulk price history         (490 tickers/day, watchlist-priority)
5. EDGAR Form 4 + 8-K
6. 13F institutional holdings
7. Finnhub (news, earnings, ratings)
8. AV sentiment
9. FRED macro data
10. FINRA short interest
11. Polygon options flow
12. AV OVERVIEW IPO enrichment       (10/day, watchlist-first)
13. FMP profile IPO enrichment       (up to 240/day)
14. Wikidata SPARQL founding dates   (once/day, bulk)
15. SEC EDGAR first-filing dates     (50/day, watchlist-first)
16. Astrology: seed natal charts
17. Astrology: daily transits
18. Astrology: compute astro scores
19. Lagrange: compute all scores
```

**Scale after v1.0.0 first run:**
| Table | Approximate rows |
|-------|-----------------|
| `company_metadata` | ~10,000 tickers |
| `natal_positions` | ~100,000 (10 planets × tickers with ipo_date) |
| `price_data` | ~2,500 (10 watchlist × ~250 AV rows) |
| `astro_scores` | ~1,200+ (tickers with ipo_date) |
| `lagrange_history` | 7 tickers (price_data FK limit — resolved in v1.1.0) |

---

### v0.9.0 — Alert Threshold System *(completed 2026-04-18)*
**Theme:** Real-time OS toast notifications when a ticker enters Optimal or Misaligned zone; full alert history panel

**New migration 0016:**
- [x] `lagrange_alerts` table: `id SERIAL, ticker, alert_date, score, label, prev_label, alert_type, is_read, created_at`
- [x] `UNIQUE (ticker, alert_date, alert_type)` — crossing is recorded once per day per direction
- [x] `alert_type` values: `entered_optimal`, `entered_misaligned`

**Scraper changes — lagrange.rs:**
- [x] `check_alert_crossing()` runs after each daily Lagrange upsert
- [x] Compares today's label to most-recent prior row (`ORDER BY score_date DESC LIMIT 1`) — weekend-safe
- [x] Only fires on threshold entry (not exit); ignores sideways stay in same zone
- [x] `ON CONFLICT DO NOTHING` — idempotent, safe on scraper re-runs

**New model:**
- [x] `LagrangeAlert { id, ticker, alert_date, score, label, prev_label, alert_type, is_read }` in `src/models.rs`

**Dashboard — db.rs:**
- [x] `fetch_alerts()` — SELECT last 50 alerts ordered by date DESC
- [x] `mark_alert_read()` — fire-and-forget UPDATE, logs to stderr on failure

**Dashboard — state.rs:**
- [x] New fields: `alerts: Vec<LagrangeAlert>`, `unread_alert_count: usize`, `notifications_fired: bool`
- [x] New messages: `AlertsLoaded`, `MarkAlertRead(i32)`, `NotifyAlerts`

**Dashboard — update.rs:**
- [x] `fetch_alerts` added to startup batch (`TickersLoaded`) and 30-second `Tick` handler
- [x] `AlertsLoaded`: stores alerts, counts unread, fires toast once per session via `notifications_fired` spam guard
- [x] `MarkAlertRead`: optimistic in-memory flip + fire-and-forget DB write
- [x] `fire_toast()` async fn using `notify-rust` — summary + up to 3 tickers in body, "+N more" overflow

**Dashboard — view.rs:**
- [x] Alerts panel below Watchlist Ranking: date, ticker, score, zone (color-coded), was, Mark Read button
- [x] Watchlist zone score text now color-coded: Misaligned=red, Unfavorable=orange, Neutral=gray, Favorable=blue, Optimal=green
- [x] `unread_alert_count` shown in alerts panel header when > 0

---

### v0.8.0 — Universal Birth Chart Database + Dynamic Ticker Search *(completed 2026-04-18)*
**Theme:** Scale the astrology engine from 10 hardcoded tickers to the full US equity market (~10k stocks)

**New scraper module — ticker_seed.rs:**
- [x] `src/scraper/ticker_seed.rs` — paginate Polygon.io `/v3/reference/tickers?market=stocks&active=true&locale=us`
- [x] Upsert all active US common stocks into `company_metadata` using `list_date` as `ipo_date`
- [x] Exchange MIC → human-readable name mapping: `XNAS` → NASDAQ, `XNYS` → NYSE, `ARCX` → NYSE Arca, etc.
- [x] Dedup guard: Polygon may return same ticker on multiple exchanges — take first occurrence
- [x] After bulk seed, run natal chart computation for every new `company_metadata` row → `natal_positions`
- [x] `ON CONFLICT (ticker) DO NOTHING` on both tables — fully idempotent, safe to re-run
- [x] Runs once at startup if `natal_positions` count < 100 (cold start), then daily incremental

**Migration 0014:**
- [x] Add `data_source TEXT NOT NULL DEFAULT 'manual'` to `company_metadata`
- [x] Add `seeded_at TIMESTAMPTZ` to `company_metadata`

**Dashboard — dynamic ticker search:**
- [x] Replace hardcoded 10-button ticker row with a text input + search button
- [x] On submit: look up ticker in DB, display full analysis if data exists
- [x] Graceful "No data yet — run scraper for this ticker" placeholder state
- [x] Pinned watchlist row preserved below search bar (user's 10 favorites, still one-click)
- [x] Recently viewed ring (last 10 tickers, stored in `recently_viewed` local DB table)

**Migration 0015:**
- [x] `recently_viewed` table: `ticker TEXT, viewed_at TIMESTAMPTZ`

---

### v0.7.0 — Lagrange History, Portfolio, CPI YoY%, Color-Coding *(completed 2026-04-16)*
**Theme:** Daily Lagrange history accumulation, portfolio positions, display polish

- [x] `src/indicators.rs` moved to lib crate — shared between scraper and dashboard binaries
- [x] `compute_lagrange_score` now returns `(f32, String, LagrangeComponents)` with component breakdown
- [x] `LagrangeComponents { fin_score, astro_score, macro_score, short_score }` stored for debugging
- [x] `src/scraper/lagrange.rs` — daily Lagrange Score computation for all 10 tickers
- [x] Migration 0013: `lagrange_history` + `portfolio_positions` tables
- [x] `LagrangeSparkline` canvas widget: 90-day score history strip below price chart
- [x] Portfolio panel: reads `portfolio_positions` table
- [x] Macro strip: CPI raw index replaced with YoY% via SQL CTE window calculation
- [x] Short interest symbol guard fix (was summing all records regardless of ticker)

---

### v0.6.0 — Expanded Data Sources + Actionable Intelligence *(completed 2026-04-15)*
**Theme:** FRED macro data, FINRA short interest, Lagrange Score, signal synthesis

- [x] Split 1,495-line `src/scraper/main.rs` monolith into 9 focused modules
- [x] FRED macroeconomic data (10 indicators: Fed Funds, CPI, Unemployment, yields, VIX, WTI, M2)
- [x] FINRA Developer API: per-ticker short sale volume
- [x] Polygon.io: options flow put/call ratio (free-tier probe pattern)
- [x] Lagrange Score: Financial(35%) + Astrology(25%) + Macro(25%) + Short Squeeze(15%)
- [x] Signal Intelligence panel: plain-English bullet synthesis per ticker
- [x] Watchlist Ranking panel: all 10 tickers sorted by quick composite score

---

### v0.5.0 — Astrology Layer *(completed 2026-04-14)*
**Theme:** Company birth charts, planetary transit scoring, astrological Fear & Greed

- [x] Pure-Rust ephemeris (Jean Meeus formulas) — Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto
- [x] Natal chart seeder + daily transit computation + aspect scoring
- [x] Migrations 0008–0011: `company_metadata`, `natal_positions`, `daily_transits`, `astro_scores`
- [x] Dashboard: natal chart wheel canvas (inner natal ring, outer transit ring, aspect lines)
- [x] Dashboard: active transits table, moon phase display, Mercury Rx flag

---

### v0.4.0 — Module Refactor *(completed 2026-04-14)*
**Theme:** Split 1,435-line monolith into maintainable modules (10 dashboard files)

---

### v0.3.0 — UI / UX Pass *(completed 2026-04-13)*
**Theme:** Usability, layout polish, copy/open buttons, Fear & Greed gauges (3 → 5 total by v0.6.0)

---

### v0.2.0 — Data Enrichment *(completed 2026-04-12)*
**Theme:** News, earnings, analyst ratings, sentiment pipeline from Finnhub + Alpha Vantage

---

### v0.1.0 — Foundation *(completed 2026-04-10)*
**Theme:** Core scraper + dashboard skeleton — Alpha Vantage prices, EDGAR feeds, Iced UI

---

## Architecture

### Binary layout

```
pursuit_week4_automation/
├── src/
│   ├── lib.rs                        shared types (models, astrology, indicators)
│   ├── models.rs                     SQLx FromRow structs
│   ├── indicators.rs                 SMA/EMA/RSI/MACD/BB + Lagrange Score (shared lib)
│   ├── astrology/                    planetary calculation engine
│   │   ├── mod.rs
│   │   ├── ephemeris.rs
│   │   ├── aspects.rs
│   │   └── natal.rs
│   ├── scraper/
│   │   ├── main.rs                   entry point + WATCHLIST + scheduler
│   │   ├── prices.rs                 Alpha Vantage daily OHLCV (watchlist)
│   │   ├── tiingo.rs                 Tiingo bulk price history (all scored tickers)  ← NEW v1.0
│   │   ├── edgar.rs                  SEC EDGAR Form 4 + 8-K
│   │   ├── edgar_enrich.rs           SEC first-filing date enrichment                ← NEW v1.0
│   │   ├── holdings.rs               SEC EDGAR 13F institutional
│   │   ├── finnhub.rs                news, earnings, analyst ratings
│   │   ├── sentiment.rs              Alpha Vantage NEWS_SENTIMENT
│   │   ├── astrology.rs              natal seeding + transit scoring
│   │   ├── macro_data.rs             FRED macroeconomic series
│   │   ├── short_interest.rs         FINRA Developer API short volume
│   │   ├── options.rs                Polygon.io put/call ratio
│   │   ├── lagrange.rs               daily Lagrange Score computation
│   │   ├── ticker_seed.rs            Polygon ticker universe seed
│   │   ├── fmp_enrich.rs             FMP ticker universe + IPO date enrichment       ← NEW v1.0
│   │   ├── company_enrich.rs         Alpha Vantage OVERVIEW IPO enrichment
│   │   ├── wikidata_enrich.rs        Wikidata SPARQL founding dates                  ← NEW v1.0
│   │   └── enrich_common.rs          shared: seed_one_natal_chart + watchlist SQL    ← NEW v1.0
│   └── dashboard/
│       ├── main.rs                   entry point + mod declarations
│       ├── state.rs                  Dashboard struct + Message enum
│       ├── indicators.rs             local indicator helpers
│       ├── signals.rs                plain-English signal bullet generator
│       ├── helpers.rs                formatting utilities
│       ├── db.rs                     async DB + API fetch functions
│       ├── gauges.rs                 FearGreedGauge canvas widget
│       ├── charts.rs                 PriceChart canvas widget with hover
│       ├── update.rs                 update() + new() + subscription()
│       ├── view.rs                   view() layout
│       └── astrology.rs              natal wheel canvas + transits table
├── migrations/
│   ├── 0001_initial_schema.sql
│   ├── 0002_seed_watchlist.sql
│   ├── 0003_add_items_to_filings.sql
│   ├── 0004_news_articles.sql
│   ├── 0005_earnings_dates.sql
│   ├── 0006_analyst_ratings.sql
│   ├── 0007_sentiment_scores.sql
│   ├── 0008_company_metadata.sql
│   ├── 0009_natal_positions.sql
│   ├── 0010_daily_transits.sql
│   ├── 0011_astro_scores.sql
│   ├── 0012_macro_indicators.sql
│   ├── 0013_lagrange_and_portfolio.sql
│   ├── 0014_company_metadata_source.sql
│   ├── 0015_recently_viewed.sql
│   ├── 0016_lagrange_alerts.sql
│   └── 0017_nullable_ipo_date.sql
├── Cargo.toml
├── .env                              secrets (never committed)
├── .gitignore
├── CLAUDE.md
└── DESIGN.md                         this file
```

### Data flow

```
Alpha Vantage API   ──┐
SEC EDGAR API       ──┤
Finnhub API         ──┤
FRED API            ──┤
FINRA API           ──┤  scraper binary (startup + cron 6AM UTC)  ──►  PostgreSQL
Polygon.io API      ──┤
FMP API             ──┤
Tiingo API          ──┤
Wikidata SPARQL     ──┤
alternative.me      ──┘

PostgreSQL  ──►  dashboard binary (SQLx async)  ──►  Iced 0.13 UI
```

### Lagrange Score formula

```
Lagrange Score = Financial(35%) + Astrology(25%) + Macro(25%) + Short Squeeze(15%)

Financial:
  RSI(14) normalized 0-100                   × 0.30
  Price vs SMA50 momentum (±10% → 0-100)     × 0.30
  MACD histogram (±0.2% of price → 0-100)    × 0.20
  AV sentiment (-1..+1 → 0-100)              × 0.20

Macro:
  VIX score: (90 - (vix - 10) × 1.4) clamped 0-100   × 0.60
  Yield spread T10Y2Y: ((spread+1)×20+30) clamped      × 0.40

Short Squeeze:
  base: pct>30% → 75, pct>20% → 65, pct>10% → 50, else 40
  bonus: +15 if price rising AND short% > 15%

Labels: Misaligned (0-24) / Unfavorable (25-44) / Neutral (45-55) /
        Favorable (56-75) / Optimal (76-100)
```

---

## v1.1.0 Implementation Plan — Active Scoring Universe + Search Autocomplete

**Target:** 2026-04-21 or later (pending user review)

### Goal

Unlock Lagrange scoring for a meaningful universe of tickers beyond the 10-ticker watchlist — without bloating the UI or breaking any existing behavior. Simultaneously upgrade the search bar from "type and submit" to live autocomplete, so discovery of the expanded universe feels instant and natural.

This is the combination of **Option B** (configurable active scoring universe, ~50–100 tickers) and **Option D** (search autocomplete from `company_metadata`).

### The core architectural problem to solve

`price_data.ticker` currently has a **foreign key to `tickers(ticker)`**. The `tickers` table has only 10 rows (the pinned watchlist). This means:
- Tiingo cannot insert price data for any non-watchlist ticker — Postgres rejects with FK violation
- Lagrange currently loops over `WATCHLIST` const — hardcoded to 10 tickers even if data existed

The fix is clean: **drop the FK constraint**. The `tickers` table becomes the UI concept ("pinned watchlist buttons"), not a data integrity gate. `company_metadata` is the canonical universe of valid tickers and serves as the implicit source of truth for scraper inserts.

### Apple HIG Principles Applied

**Clarity — show the user what they have, not what they might have.**
- The watchlist buttons at the top are the user's "Dock" — fast-access to 10 curated tickers. They never move.
- The ranking panel shows the scored universe with clear visual hierarchy: rank, ticker, score zone (color + label), supporting signals.
- Search suggestions show company name alongside ticker symbol — "AAPL — Apple Inc." — never just a bare symbol.

**Progressive Disclosure — reveal complexity only when requested.**
- Typing 1 character shows up to 8 autocomplete suggestions. Not the full 10,000.
- The ranked universe panel shows the top 20 by default. A "Show more" row expands to the full set.
- Score details (component breakdown) live in the per-ticker drill-down, not the ranking row.

**Feedback — the UI should feel alive.**
- Autocomplete results appear within one keystroke delay (async DB query is fast: prefix match on indexed TEXT column).
- While loading suggestions, show a subtle spinner or dim the input.
- When a ticker search lands on a ticker with no price data, show the birth chart and explain what's missing.

**Consistency — reuse existing patterns.**
- Suggestion dropdown uses the same row style as the "Recently viewed" section.
- Ranking rows use the same `FillPortion` column widths already established in the watchlist panel.
- Score zone colors are unchanged: red / orange / gray / blue / green.

**Accessibility — color is never the only signal.**
- Every color-coded zone also carries a text label (Mis / Unf / Neu / Fav / Opt).
- Rank numbers provide a secondary ordering cue independent of color.

---

### Schema Changes (v1.1.0)

**Migration 0018 — Drop price_data FK, add performance index**
```sql
-- Remove the constraint that prevented Tiingo from inserting non-watchlist rows
ALTER TABLE price_data DROP CONSTRAINT IF EXISTS price_data_ticker_fkey;

-- Composite index for all the per-ticker price queries (Tiingo, Lagrange, dashboard)
CREATE INDEX IF NOT EXISTS idx_price_data_ticker_date ON price_data (ticker, date DESC);
```

**Migration 0019 — Add scoring_active flag to tickers table**
```sql
-- Separates "pinned watchlist button" from "active scoring universe"
-- watchlist (active=true, scoring_active defaults to true for existing rows)
-- additional scoring tickers: active=false, scoring_active=true
ALTER TABLE tickers ADD COLUMN IF NOT EXISTS scoring_active BOOLEAN NOT NULL DEFAULT true;

-- Future: INSERT INTO tickers (ticker, name, scoring_active) VALUES ('NVDA', ..., true)
-- to add tickers to the scoring universe without making them pinned watchlist buttons.
-- For now, the 10 existing rows all have scoring_active = true automatically.
```

> **Note:** v1.1.0 starts with the 10 existing watchlist tickers as the scoring universe. The migration establishes the concept cleanly. v1.2.0 will add a UI panel for managing the active set.

---

### Scraper Changes (v1.1.0)

**tiingo.rs — unblock non-watchlist inserts**
After migration 0018 drops the FK, Tiingo can insert for any `company_metadata` ticker. No code change needed — the query already targets `company_metadata WHERE ipo_date IS NOT NULL`.

**lagrange.rs — expand scoring universe**
Replace the hardcoded `WATCHLIST` loop with a DB-driven query:
```rust
// Was:
for ticker in WATCHLIST { ... }

// Becomes:
let scoreable: Vec<String> = sqlx::query_scalar(
    "SELECT DISTINCT pd.ticker
     FROM price_data pd
     JOIN tickers t ON t.ticker = pd.ticker
     WHERE t.scoring_active = true
     GROUP BY pd.ticker
     HAVING COUNT(*) >= 26
     ORDER BY pd.ticker"
).fetch_all(pool.as_ref()).await.unwrap_or_default();

for ticker in &scoreable { ... }
```

This means any ticker added to `tickers` with `scoring_active = true` that accumulates 26+ days of Tiingo price data automatically enters the Lagrange scoring loop the next morning.

---

### Dashboard Changes (v1.1.0)

**Search autocomplete flow**

New state field:
```rust
autocomplete_suggestions: Vec<(String, String)>,  // (ticker, company_name)
```

New DB function in `db.rs`:
```rust
pub async fn search_tickers(pool: Arc<PgPool>, prefix: String) -> Result<Vec<(String, String)>, String> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT ticker, COALESCE(company_name, ticker) FROM company_metadata
         WHERE ticker ILIKE $1 OR company_name ILIKE $2
         ORDER BY
           CASE WHEN ticker ILIKE $1 THEN 0 ELSE 1 END,
           ticker
         LIMIT 8"
    )
    .bind(format!("{}%", prefix.to_uppercase()))
    .bind(format!("%{}%", prefix))
    .fetch_all(pool.as_ref()).await.map_err(|e| e.to_string())
}
```

New messages:
```rust
AutocompleteResults(Vec<(String, String)>),
AutocompleteSelected(String),
```

**view.rs — search bar with suggestion dropdown**

Apple HIG pattern: the suggestions appear as a column directly below the search input, showing ticker + company name. Maximum 8 rows. Tapping any row loads that ticker exactly as if the user had submitted the search.

```
[ AAPL ] [ MSFT ] [ GOOGL ] ...  ← pinned watchlist (unchanged)
Search: [ APPL________________ ] [Go]
┌─────────────────────────────┐
│ AAPL  — Apple Inc.          │  ← suggestion row (click to select)
│ AAPB  — GraniteShares 2x... │
│ AAPU  — Direxion Daily...   │
└─────────────────────────────┘
Recently viewed: NVDA · JPM · V
```

**view.rs — ranked universe panel upgrade**

The existing Watchlist Ranking panel shows the active scoring universe (not just the 10 watchlist buttons). Add a scrollable container with fixed height and a "Scored universe" header that shows total count:

```
Scored Universe — 10 tickers  [ ▼ Sort: Score ]
────────────────────────────────────────────────
#   Ticker  Score         Astro    Sentiment  Short%
1   AAPL    82 Opt ■     73 Greed  Bull (0.3)  0.7%
2   MSFT    71 Fav ■     68 Greed  Neut (0.1)  0.8%
...
```

Scrollable container fixed at 200px height. Sort toggle (by Score desc, by Ticker asc) is a button row at the header.

---

### File Changes (v1.1.0)

| File | Change |
|------|--------|
| `migrations/0018_drop_price_data_fk.sql` | New — drops FK, adds composite index |
| `migrations/0019_scoring_active.sql` | New — adds `scoring_active` column to `tickers` |
| `src/scraper/lagrange.rs` | Replace `WATCHLIST` loop with DB query on `scoring_active = true` |
| `src/dashboard/state.rs` | Add `autocomplete_suggestions: Vec<(String, String)>` |
| `src/dashboard/db.rs` | Add `search_tickers()` function |
| `src/dashboard/update.rs` | Handle `TickerSearchInput` → fire autocomplete query; handle `AutocompleteResults`, `AutocompleteSelected` |
| `src/dashboard/view.rs` | Add suggestion dropdown below search bar; upgrade ranking panel header |

**Estimated new code:** ~180 lines across 6 files.

---

### Scale Estimates (v1.1.0)

| Concern | Assessment |
|---------|-----------|
| Price table in UI | Already `LIMIT 100` in `db.rs` — safe at any scale |
| Ranking panel | Scrollable `Length::Fixed(200.0)` — handles 100+ rows fine |
| Autocomplete query | Indexed TEXT prefix match — sub-10ms even at 10k rows |
| Market breadth query | `price_data` CTEs will be heavier with 490+ tickers; `idx_price_data_ticker_date` index from migration 0018 handles it |
| Lagrange loop | DB-driven, not hardcoded — if 0 new tickers qualify, loop body never runs |
| Alert flood risk | `check_alert_crossing` returns early on first-ever score — no flood on Tiingo first run |

---

## v1.2.0 Preview — Universe Explorer (Option C Foundation)

**Target:** Future sprint (after v1.1.0 is validated)

### Goal

A dedicated "Universe Explorer" panel that gives the user a sortable, filterable view of Lagrange scores across the full population of scored tickers — potentially 500–1,000+ tickers after several weeks of Tiingo accumulation. This is the full realization of Option C.

### What v1.1.0 prepares for v1.2.0

- FK dropped: Tiingo can score any ticker
- `scoring_active` column: the concept of "active universe" exists in the DB
- Composite index on `price_data(ticker, date)`: bulk queries are fast
- Lagrange loop is DB-driven: adding tickers automatically enters them into scoring

### New features in v1.2.0

**Universe Explorer panel** (new tab or expandable section):
- Sortable columns: Ticker, Lagrange Score, Zone, Financial, Astro, Macro, Short%
- Filter controls: Zone filter (show only Optimal / Favorable), Sector filter (requires sector data in `company_metadata`)
- Pagination: 50 rows per page, page controls
- "Add to Scoring Universe" button on each row (sets `scoring_active = true`)
- Export to CSV button (for integration with other tools)

**DB query driving the Explorer:**
```sql
SELECT lh.ticker, cm.company_name, lh.score, lh.label,
       lh.fin_score, lh.astro_score, lh.macro_score, lh.short_score
FROM lagrange_history lh
JOIN company_metadata cm ON cm.ticker = lh.ticker
WHERE lh.score_date = (SELECT MAX(score_date) FROM lagrange_history WHERE ticker = lh.ticker)
ORDER BY lh.score DESC
LIMIT 50 OFFSET $1
```

**Sector data source:** FMP `/v3/profile/{ticker}` returns `sector` and `industry` fields — add to `company_metadata` during enrichment pass.

**Managing the scoring universe (v1.2.0 UI):**
```
Scoring Universe Settings
─────────────────────────
Currently scoring: 47 tickers  [ + Add tickers ]  [ Import from CSV ]
Tiingo budget: 490/day  ·  Today's usage: 312/490

[ AAPL × ] [ MSFT × ] [ NVDA × ] ... → click × to remove from scoring
```

### Apple HIG principles for v1.2.0

**Deference:** The Explorer is a secondary surface — it defers to the per-ticker drill-down. Row tap → load that ticker in the main view.

**Depth:** Three levels of information density:
1. Universe Explorer row: ticker, score, zone (summary)
2. Ranked list in main view: all signal components (intermediate)
3. Per-ticker drill-down: full chart, astrology, news, filings (full depth)

**Information hierarchy:** Score zone color is the primary signal. Numeric score is secondary. Company name is tertiary (truncated to fit). Users scan by color band, then read numbers.

---

## Plan of Action — Versioned Checklist

### v1.0.0 — COMPLETE *(2026-04-20)*

- [x] `edgar_enrich.rs` — SEC first-filing date enrichment
- [x] `wikidata_enrich.rs` — Wikidata SPARQL founding dates (UNION query fix, POST form method)
- [x] `fmp_enrich.rs` — FMP ticker universe seed + IPO date enrichment
- [x] `company_enrich.rs` — AV OVERVIEW IPO date enrichment
- [x] `enrich_common.rs` — DRY extraction: `seed_one_natal_chart` + `watchlist_priority_sql`
- [x] `tiingo.rs` — Tiingo bulk price history (490/day, watchlist-priority, 5-year history)
- [x] `main.rs` — wire all new modules + `TIINGO_API_KEY` env var
- [x] Migration 0017 — `ipo_date` nullable in `company_metadata`
- [x] T. Rowe Price removed from `INSTITUTION_MAP` (subsidiary CIK issue)
- [x] EDGAR CIK deduplication cache (share-class variants share one API call)
- [x] `Cargo.toml` — added `"form"` feature to reqwest for Wikidata SPARQL POST
- [x] `.env` — `TIINGO_API_KEY` added

---

### v1.1.0 — COMPLETE *(2026-04-20)*

**Database**
- [x] Write `migrations/0018_drop_price_data_fk.sql` — drop FK constraint, add composite index
- [x] Write `migrations/0019_scoring_active.sql` — add `scoring_active BOOLEAN` to `tickers`
- [x] Migrations applied (confirmed "Migrations OK" on 2026-04-20 scraper run)
- [x] Tiingo insert verified for non-watchlist tickers (scraper run 2026-04-20, no FK violations)

**Scraper**
- [x] `lagrange.rs` — replace `WATCHLIST` const loop with DB-driven `scoring_active = true` query
- [x] `tiingo.rs` — lower `MAX_PER_RUN` 490 → 45 (burst limit fix), sleep 200ms → 1000ms, graceful 429 stop
- [x] Verify: run scraper, confirm Lagrange scores for 10 tickers — no regression (confirmed 2026-04-20)

**Dashboard — autocomplete**
- [x] `db.rs` — add `search_tickers(prefix: String)` function (ILIKE prefix + company name fuzzy)
- [x] `state.rs` — add `autocomplete_suggestions: Vec<(String, String)>`
- [x] `state.rs` — add `Message::AutocompleteResults`, `Message::AutocompleteSelected`, `Message::ToggleWatchlistSort`
- [x] `update.rs` — `TickerSearchInput` fires `search_tickers` on each keystroke, clears on empty
- [x] `update.rs` — `AutocompleteSelected` clears suggestions and dispatches `TickerSelected`
- [x] `view.rs` — suggestion dropdown column below search bar (max 8 rows, "TICKER — Company Name")

**Dashboard — ranking panel**
- [x] `view.rs` — "Scored Universe — N tickers" header
- [x] `view.rs` — `scrollable(...).height(Length::Fixed(200.0))` on ranking rows
- [x] `view.rs` — Sort toggle button (Score desc / Ticker asc) in panel header
- [x] `update.rs` — `ToggleWatchlistSort` flips bool, re-sorts watchlist in place
- [x] `update.rs` — `RefreshNow` expanded to reload all panels (lagrange history, market FG, watchlist, macro, alerts)

**Dashboard — no-data states**
- [x] Chart: placeholder text when `self.rows.is_empty()` instead of blank canvas
- [x] Signal Intelligence: "No price data yet" guard on outer `if self.rows.is_empty()`
- [x] Indicator row: "Indicators: —" when rows empty
- [x] Astrology: conditional render — informational message if `natal_positions.is_empty()`

**Regression testing checklist**
- [x] All 10 existing watchlist buttons still work
- [x] Recently viewed still populates
- [x] Lagrange sparkline still renders for watchlist tickers
- [x] Alert panel still shows / marks read
- [x] Price chart still renders `LIMIT 100` rows
- [x] No FK violations in scraper output

---

### v1.2.0 — PREP ITEMS *(completed during v1.1.0, 2026-04-20)*

- [x] `company_metadata` — add `sector TEXT`, `industry TEXT` columns (migration 0020)
- [x] `fmp_enrich.rs` — capture `sector` / `industry` from `/v3/profile` response during enrichment; `FmpProfile` struct expanded; `fetch_profile_ipo_date` returns `(Option<NaiveDate>, Option<String>, Option<String>)`; sector/industry written via `COALESCE` upsert
- [x] `lagrange_history` — add `ticker_count INTEGER` column (migration 0020)
- [x] `state.rs` — add `universe_page: usize`, `universe_filter_zone: Option<String>` stubs
- [ ] Plan the Universe Explorer panel layout (wireframe in comments or separate WIREFRAME.md)

---

### Backlog (no version assigned yet)

- [ ] `docker-compose.yml` — reproducible local PostgreSQL setup
- [ ] FINRA API token refresh — current key expires when session does; need OAuth2 refresh flow
- [ ] Polygon.io Starter plan ($29/mo) — full options snapshot endpoint
- [ ] Swiss Ephemeris (`swisseph` FFI) — sub-arcsecond planetary accuracy
- [ ] One-click installer / packaged binary for distribution

### Completed

- [x] CPI display: YoY% via SQL CTE — v0.7.0
- [x] Lagrange Score history accumulation — v0.7.0
- [x] Universal ticker seed + birth chart database — v0.8.0
- [x] Dynamic ticker search + recently viewed — v0.8.0
- [x] Alert threshold system — v0.9.0
- [x] IPO date enrichment pipeline (4 sources) — v1.0.0
- [x] Tiingo bulk price history — v1.0.0
- [x] Codebase DRY refactor (`enrich_common.rs`) — v1.0.0

---

## Appendix: v0.5.0 Astrology Implementation Detail

*(kept for reference — implementation is complete)*

### Calculation Engine

No external API. No new crates. Pure Rust math using Jean Meeus *Astronomical Algorithms* formulas.
**Planets:** Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto.

### Aspect Scoring

| Aspect | Angle | Orb |
|--------|-------|-----|
| Conjunction | 0° | ±8° |
| Sextile | 60° | ±6° |
| Square | 90° | ±8° |
| Trine | 120° | ±8° |
| Opposition | 180° | ±8° |

Score formula: `clamp(50 + Σ aspect_deltas + moon_modifier, 0, mercury_rx_cap)`

### IPO Seed Data

| Ticker | Company | IPO Date | Exchange |
|--------|---------|----------|----------|
| AAPL | Apple | 1980-12-12 | NASDAQ |
| MSFT | Microsoft | 1986-03-13 | NASDAQ |
| GOOGL | Alphabet | 2004-08-19 | NASDAQ |
| AMZN | Amazon | 1997-05-15 | NASDAQ |
| META | Meta | 2012-05-18 | NASDAQ |
| TSLA | Tesla | 2010-06-29 | NASDAQ |
| NVDA | NVIDIA | 1999-01-22 | NASDAQ |
| JPM | JPMorgan Chase | 1969-05-01 | NYSE |
| V | Visa | 2008-03-19 | NYSE |
| UNH | UnitedHealth | 1984-10-17 | NYSE |

All at 09:30 EST. Coordinates: New York (40.7128°N, 74.0060°W).
