---
name: FinceptTerminal API Reference + v3.1 Integration Plan
description: 13 API integrations cataloged from FinceptTerminal; 3 scheduled for v3.1.x (DBnomics, RSS, Polymarket). Local C++ source saved for Rust translation.
type: reference
---

FinceptTerminal (C++20/Qt6) API integrations cataloged and saved locally for porting reference.

**Local files:**
- `reference/fincept_terminal_api_catalog.md` -- full API catalog with Rust port notes
- `reference/fincept_src/` -- 9 key C++ source files (3,793 lines) fetched from GitHub

**v3.1.x "The Network" -- scheduled integrations:**
1. **v3.1.0 DBnomics** (`https://api.dbnomics.org`) -- free, no API key, 70+ providers. New file: `src/scraper/dbnomics.rs`. Ported from `reference/fincept_src/services/dbnomics/DBnomicsService.cpp`.
2. **v3.1.1 RSS News** -- 60 financial RSS feeds, parallel fetch. New file: `src/scraper/rss_news.rs`, new crate: `feed-rs`. Ported from `reference/fincept_src/services/news/NewsService.cpp`.
3. **v3.1.2 Polymarket** -- prediction market probabilities. New file: `src/scraper/polymarket.rs`. Three API bases (Gamma/CLOB/Data). Ported from `reference/fincept_src/services/polymarket/PolymarketService.cpp`.
4. **v3.1.3 Dashboard wiring** -- macro strip, merged news, Polymarket gauges.

**Future (backlog, not versioned yet):**
- Databento derivatives data (paid API, v4.x)
- WebSocket real-time streaming (v4.x)
- Broker trading via Alpaca (v5.x)
- DataHub pub/sub architecture pattern (v4.x)

**How to apply:** Implementation follows v3.0.7 bug fixes. v3.1.0-v3.1.2 are independent and can be built in parallel. v3.1.3 wires them all into the dashboard.
