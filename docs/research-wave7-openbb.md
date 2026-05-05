# Wave 7 Research: OpenBB Platform Integration

**Date:** 2026-05-04
**Decision:** Add OpenBB as new data tier alongside existing 20 scrapers. Do NOT replace working code. Treat OpenBB as "deep cabinet" for 350+ datasets we can't easily integrate one-by-one.

---

## Why Add (Not Replace)

Existing scrapers have 6 months of bespoke logic per source: rate limits, retries, schema mapping, fallback chains (Wave 6.A1/A2). Tearing them out for OpenBB = losing hardened production code.

OpenBB's value comes from access to datasets we currently lack: CFTC commitment of traders, IMF, World Bank, ECB, OECD, granular SEC institutional ownership. Building native scrapers for each = months. OpenBB Platform aggregates them through one Python interface.

## Two Integration Modes

OpenBB has two product surfaces. We can pursue both independently.

### Mode 1: Platform API (programmatic — Rust → Python data pipe)

**What:** OpenBB Platform runs as separate Python process exposing REST API at `localhost:6900`. Our Rust scraper makes HTTP calls to enrich our existing tables.

**Setup:**
```bash
python3 -m venv .venv
source .venv/bin/activate         # or .venv\Scripts\Activate.ps1 on Windows
pip install openbb
openbb-api                         # serves on localhost:6900
```

**Endpoints to explore:**
- `/api/v1/economy/calendar`
- `/api/v1/economy/indicators` (FRED + WB + ECB through one interface)
- `/api/v1/regulators/sec/institutional-holdings/{cik}`
- `/api/v1/derivatives/futures/curve` (CFTC adjacent)
- `/api/v1/etf/holdings/{symbol}`

**Rust integration shape:**
```rust
// src/scraper/sources/openbb.rs
pub async fn fetch_endpoint(
    client: &reqwest::Client,
    path: &str,
    params: &[(&str, &str)],
) -> Result<serde_json::Value> {
    let base = std::env::var("OPENBB_API_URL")
        .unwrap_or_else(|_| "http://localhost:6900".to_string());
    let url = format!("{base}{path}");
    let resp = client.get(&url).query(params).send().await
        .context("OpenBB Platform request failed")?;
    if !resp.status().is_success() {
        anyhow::bail!("OpenBB HTTP {}", resp.status());
    }
    resp.json().await.context("OpenBB JSON parse")
}

pub async fn check_platform_available(client: &reqwest::Client) -> bool {
    fetch_endpoint(client, "/api/v1/health", &[]).await.is_ok()
}
```

Scraper startup logs warning if Platform not running but doesn't crash:
```rust
if !openbb::check_platform_available(&client).await {
    eprintln!("[OpenBB] Platform not running on localhost:6900. \
               Skipping OpenBB-sourced phases.");
}
```

### Mode 2: Workspace (cloud research UI)

**What:** OpenBB Workspace is a hosted browser dashboard at `pro.openbb.co`. Connects to our local Platform via ngrok tunnel. Lets us build interactive research views.

**NOT integrated into our Rust dashboard.** Runs in parallel as a separate research/exploration tool. Useful for:
- Quick data exploration before deciding to build native scraper
- Design inspiration for our dashboard
- Sharing screenshots in pitch decks

**Setup chain:**
```bash
ngrok http 6900                    # exposes localhost:6900 to public URL
# → Sign up at pro.openbb.co
# → Generate Personal Access Token
# → Add backend URL with PAT in Workspace settings
# → Add header: ngrok-skip-browser-warning: true
```

**Caveat:** Workspace runs in OpenBB's cloud, processes the data we send through them. If exposing proprietary data (Lagrange scores, astro patterns), audit privacy terms first. For just FRED/SEC public data, no concern.

## Architecture

```
┌──────────────────────────────────────────────────┐
│       Pursuit Dashboard (Rust/Iced)              │
│  ┌───────────────────────────────────────────┐   │
│  │  Existing 20 scrapers (untouched)         │   │
│  │  AV, FMP, Finnhub, Tiingo, Yahoo, Stooq,  │   │
│  │  EDGAR, FRED, GDELT, Polymarket, RSS,...  │   │
│  └───────────────────────────────────────────┘   │
│  ┌───────────────────────────────────────────┐   │
│  │  NEW: OpenBB Bridge (Wave 7)              │   │
│  │  src/scraper/sources/openbb.rs            │   │
│  │  HTTP client → localhost:6900             │   │
│  └───────────────────────────────────────────┘   │
└──────────────────────────────────────────────────┘
                      ↓
        ┌──────────────────────────────┐
        │ OpenBB Platform (Python)     │
        │ localhost:6900               │
        │ Wraps 350+ data providers    │
        └──────────────────────────────┘
                      ↓ (optional)
        ┌──────────────────────────────┐
        │ OpenBB Workspace (cloud UI)  │
        │ pro.openbb.co                │
        │ Reads via ngrok tunnel       │
        │ Research/inspiration only    │
        └──────────────────────────────┘
```

## Sub-Wave Breakdown

### 7.0 — "The Connection" (~1 day)
Bare-metal Platform setup. No Rust changes.

1. Create `.venv` in project root, install `openbb`
2. Run `openbb-api`, verify health endpoint
3. Curl 3-5 free endpoints to understand response shapes
4. Document in `docs/openbb-setup.md`: install steps, port, env vars, troubleshooting common errors

**Verification:** `curl http://localhost:6900/api/v1/health` returns 200.

### 7.1 — "The Bridge" (~2 days)
Rust ↔ Platform pipe + first new dataset.

**Pick first dataset by ROI:**

| Candidate | Coverage gap closed | Effort | OpenBB endpoint |
|-----------|---------------------|--------|-----------------|
| SEC institutional ownership | Granular 13F-HR holdings vs current quarterly aggregate | Low | `/regulators/sec/institutional-holdings` |
| World Bank macro | International country data we don't have | Low | `/economy/indicators?provider=worldbank` |
| CFTC Commitment of Traders | Futures positioning sentiment | Medium | `/derivatives/futures/cot` |
| IMF data | Sovereign macro | Low | `/economy/indicators?provider=imf` |

**Recommendation: World Bank macro first.** Closest fit to existing FRED scraper pattern. Schema parallel. Lowest risk for first integration. Confirms the pipe works on real data before tackling more complex datasets.

**Files to create:**
- `src/scraper/sources/openbb.rs` — generic HTTP client
- `src/scraper/world_bank.rs` — specific World Bank wrapper using openbb client
- `migrations/0043_world_bank_indicators.sql` — table

**Wire-up:**
```rust
// src/scraper/main.rs run_all_fetches phase 2.4b (after FRED)
if openbb::check_platform_available(&client).await {
    println!("2.4b Fetching World Bank indicators (OpenBB)...");
    world_bank::fetch_all(Arc::clone(&pool), Arc::clone(&client)).await;
}
```

### 7.2 — "The Workspace" (~half day)
Cloud UI as a research tool. Mostly setup, no Rust.

1. Install ngrok, sign up for free account, get auth token
2. `ngrok http 6900` — get public URL
3. Sign up at `pro.openbb.co`, generate PAT
4. Workspace settings → add backend URL → paste ngrok URL → save
5. Add custom HTTP header `ngrok-skip-browser-warning: true`
6. Verify connection — Workspace should list all OpenBB Platform endpoints
7. Build one dashboard combining FRED + SEC + something — screenshot for portfolio

Document in `docs/openbb-workspace.md`. Include screenshots.

### 7.3 — "The Custom Backend" (~3 days, optional)
Expose OUR proprietary data (Lagrange scores, astro patterns) to Workspace via a small FastAPI service. Lets us see our data in Workspace alongside OpenBB's.

**New directory:** `services/openbb-bridge/`
- `main.py` — FastAPI app with CORS allowing `pro.openbb.co`
- `widgets.json` — declares our custom widgets to Workspace
- `requirements.txt` — fastapi, uvicorn, asyncpg
- `Dockerfile` — for deployment
- `.env.example` — DB connection strings

**Endpoints:**
- `GET /widgets.json` — widget catalog
- `GET /lagrange-scores?ticker={X}` — recent scores + components
- `GET /aspect-patterns?ticker={X}` — pattern detections
- `GET /eclipse-activations?ticker={X}` — upcoming eclipse hits
- `GET /fixed-stars?date={D}` — current fixed-star activations across watchlist

Connect to existing Postgres via read-only role:
```sql
CREATE ROLE openbb_readonly LOGIN PASSWORD '...';
GRANT CONNECT ON DATABASE pursuit TO openbb_readonly;
GRANT USAGE ON SCHEMA public TO openbb_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO openbb_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO openbb_readonly;
```

Run as sidecar to scraper:
```bash
cd services/openbb-bridge
uvicorn main:app --port 7100
ngrok http 7100   # second tunnel for our backend
```

**Caveat (privacy):** Workspace processes whatever data we expose. Lagrange scores are derivative work, fine to expose. Astro patterns + eclipse activations are derivative public ephemeris data, fine to expose. Anything that's commercially licensed (some FMP fields) — check terms before piping to a 3rd-party UI.

### 7.4 — "The Cross-Check" (~2 days)
Use OpenBB redundantly with existing sources to detect discrepancies.

**Pick overlap:** FRED. We pull `GDP`, `CPI`, `UNRATE`, `FEDFUNDS` directly. OpenBB also wraps FRED. Run both in parallel and compare same series.

**Mechanics:**
1. New row in `macro_indicators` tagged `data_source = 'openbb_fred'` alongside existing `'fred'`
2. Discrepancy detector job:
   ```sql
   SELECT a.indicator, a.fetch_date, a.value AS native, o.value AS openbb,
          ABS(a.value - o.value) / a.value AS pct_diff
   FROM macro_indicators a
   JOIN macro_indicators o
     ON a.indicator = o.indicator
    AND a.fetch_date = o.fetch_date
   WHERE a.data_source = 'fred'
     AND o.data_source = 'openbb_fred'
     AND ABS(a.value - o.value) / NULLIF(a.value, 0) > 0.005;
   ```
3. Log discrepancies with timestamp + values. Surface in dashboard's data-quality view (Wave 6.A4 already added the freshness column; extend it to include discrepancy_count).

**Why valuable:** if our FRED scraper has a bug (wrong field, off-by-one date), OpenBB FRED data won't share that bug. Discrepancies = bug signals. Builds confidence in our pipeline.

## Risks

- **Python operational complexity** — currently Rust-only project. Adding Python venv + `openbb-api` process means more startup steps. Mitigation: clear startup script (`scripts/start.sh` or `.ps1`) that boots both scraper + openbb-api. Document well.
- **`pip install openbb` is heavy** — pulls many transitive deps (pandas, numpy, etc.). Pin Python version (3.11), use venv to isolate. Disk: ~2GB.
- **Latency stack** — Rust → localhost → Python → external API. Each hop adds 50-200ms. Use only for non-time-sensitive enrichment. Real-time prices stay on direct AV/Tiingo/Yahoo path.
- **OpenBB Workspace data exposure** — Mode 2 ships data through their cloud. Read [their privacy docs](https://docs.openbb.co/) before exposing anything proprietary. Free public data (FRED, SEC) — fine. Internal Lagrange scores — audit first.
- **Maintenance: 2 update cycles** — `pip install --upgrade openbb` periodically. Pin minor version in venv requirements file to avoid surprise breaks.

## Open Questions

1. **Python venv location** — project root `.venv/` or separate `services/openbb-platform/.venv/`? Cleaner isolation = the latter, more discoverable = the former.
2. **OpenBB Pro features** — paid tier unlocks more providers (Polygon premium, etc.). Start free, evaluate after 7.0/7.1 ship.
3. **Containerization** — should `openbb-api` run via docker-compose alongside Postgres? Aligns with the existing `docker-compose.yml` backlog item. Decide after 7.0 confirms feasibility.
4. **Startup integration** — does scraper auto-start `openbb-api`, or assume user runs it separately? First version: assume separate, log warning if not running. Ship-2 version: auto-start as subprocess if Python detected.

## Recommended Sequence

1. **Ship 7.0** — verify OpenBB works on this machine. ~1 day. Low risk. Decision point: continue or abandon based on smoothness.
2. **Ship 7.1** — first real Rust-side integration. Validates the pipe with one dataset. ~2 days.
3. **Optional: Ship 7.2** — Workspace setup. Half day. Low value to the Rust dashboard but high value as exploration tool.
4. **Optional: Ship 7.4** — cross-check. Builds data-quality confidence. ~2 days.
5. **Defer 7.3** — custom backend. High effort, niche value. Only do this if Workspace becomes the primary research tool.

## Won't Do

- ❌ Replace existing scrapers with OpenBB equivalents. Existing code is hardened, OpenBB is unproven for our use case.
- ❌ Embed OpenBB Workspace inside our Iced dashboard. Wrong tool for that — Iced is desktop-native, Workspace is web. Run in parallel.
- ❌ Use OpenBB for time-sensitive data (real-time prices, options chains). Latency stack is too long. Stay on direct API for those.
