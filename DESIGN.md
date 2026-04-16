# Financial Dashboard — Design Document

**Project:** Pursuit NYC Week 4 Fellowship — Native Rust Desktop Financial Dashboard
**Stack:** Rust, Iced 0.13, SQLx, PostgreSQL
**Author:** Aisling Leiva
**Current version:** v0.7.0

---

## Changelog

### v0.7.0 — Lagrange History, Portfolio, CPI YoY%, Color-Coding *(completed 2026-04-16)*
**Theme:** Daily Lagrange history accumulation, portfolio positions, display polish

**Shared lib extraction:**
- [x] `src/indicators.rs` moved to lib crate — shared between scraper and dashboard binaries
- [x] `compute_lagrange_score` now returns `(f32, String, LagrangeComponents)` with component breakdown
- [x] `LagrangeComponents { fin_score, astro_score, macro_score, short_score }` stored for debugging

**New scraper module:**
- [x] `src/scraper/lagrange.rs` — daily Lagrange Score computation for all 10 tickers
- [x] Reads price/sentiment/astro/macro/short from DB, computes score, upserts to `lagrange_history`
- [x] `ON CONFLICT (ticker, score_date) DO NOTHING` — safe to re-run
- [x] Runs at end of `run_all_fetches` pipeline

**Migration 0013:**
- [x] `lagrange_history` table: one row per ticker per day, UNIQUE (ticker, score_date)
- [x] `portfolio_positions` table: user-editable via SQL, UNIQUE on ticker

**New models:**
- [x] `LagrangeHistory { ticker, score_date, score, label, fin_score, astro_score, macro_score, short_score }`
- [x] `PortfolioPosition { ticker, shares, avg_cost, notes }`

**Dashboard — Lagrange sparkline:**
- [x] `LagrangeSparkline` canvas widget: 90-day score history strip below price chart
- [x] Zone color bands (red/orange/yellow/green) matching Lagrange label thresholds
- [x] Grid lines at 25 / 45 / 55 / 75 zone boundaries
- [x] Date labels at first and last data point, score label at latest point
- [x] Falls back to "Not enough history yet" until ≥2 days of data

**Dashboard — Portfolio panel:**
- [x] Reads `portfolio_positions` table, shows ticker / shares / avg cost / total cost basis
- [x] Notes column display; total cost basis footer
- [x] Empty state message directing user to portfolio_seed.sql

**Dashboard — display polish:**
- [x] Watchlist ranking: zone-coded score column (e.g. `66 ■ Fav`)
- [x] Macro strip: raw Decimal → f64 → `{:.2}` formatting (was 6 decimal places)
- [x] CPI strip item: raw index value replaced with YoY% via SQL CTE window calculation
- [x] `WatchlistRow.astro_score`: `i32` → `f64` (type mismatch was silently dropping all watchlist data)
- [x] `fetch_macro_indicators`: fixed `observation_date` → `obs_date` (actual DB column name)

**Data quality fixes:**
- [x] FINRA short interest: added symbol guard in aggregation — previously summed all records regardless of ticker
- [x] Deleted corrupt 43.5957% aggregate OTC short ratio rows from all 10 tickers
- [x] Short interest data limitation documented: regShoDaily is OTC-only; exchange-listed stocks return 0 rows (correct)

---

### v0.6.0 — Expanded Data Sources + Actionable Intelligence *(completed 2026-04-15)*
**Theme:** FRED macro data, FINRA short interest, Lagrange Score, signal synthesis

**Scraper module split:**
- [x] Split 1,495-line `src/scraper/main.rs` monolith into 9 focused modules
- [x] `prices.rs` — Alpha Vantage price fetch
- [x] `edgar.rs` — SEC EDGAR Form 4 + 8-K fetch
- [x] `holdings.rs` — 13F institutional holdings
- [x] `finnhub.rs` — news, earnings calendar, analyst ratings
- [x] `sentiment.rs` — Alpha Vantage NEWS_SENTIMENT
- [x] `astrology.rs` — natal chart seeding + daily transit/score computation
- [x] `macro_data.rs` — FRED macroeconomic series (10 indicators)
- [x] `short_interest.rs` — FINRA Developer API short sale volume
- [x] `options.rs` — Polygon.io options flow (graceful skip on free tier)

**New data sources:**
- [x] FRED (St. Louis Fed): Fed Funds Rate, CPI, Unemployment, 10Y/2Y Treasury yields, yield spread, real GDP, VIX, M2, WTI crude — 1,660+ rows per first run
- [x] FINRA Developer API: per-ticker short sale volume (3 reporting facilities aggregated via HashMap), `short_pct` computed
- [x] Polygon.io: options flow put/call ratio — free-tier probe pattern (`probe_options_access`) detects 401/403/404 and skips gracefully without burning rate limit
- [x] Migration 0012: `macro_indicators`, `short_interest`, `options_flow` tables

**Lagrange Score (renamed from Pursuit Score):**
- [x] Blends 4 signal components: Financial 35% + Astrology 25% + Macro Environment 25% + Short Squeeze Signal 15%
- [x] Financial: RSI(14) + SMA50 momentum + MACD histogram + AV sentiment
- [x] Macro: VIX score (inverted: calm = bullish) + yield curve spread
- [x] Short squeeze: high short % × rising price = elevated score
- [x] Labels: Misaligned / Unfavorable / Neutral / Favorable / Optimal
- [x] 5th gauge in the gauge row

**Dashboard — new panels:**
- [x] Macro strip: Fed Funds, CPI, Unemployment, 10Y, 2Y, Spread, VIX, WTI Oil (live from DB)
- [x] Short% appended to indicator row
- [x] Signal Intelligence panel: plain-English bullet synthesis per ticker
  - RSI overbought/oversold, Bollinger position, MACD direction
  - Short interest squeeze setup detection
  - News sentiment, analyst consensus, astrology context
  - VIX regime, yield curve shape, Mercury Rx warning
  - Upcoming earnings countdown with volatility warning
- [x] Watchlist Ranking panel: all 10 tickers sorted by quick Lagrange Score
  - Astro score, sentiment label, short % per ticker
  - Click any row to switch to that ticker
- [x] Horizontal `scrollable` wrapper on gauge row (5 gauges, no clipping)
- [x] `horizontal_rule` separators between all major sections
- [x] Padded `container` wrappers on Signal Intelligence + Watchlist panels
- [x] Bullet character `•` on all signal items

---

### v0.5.0 — Astrology Layer *(completed 2026-04-14)*
**Theme:** Company birth charts, planetary transit scoring, astrological Fear & Greed

- [x] Migration 0008: `company_metadata` (IPO dates, exchange, coordinates)
- [x] Migration 0009: `natal_positions` (pre-computed natal chart per ticker)
- [x] Migration 0010: `daily_transits` (today's planetary positions)
- [x] Migration 0011: `astro_scores` (per-ticker astrological F&G per day)
- [x] `src/astrology/ephemeris.rs` — pure-Rust planetary position math (Jean Meeus formulas)
- [x] `src/astrology/aspects.rs` — aspect detection (conjunction/sextile/square/trine/opposition)
- [x] `src/astrology/natal.rs` — natal chart derivation + transit-to-natal scoring
- [x] Scraper: natal chart seeder (per-ticker NOT EXISTS check) + daily transit computation + astro score storage
- [x] Dashboard: astrological F&G gauge (4th gauge in row)
- [x] Dashboard: natal chart wheel canvas widget (inner ring = natal, outer = current transits)
- [x] Dashboard: active transits table with aspect, orb, and effect
- [x] Dashboard: moon phase display + Mercury Rx flag

See full plan: [§ v0.5.0 Astrology Implementation Plan](#v050-astrology-implementation-plan)

---

### v0.4.0 — Module Refactor *(completed 2026-04-14)*
**Theme:** Split 1,435-line monolith into maintainable modules

- [x] `src/dashboard/main.rs` — entry point only (14 lines)
- [x] `src/dashboard/state.rs` — `Dashboard` struct + `Message` enum
- [x] `src/dashboard/indicators.rs` — SMA, EMA, RSI, MACD, Bollinger Bands + `compute_ticker_score`
- [x] `src/dashboard/helpers.rs` — formatting utilities
- [x] `src/dashboard/db.rs` — all async DB + API fetch functions
- [x] `src/dashboard/gauges.rs` — `FearGreedGauge` canvas widget
- [x] `src/dashboard/charts.rs` — `PriceChart` canvas widget with hover
- [x] `src/dashboard/update.rs` — `impl Dashboard { update, new, subscription, ... }`
- [x] `src/dashboard/view.rs` — `impl Dashboard { view }`
- [x] `.gitignore` created (`/target/`, `.env`, IDE folders)
- [x] Gauge light-mode readability fix (theme-aware color variables)

---

### v0.3.0 — UI / UX Pass *(completed 2026-04-13)*
**Theme:** Usability, layout polish, copy/open buttons, Fear & Greed gauges

- [x] Full-page scrollable layout (`container(scrollable(content))`)
- [x] News and 8-K filings side-by-side (`row![].width(FillPortion(1))`)
- [x] Copy button on news rows (copies headline + URL to clipboard via `iced::clipboard::write`)
- [x] Open button on news rows (launches browser via `open::that_detached`)
- [x] Copy + Open buttons on 8-K filing rows
- [x] Three Fear & Greed gauges in a row:
  - Crypto / Risk Sentiment (alternative.me API — free, bot-friendly)
  - Equities Sentiment (DB aggregate: price breadth 50% + AV sentiment 30% + analyst buy ratio 20%)
  - Per-Ticker Score (RSI 30% + SMA50 momentum 30% + MACD histogram 20% + AV sentiment 20%)
- [x] Semicircular gauge canvas widget (five color zones: red → orange → yellow → yellow-green → green)
- [x] Toggle between Dark / Light mode
- [x] Fixed: CNN Fear & Greed API returned HTTP 418 "I'm a teapot" — switched to alternative.me
- [x] Fixed: PostgreSQL `AVG(NUMERIC)` type mismatch — cast all literals to `::float8`
- [x] Fixed: gauge canvas lifetime issue — changed closure params from `&str` to owned `String`

---

### v0.2.0 — Data Enrichment *(completed 2026-04-12)*
**Theme:** News, earnings, analyst ratings, sentiment pipeline from Finnhub + Alpha Vantage

- [x] Migration 0004: `news_articles` (Finnhub company news)
- [x] Migration 0005: `earnings_dates` (Finnhub earnings calendar)
- [x] Migration 0006: `analyst_ratings` (Finnhub recommendation trends)
- [x] Migration 0007: `sentiment_scores` (Alpha Vantage NEWS_SENTIMENT)
- [x] Scraper: Finnhub news fetch with rate limiting (governor)
- [x] Scraper: Finnhub earnings calendar fetch
- [x] Scraper: Finnhub analyst ratings fetch
- [x] Scraper: Alpha Vantage sentiment fetch + label derivation
- [x] Dashboard: earnings calendar section (all tickers, sorted, upcoming highlighted with `>>`)
- [x] Dashboard: news headlines section with source + date
- [x] Dashboard: analyst consensus inline display (SB/B/H/S/SS counts)
- [x] Dashboard: AV sentiment label + score inline display
- [x] Cron scheduler wired in scraper (`tokio-cron-scheduler`, fires 6:00 AM daily)
- [x] Price chart hover tooltip with crosshair + OHLCV data (`canvas::Program<Message>` with `type State = Option<Point>`)

---

### v0.1.0 — Foundation *(completed 2026-04-10)*
**Theme:** Core scraper + dashboard skeleton

- [x] Migration 0001: initial schema (`tickers`, `price_data`, `insider_trades`, `institutional_holdings`, `filings`)
- [x] Migration 0002: seed watchlist tickers
- [x] Migration 0003: add `items` column to filings
- [x] Scraper binary: Alpha Vantage TIME_SERIES_DAILY → `price_data`
- [x] Scraper binary: SEC EDGAR Form 4 → `insider_trades`
- [x] Scraper binary: SEC EDGAR 13F → `institutional_holdings`
- [x] Scraper binary: SEC EDGAR 8-K → `filings`
- [x] Rate limiting (governor) + retry logic on all HTTP calls
- [x] Dashboard binary: PostgreSQL connection via SQLx
- [x] Dashboard: ticker selector buttons
- [x] Dashboard: candlestick price chart (Iced canvas)
- [x] Dashboard: SMA(20), SMA(50), Bollinger Bands overlays
- [x] Dashboard: RSI(14), MACD indicator summary row
- [x] Dashboard: institutional holdings table (13F)
- [x] Dashboard: insider trades table (Form 4)
- [x] Dashboard: recent 8-K filings with item descriptions
- [x] Dashboard: 30-second auto-refresh subscription

---

## Architecture

### Binary layout

```
pursuit_week4_automation/
├── src/
│   ├── lib.rs                    shared types (models, astrology)
│   ├── models.rs                 SQLx FromRow structs
│   ├── astrology/                planetary calculation engine
│   │   ├── mod.rs
│   │   ├── ephemeris.rs
│   │   ├── aspects.rs
│   │   └── natal.rs
│   ├── scraper/
│   │   ├── main.rs               entry point + WATCHLIST + scheduler
│   │   ├── prices.rs             Alpha Vantage daily OHLCV
│   │   ├── edgar.rs              SEC EDGAR Form 4 + 8-K
│   │   ├── holdings.rs           SEC EDGAR 13F institutional
│   │   ├── finnhub.rs            news, earnings, analyst ratings
│   │   ├── sentiment.rs          Alpha Vantage NEWS_SENTIMENT
│   │   ├── astrology.rs          natal seeding + transit scoring
│   │   ├── macro_data.rs         FRED macroeconomic series
│   │   ├── short_interest.rs     FINRA Developer API short volume
│   │   └── options.rs            Polygon.io put/call ratio
│   └── dashboard/
│       ├── main.rs               entry point + mod declarations
│       ├── state.rs              Dashboard struct + Message enum
│       ├── indicators.rs         SMA/EMA/RSI/MACD/BB + Lagrange Score
│       ├── signals.rs            plain-English signal bullet generator
│       ├── helpers.rs            formatting utilities
│       ├── db.rs                 async DB + API fetch functions + WatchlistRow
│       ├── gauges.rs             FearGreedGauge canvas widget
│       ├── charts.rs             PriceChart canvas widget with hover
│       ├── update.rs             update() + new() + subscription()
│       ├── view.rs               view() layout
│       └── astrology.rs          natal wheel canvas + transits table
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
│   └── 0012_macro_indicators.sql     macro_indicators + short_interest + options_flow
├── Cargo.toml
├── .env                          secrets (never committed)
├── .gitignore
├── CLAUDE.md
└── DESIGN.md                     this file
```

### Data flow

```
Alpha Vantage API  ──┐
SEC EDGAR API      ──┤
Finnhub API        ──┤
FRED API           ──┤  scraper binary (startup + cron 6AM UTC)  ──►  PostgreSQL
FINRA API          ──┤
Polygon.io API     ──┤
alternative.me     ──┘

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

## v0.5.0 Astrology Implementation Plan

### Goal

Attach a birth chart to each publicly traded company (using IPO date as the "birth moment"), compute daily planetary transits against that natal chart, score the astrological climate, and display it alongside the existing financial indicators.

### Premise

Financial astrology is a real niche — W.D. Gann built entire trading systems around it. Whether it "works" is beside the point. This dashboard treats it as a creative, exploratory lens: a different way to read market cycles. The scoring is honest about what it is (astrological, not fundamental).

---

### Calculation Engine

No external API. No new crates. Pure Rust math using Jean Meeus *Astronomical Algorithms* formulas.

**Accuracy:** ~1-2 degrees for outer planets, ~1 arcminute for Sun/Moon.
**Good enough for:** Astrology (typical aspect orbs are 6-10 degrees).
**Not good enough for:** Precise house cusp calculation (requires Swiss Ephemeris — can add later via `swisseph` FFI without touching anything else).

**Planets computed:** Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto.

**Core pipeline:**
```
Date + Time (IPO or Today)
    → Julian Day Number
    → T = centuries from J2000.0
    → Mean longitude (polynomial in T, per planet)
    → Eccentricity + equation-of-center corrections
    → Ecliptic longitude 0-360°
    → Zodiac sign + degree within sign
```

---

### Aspect Scoring

**Five classical aspects:**

| Aspect | Angle | Orb |
|--------|-------|-----|
| Conjunction | 0° | ±8° |
| Sextile | 60° | ±6° |
| Square | 90° | ±8° |
| Trine | 120° | ±8° |
| Opposition | 180° | ±8° |

**Planet nature:**
- Benefic (positive): Jupiter, Venus
- Malefic (challenging): Saturn, Pluto, Mars
- Neutral (context-dependent): Sun, Moon, Mercury, Uranus, Neptune

**Score delta per aspect:**
- Benefic planet + harmonious aspect (trine/sextile): +8 to +15
- Benefic planet + challenging aspect (square/opposition): -4 to -8
- Malefic planet + challenging aspect: -10 to -18
- Malefic planet + harmonious aspect: +2 to +5 (mitigated)
- Orb modifier: linear falloff from exact (100%) to max orb (25%)

**Final score formula:**
```
base_score = clamp(50 + Σ aspect_deltas, 0, 100)

moon_modifier:
  New Moon (0-30°)      → +5
  Waxing (30-150°)      → +8
  Full Moon (150-210°)  → -5
  Waning (210-330°)     → -8
  Balsamic (330-360°)   → -3

mercury_rx_cap:
  Mercury retrograde    → cap score at 65 (uncertainty)

astro_score = clamp(base_score + moon_modifier, 0, mercury_rx_cap)
```

**Labels:** Extreme Fear (0-24) / Fear (25-44) / Neutral (45-55) / Greed (56-75) / Extreme Greed (76-100)

---

### Database Schema (Migrations 0008-0011)

**0008 — company_metadata**
```sql
CREATE TABLE company_metadata (
    ticker        TEXT PRIMARY KEY,
    company_name  TEXT NOT NULL,
    ipo_date      DATE NOT NULL,
    ipo_time      TIME NOT NULL DEFAULT '09:30:00',
    exchange      TEXT NOT NULL DEFAULT 'NYSE',
    latitude      DOUBLE PRECISION NOT NULL DEFAULT 40.7128,
    longitude     DOUBLE PRECISION NOT NULL DEFAULT -74.0060,
    founding_date DATE,
    notes         TEXT
);
-- Seeded with IPO dates for all watchlist tickers
```

**0009 — natal_positions**
```sql
CREATE TABLE natal_positions (
    ticker     TEXT NOT NULL,
    planet     TEXT NOT NULL,
    longitude  DOUBLE PRECISION NOT NULL,
    sign       TEXT NOT NULL,
    degree     DOUBLE PRECISION NOT NULL,
    retrograde BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (ticker, planet)
);
```

**0010 — daily_transits**
```sql
CREATE TABLE daily_transits (
    fetch_date  DATE NOT NULL,
    planet      TEXT NOT NULL,
    longitude   DOUBLE PRECISION NOT NULL,
    sign        TEXT NOT NULL,
    retrograde  BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (fetch_date, planet)
);
```

**0011 — astro_scores**
```sql
CREATE TABLE astro_scores (
    ticker          TEXT NOT NULL,
    score_date      DATE NOT NULL,
    astro_score     DOUBLE PRECISION,
    astro_label     TEXT,
    moon_phase      TEXT,
    moon_phase_deg  DOUBLE PRECISION,
    mercury_rx      BOOLEAN,
    active_aspects  JSONB,
    PRIMARY KEY (ticker, score_date)
);
```

---

### IPO Date Seed Data

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
| BRK.B | Berkshire Hathaway | 1996-05-09 | NYSE |

All at 09:30 EST (market open). All coordinates: NYSE/NASDAQ, New York (40.7128°N, 74.0060°W).

---

### New UI Components

**A) Astrological F&G Gauge**
Reuses `FearGreedGauge` canvas widget. Fourth gauge in the gauges row. Data: `astro_score`.

**B) Natal Chart Wheel** (new canvas widget in `astrology.rs`)
- Circle divided into 12 equal zodiac sign sectors (30° each)
- Inner ring: natal planet positions at IPO (gold)
- Outer ring: today's transiting positions (blue)
- Aspect lines drawn between rings when within orb
- Planet glyphs or abbreviations: Su Mo Me Ve Ma Ju Sa Ur Ne Pl

**C) Active Transits Table**
```
TRANSIT              NATAL          ASPECT      ORB    EFFECT
Jupiter (Gemini)  ⊡  Sun (Virgo)   Square       2.3°  Challenging
Venus (Pisces)    △  Jupiter        Trine        1.1°  Favorable
Mercury Rx ☿                                           Caution: uncertainty
```

**D) Moon Phase Line**
`Moon: Waxing Gibbous (142°)  •  Next Full Moon: Apr 23`

---

### File Changes (v0.5.0)

| File | Status |
|------|--------|
| `migrations/0008_company_metadata.sql` | New |
| `migrations/0009_natal_positions.sql` | New |
| `migrations/0010_daily_transits.sql` | New |
| `migrations/0011_astro_scores.sql` | New |
| `src/astrology/mod.rs` | New |
| `src/astrology/ephemeris.rs` | New |
| `src/astrology/aspects.rs` | New |
| `src/astrology/natal.rs` | New |
| `src/lib.rs` | Add `pub mod astrology` |
| `src/models.rs` | Add `AstroScore`, `NatalPosition`, `DailyTransit` |
| `src/scraper/main.rs` | Add transit + score cron steps |
| `src/dashboard/main.rs` | Add `mod astrology` |
| `src/dashboard/astrology.rs` | New — natal wheel canvas + transits table |
| `src/dashboard/state.rs` | Add 3 new fields |
| `src/dashboard/update.rs` | Add 3 new Message arms + fetch calls |
| `src/dashboard/view.rs` | Add astro section to layout |
| `src/dashboard/db.rs` | Add 3 new fetch functions |

**Estimated new code:** ~900 lines across 12 files.

---

## Open Items (TODOS)

- [ ] `docker-compose.yml` for reproducible local PostgreSQL setup
- [ ] CPI display: show YoY% change instead of raw index value (requires two observations + math in DB query)
- [ ] Lagrange Score for all watchlist tickers (currently only computes for selected ticker using price indicators; watchlist ranking uses simplified astro+sentiment proxy)
- [ ] FINRA API token refresh (current key `dd72083958f646ad8564` — expires when session does, need OAuth2 refresh flow)
- [ ] Polygon.io Starter plan ($29/mo) to unlock full options snapshot endpoint for richer put/call data
- [ ] Alert thresholds: highlight tickers in Optimal or Misaligned territory with color coding in watchlist ranking
- [ ] Swiss Ephemeris (`swisseph` FFI) for sub-arcsecond planetary accuracy if needed
