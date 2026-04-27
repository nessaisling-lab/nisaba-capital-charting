# FinceptTerminal API Integration Catalog

> **Source:** https://github.com/Fincept-Corporation/FinceptTerminal
> **Stack:** C++20 / Qt6, AGPL-3.0, ~13k stars
> **Cataloged:** 2026-04-22

## Architecture: DataHub Pub/Sub

All data flows through a central DataHub with topic-based routing (e.g.,
`market:quote:AAPL`, `econ:fred:GDP`). Services implement a `Producer` interface
that declares topic patterns and a `refresh()` method.

Key features:
- Per-producer rate limiting via `max_requests_per_sec()`
- Topic policies: configurable TTL, min refresh interval, push-only mode
- Request coalescing (100ms default batching window)
- In-flight deduplication (won't re-request if fetch already pending)

Two data fetching paths:
1. **Direct C++ HTTP** via `HttpClient` (Qt `QNetworkAccessManager`)
2. **Python subprocess** via `PythonRunner`/`PythonWorker` for yfinance, Databento, etc.

---

## 1. Market Data (Yahoo Finance / yfinance)

| Detail | Value |
|--------|-------|
| Provider | Yahoo Finance via `yfinance` Python library |
| Script | `yfinance_data.py` |
| Rate limit | 10 req/s |
| Transport | Python subprocess |
| Source file | `fincept-qt/src/services/markets/MarketDataService.cpp` |

**Commands:**
- `batch_all` -- bulk quotes + sparklines
- `batch_quotes` -- real-time quote data
- `batch_sparklines` -- intraday mini-charts
- `historical_period` -- OHLCV history
- `info` -- company info/profile
- `financial_ratios` -- key financial ratios
- `news` -- company news

**Cache TTLs:** quotes 30s, sparklines 10min, history 30min

### Rust Port Notes
We already use Alpha Vantage and FMP for this. yfinance is free but scraping-based,
so it's fragile. Our AV + FMP + Tiingo stack is more reliable for production. However,
yfinance's `info` endpoint returns extremely rich company profiles that could
supplement our EDGAR enrichment.

---

## 2. DBnomics (Economic Data)

| Detail | Value |
|--------|-------|
| Base URL | `https://api.dbnomics.org` |
| Rate limit | 3 req/s |
| Cache | providers 10min, datasets/series 5min |
| Transport | Direct C++ HTTP |
| Source file | `fincept-qt/src/services/dbnomics/DBnomicsService.cpp` |

**Endpoints:**
```
GET /providers                              -- list all data providers
GET /datasets/{provider}                    -- datasets per provider (limit=50, offset)
GET /series/{provider}/{dataset}            -- series listing (limit=50, offset, q=query)
GET /series/{provider}/{dataset}/{series}   -- observations (observations=1, format=json)
GET /search                                 -- global search (q=query, limit=20, offset)
```

### Rust Port Notes
DBnomics aggregates 70+ economic data providers into one API. **No API key required.**
This is a direct upgrade over our FRED-only macro data. Could replace or supplement
our FRED integration for international economic indicators. Simple REST API,
pagination via offset. Perfect candidate for a new `src/scraper/dbnomics.rs`.

**Priority: HIGH** -- free, no API key, covers international economics we're missing.

---

## 3. Polymarket (Prediction Markets)

| Detail | Value |
|--------|-------|
| Gamma URL | `https://gamma-api.polymarket.com` |
| CLOB URL | `https://clob.polymarket.com` |
| Data URL | `https://data-api.polymarket.com` |
| Rate limit | 10 req/s |
| Cache | tags/leaderboard 300s, markets/events 120s |
| Transport | Direct C++ HTTP |
| Source file | `fincept-qt/src/services/polymarket/PolymarketService.cpp` |

**Gamma endpoints (discovery):**
```
GET /markets                    GET /markets/{id}
GET /events                     GET /events/{id}
GET /search?query=              GET /tags
GET /comments?market_id=        GET /teams
```

**CLOB endpoints (pricing):**
```
GET /book?token_id=             GET /prices-history?market=&interval=
GET /midpoint?token_id=         GET /spread?token_id=
GET /last-trade-price?token_id=
```

**Data endpoints (analytics):**
```
GET /trades?market=             GET /holders?market=
GET /v1/leaderboard             GET /activity?market=
GET /live-volume                GET /open-interest?market=
```

### Rust Port Notes
Prediction markets as a sentiment/macro indicator is interesting. "What does the
market think the probability of a rate cut is?" Could be a novel signal in our
Lagrange composite. **No API key required** for read-only access. Consider as a
future v3.x feature alongside our existing macro indicators.

---

## 4. Geopolitics / Conflict Monitor

| Detail | Value |
|--------|-------|
| Base URL | `https://api.fincept.in/research/news-events` |
| Cache | 2min TTL |
| Transport | Direct C++ HTTP |
| Source file | `fincept-qt/src/services/geopolitics/GeopoliticsService.cpp` |

**Query params:** `country`, `city`, `event_category`, `limit`,
`get_unique_countries`, `get_unique_categories`, `get_unique_cities`

### Rust Port Notes
This is Fincept's proprietary API. Not usable for us. However, the concept of
tracking geopolitical events as market signals is worth exploring. GDELT
(https://www.gdeltproject.org) is a free alternative.

---

## 5. HDX (Humanitarian Data Exchange)

| Detail | Value |
|--------|-------|
| Script | `hdx_data.py` |
| Transport | Python subprocess |
| Source file | `fincept-qt/src/services/geopolitics/GeopoliticsService.cpp` |

**Operations:** `search_conflict`, `search_humanitarian`, `search_by_country`,
`search_by_topic`, `search_datasets`

### Rust Port Notes
Niche. Low priority for a financial terminal. Skip unless we add a geopolitics tab.

---

## 6. Maritime / Vessel Tracking

| Detail | Value |
|--------|-------|
| Base URL | `https://api.fincept.in/marine/` |
| Transport | Direct C++ HTTP |
| Source file | `fincept-qt/src/services/maritime/MaritimeService.cpp` |

**Methods:** search by area (bounding box), vessel position by IMO, multi-vessel
positions, vessel history

### Rust Port Notes
Proprietary Fincept API. For supply chain analysis. Very specialized. Skip.

---

## 7. Databento (Institutional Market Data)

| Detail | Value |
|--------|-------|
| Script | `databento_provider.py` |
| Transport | Python subprocess |
| API key | Required, managed via SecureStorage |
| Source file | `fincept-qt/src/services/databento/DatabentoService.h` |

**Functions:** OHLCV, options volatility surfaces, Greeks surfaces, futures term
structure, yield curves, forward rates, FX forward points, crack spreads, stress tests

### Rust Port Notes
Databento has excellent Rust SDK (`databento` crate). Premium data source.
Options volatility surfaces and Greeks would be a huge addition for our options
analysis (currently we only have basic Polygon options flow). Consider for v4.x
when we need institutional-grade derivatives data. **Paid service.**

---

## 8. News (80+ RSS Feeds)

| Detail | Value |
|--------|-------|
| Cache | 5min TTL |
| Transport | Direct C++ HTTP + XML parsing |
| Internal API | `api.fincept.in/news/analyze`, `/news/summarize` |
| Source file | `fincept-qt/src/services/news/NewsService.cpp` |

**Feed tiers:**
- **Tier 1 (Wire):** Reuters (4 feeds), AP, SEC, Federal Reserve, UN, IMF
- **Tier 2 (Financial):** Bloomberg, WSJ (2 feeds), MarketWatch, CNBC, CoinDesk,
  Seeking Alpha, BBC, Al Jazeera, NYT, Guardian, France 24
- **Tier 3-4:** ZeroHedge, Defense One, Ars Technica, The Verge, MIT Tech Review,
  Hacker News, etc.

Pure C++ XML parsing via `QXmlStreamReader`.

### Rust Port Notes
We already have Finnhub news. Adding RSS feeds would massively expand our news
coverage. Rust has `quick-xml` and `feed-rs` crates for RSS/Atom parsing. The big
win: **free, no API key, ~80 sources**. Consider adding a `src/scraper/rss_news.rs`
that fetches the top 20 financial RSS feeds. Priority: MEDIUM.

**Key RSS feeds to start with:**
```
Reuters Business:   https://www.reutersagency.com/feed/?best-topics=business-finance
SEC:                https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&type=8-K&dateb=&owner=include&count=40&search_text=&action=getcompany&output=atom
FRED:               https://fred.stlouisfed.org/feed
MarketWatch:        https://feeds.marketwatch.com/marketwatch/topstories
CNBC:               https://search.cnbc.com/rs/search/combinedcms/view.xml?partnerId=wrss01&id=100003114
```

---

## 9. AkShare (Chinese/Asian Markets)

| Detail | Value |
|--------|-------|
| Transport | Python subprocess (dynamic endpoint discovery) |
| Source file | `fincept-qt/src/services/akshare/AkShareService.h` |

### Rust Port Notes
China-specific. Skip unless we add international markets.

---

## 10. Economics (FRED + others)

| Detail | Value |
|--------|-------|
| Rate limit | 2 req/s |
| Cache | 10min TTL |
| Transport | Python subprocess |
| Source file | `fincept-qt/src/services/economics/EconomicsService.cpp` |

Topic format: `econ:<source>:<request_id>`

### Rust Port Notes
We already have FRED integration. DBnomics (item 2) is a better expansion path
since it aggregates FRED + 70 other providers.

---

## 11. Government Data

| Detail | Value |
|--------|-------|
| Script | `government_us_data.py` |
| Rate limit | 2 req/s |
| Cache | 5min TTL |
| Source file | `fincept-qt/src/services/gov_data/GovDataService.h` |

Provider metadata system (id, name, country, flag).

### Rust Port Notes
Could supplement our SEC EDGAR data. Low priority.

---

## 12. WebSocket Streams

| Provider | Protocol | Reconnect | Source |
|----------|----------|-----------|--------|
| Kraken | WSS | Auto, 10 max | `WebSocketClient.h` |
| HyperLiquid | WSS | Auto, 10 max | `WebSocketClient.h` |
| Polymarket | WSS | Auto, 10 max | `WebSocketClient.h` |

Ref-counted lazy channel management.

### Rust Port Notes
`tokio-tungstenite` is the standard Rust WebSocket client. Real-time streaming
would be a v4.x feature for live price updates without polling. Consider Alpaca
or Polygon WebSocket for US equities.

---

## 13. Broker Adapters (16 Brokers)

**Indian:** Zerodha (3 req/s), Angel One (1 req/s), Upstox, Fyers, Dhan, Groww,
Kotak, IIFL, 5paisa, AliceBlue, Shoonya, Motilal

**Global:** IBKR, Alpaca, Tradier, Saxo

Unified `IBroker` interface: place/modify/cancel orders, positions, holdings,
funds, GTT orders, WebSocket streaming.

**Files:** `fincept-qt/src/trading/brokers/` (16 subdirectories + shared `BrokerHttp.cpp`)

### Rust Port Notes
Trading integration is a major future feature. Alpaca has an official Rust crate.
IBKR has a REST API that works with `reqwest`. This is v5.x territory.

---

## Key Reference Files in Their Repo

| Component | Path |
|-----------|------|
| DataHub core | `fincept-qt/src/datahub/DataHub.h` |
| Producer interface | `fincept-qt/src/datahub/Producer.h` |
| Topic policy | `fincept-qt/src/datahub/TopicPolicy.h` |
| HTTP client | `fincept-qt/src/network/http/HttpClient.h` |
| WebSocket client | `fincept-qt/src/network/websocket/WebSocketClient.h` |
| Market data | `fincept-qt/src/services/markets/MarketDataService.cpp` |
| DBnomics | `fincept-qt/src/services/dbnomics/DBnomicsService.cpp` |
| Polymarket | `fincept-qt/src/services/polymarket/PolymarketService.cpp` |
| News/RSS | `fincept-qt/src/services/news/NewsService.cpp` |
| Geopolitics | `fincept-qt/src/services/geopolitics/GeopoliticsService.cpp` |
| Broker interface | `fincept-qt/src/trading/BrokerInterface.h` |
| Python runner | `fincept-qt/src/python/PythonRunner.h` |

---

## Patterns Worth Porting to Rust

### 1. DataHub Pub/Sub with TopicPolicy
Their topic-based architecture with per-topic TTL, min_interval, coalesce windows
maps to Rust channels (`tokio::broadcast` or custom hub). The `TopicPolicy` struct
is directly portable.

### 2. Producer Trait
The `Producer` interface (topic_patterns, refresh, max_requests_per_sec) maps cleanly
to a Rust trait. Their pattern-matching with `*`-suffix globbing is simple.

### 3. Hub-Level Rate Limiting
Rather than each service managing its own limiter, the hub scheduler paces
`refresh()` calls based on declared limits. In Rust: `tower::RateLimit` or
`governor` crate per producer.

### 4. Request Coalescing
100ms batching window groups multiple subscriber requests into one producer call.
Prevents duplicate fetches. A `tokio::time::sleep` + drain pattern in Rust.

### 5. Polymarket Three-API Pattern
Clean separation of Gamma (discovery), CLOB (pricing), Data (analytics) with a
shared HTTP helper. Good model for multi-endpoint providers.

### 6. RSS Feed Aggregation
80+ feeds in parallel with native XML parsing. Achievable in Rust with
`reqwest` + `quick-xml` or `feed-rs`. No API key, free, massive news coverage.

---

## Priority Ranking for Our Project

| # | Integration | Priority | API Key | Cost | Why |
|---|-------------|----------|---------|------|-----|
| 1 | DBnomics | **HIGH** | None | Free | 70+ economic providers, replaces/supplements FRED |
| 2 | RSS News Feeds | **MEDIUM** | None | Free | 80+ sources, supplements Finnhub news |
| 3 | Polymarket | **MEDIUM** | None | Free | Novel sentiment/probability signal |
| 4 | Databento | LOW | Required | Paid | Institutional derivatives data, v4.x |
| 5 | WebSockets | LOW | Varies | Varies | Real-time streaming, v4.x |
| 6 | Broker APIs | LOW | Required | Varies | Trading execution, v5.x |
