# TODOS

## Active — none. v12.1 shipped + user-validated 2026-05-06.

User: *"Overall huge win, very happy with the progress we made!"*

Next iteration awaits next user video review or backlog target.

## Open backlog (deferred)

### Carryover from v12.0 series
- **Chart hover real-time** — v12.0.B precompute + v12.0.D state-mutation skip both landed. User still flags non-real-time vs gauges/sparkline. Suspect: cosmic-text glyph shaping inside hover `fill_text` per redraw. Bigger lift — needs frame-time instrumentation OR pre-shaped glyph cache. Deferred to v12.3 or later.
- **OS toast HRESULT** — installer ships AUMID binding (v12.0.A). User reported `[fire_toast] OS notification FAILED: HRESULT(0x80070005)` even after install. Likely Start Menu indexer hasn't picked up the binding yet — recommend sign-out/in OR explorer.exe restart. Pill system covers visibility need; OS toast is bonus.

### v12.2 — "The Drawer + Polish" (planned, ~2-3 days)
Batched polish + completing the pill system.
- [ ] Settings tab scrollable wrap — user flagged in v8j ("can't scroll down the settings menu")
- [ ] Notification drawer — gear-click reveals 24h history overlay via `stack!`. UI sketched in `mockups/v12.1-pill-notifications.html` Scenario 5.
- [ ] Click-pill-to-dismiss — `DismissNotification(u64)` Message wired but no UI trigger yet
- [ ] Transit pills emit — wire astrology-event detection (Mars enters Aries, retrograde station, eclipse window) → `NotificationVariant::Transit`
- [ ] Toast → pill migration — replace residual `push_toast` calls with `notify_*` shortcuts
- [ ] Trim dead code — `alert_pill_until`, `fetch_ticker_error`, `show_settings_modal` flags now unused

### v12.3 — "The Profiling" (planned, ~1-2 days)
Deeper chart hover investigation + perf instrumentation.
- [ ] Frame-time instrumentation in `PriceChart::draw()` — log static_geo + hover_frame durations
- [ ] Test cosmic-text glyph cache hypothesis — measure `fill_text` cost vs strokes
- [ ] Investigate Iced 0.14 `canvas::Cache` for hover-side too (separate from static cache)
- [ ] Compare hover redraw rate vs visible cursor sweep speed — confirm whether issue is paint cost OR redraw count

### Wave 9 — "The Compounding" (planned, ~10 days)
Astrology engine deepening per Wave 6 spec carry-overs + new directions.
- [ ] Solar return charts (annual birthday-anchored sub-chart with its own transits)
- [ ] Progressed chart (1° = 1 year secondary progressions)
- [ ] Planetary returns timing (Saturn return, Jupiter return cycles)
- [ ] Aspect line color-by-strength (replace binary green/red with strength gradient)
- [ ] Backtest extension — current Lagrange backtest stops at 90 days, extend to 1+ year for cycle validation

### v12.0.E — packaging hardening (deferred)
- [ ] Code-signing the installer .exe (Windows Defender SmartScreen warning on download)
- [ ] Auto-update channel
- [ ] System-wide install option (currently per-user only)

---

## Closed — v12.1 "The Pill" — SHIPPED 2026-05-06

User: *"Overall huge win, very happy with the progress we made!"*

Source: video review v8j 2026-05-05 (`transcript-v8j.txt`) + screenshots 020627/020645/020657/020725.

- [x] Mockup-first artifact: `mockups/v12.1-pill-notifications.html` — 5 scenarios + pill catalog
- [x] New `src/dashboard/notifications.rs` (~210 lines) — Notification struct, 6 variants, render_pill, render_pill_stack
- [x] State fields: `notifications: VecDeque`, `notification_history: Vec`, `next_notification_id`, `alerted_lagrange_ids`, `fetch_notification_id`
- [x] Helpers: `push_notification`, `next_notif_id`, `dismiss_notification`, `expire_notifications`, `notify_error/success/info`
- [x] Messages: `DismissNotification(u64)`, `NotificationsTick`
- [x] **KILLED** push-down `fetch_error_banner` from main column flow
- [x] **KILLED** v11.9 ad-hoc fetching_pill + alert_pill blocks (~115 lines), replaced with `render_pill_stack`
- [x] FetchThisTicker → sparkly sticky pill; FetchTickerComplete → success/error pill
- [x] AlertsLoaded → Alert pill per new unread Lagrange (deduped via alerted_lagrange_ids), click → Universe
- [x] Tick keeps animation subscription active while pills exist (twinkle stays smooth)

## Closed — v12.0 series — SHIPPED 2026-05-06

### v12.0.A — OS toast real fix
- [x] Runtime: windows-rs IShellLinkW + IPropertyStore.SetValue(PKEY_AppUserModel_ID) on Start Menu .lnk at boot
- [x] Installer: Inno Setup `[Icons] AppUserModelID:` parameter binding (`installer/pursuit-astro.iss`)
- [x] Cargo dep: `windows = "0.58"` Win32_UI_Shell + Win32_UI_Shell_PropertiesSystem + Win32_Storage_EnhancedStorage
- [ ] Confirmed working post-install — user reported HRESULT 0x80070005 still firing on next session, deferred (Start Menu indexer)

### v12.0.B — Chart hover full profiling
- [x] Profiled hover redraw path — found O(n) min/max + 4n Decimal→String→f32 per cursor pixel
- [x] Added `price_min`, `price_max`, `ohlc_f32` fields to PriceChart
- [x] Helpers: `precompute_ohlc()`, `compute_price_range()`
- [x] Wired in `view/overview.rs` — runs once per chart construction
- [x] Hover redraw now O(1) field reads
- [ ] Real-time goal not met — see v12.3 / v12.0.D follow-up

### v12.0.C — Munger narrative expansion
- [x] Sector mental model: 6 → 11 sectors with Munger aphorism quotes
- [x] ROE: 1 → 5 bands
- [x] Operating margin: 2 → 3 bands
- [x] EV/EBITDA: 0 → 4 bands
- [x] Debt/equity: 0 → 4 bands
- [x] FCF: 0 → 3 bands
- [x] News tone: 2 → 4 cases
- [x] Astro angle: 0 → 2 bands
- [x] Score-band closer: 2 → 6 bands

### v12.0.D — Chart hover state-mutation skip
- [x] Found Iced 0.14 redraws on canvas state mutation alone (v11.7.D bar-snap missed half)
- [x] Skip `*state = new_pos` when prev_bar == next_bar
- [ ] Real-time still not achieved — deferred to v12.3 with cosmic-text profiling

---

## Closed — v11.9 "The Convergence" — SHIPPED 2026-05-05

User: *"working and exactly how I wanted!"*

- [x] Toast layout shift — `column![toast, main]` → `stack![main, toast]` overlay
- [x] Drop redundant `push_toast("Fetching data for…")` on FetchThisTicker
- [x] Fetching pill with ShootingStar canvas, anchored left of gear in tab strip
- [x] Alert pill with 8s TTL via `alert_pill_until: Option<Instant>` field
- [x] Both pills coexist; ordering tabs → spacer → alert_pill → fetching_pill → gear
- [x] Tab::Settings dropped from `Tab::all()`; gear icon is sole entry point
- [x] Loading bar restored to inline below header (was overlay attempt then push-down) with tip-star + percentage label
- [x] Mockup artifact: `docs/v11.9-layout-mockup.html`
- [x] Transcripts: `docs/video-review-v11.{7,8-issues,8.H,8.I,9-mockup-request}-transcript.txt`

## Closed — v11.8 "The Persistence" — SHIPPED 2026-05-05

- [x] **8.A** Settings prominence (revised in 8.I)
- [x] **8.B** Header right-side spacing tightened
- [x] **8.C** Paper Trail icon mockup round 3 — TARGET (concentric circles)
- [x] **8.D** Windows AUMID registry write at boot
- [x] **8.E** Loading bar dropped (reverted in 8.H)
- [x] **8.F** ShootingStar canvas animation
- [x] **8.G** Wikipedia phase 7 in scraper single-ticker fetch
- [x] **8.H** Loading bar restored with tip-star + drop pop-up pill
- [x] **8.I** PowerShell Start Menu shortcut + gear-only tab strip right

## Closed — v11.7 "The Resolution" — SHIPPED 2026-05-05

- [x] **7.A** Paper Trail icon RECEIPT (later replaced TARGET in 8.C)
- [x] **7.B** Munger Sell/StrongSell variants — 6 each, ticker-hash rotation
- [x] **7.C** OS notification diagnostic logging + Test Notification button
- [x] **7.D** Chart hover bar-index snap (1000Hz mouse polling → ~n redraws)
- [x] **7.E** Sparkle visibility audit
- [x] **7.F** Blinking root cause — re-seed 8Hz → 1.5Hz with sinusoidal alpha

---

## Closed — v11.6 "The Persistence" — COMPLETED 2026-05-05

**Source:** 6.5-min v11.6 production review (`docs/video-review-v11.6-transcript.txt`). User: *"Overall, huge improvement. Proper circling."* Council de-astro validated. 6 issues + 1 mockup request.

### v11.7.A — "Paper Trail icon redesign" — SHIPPED 2026-05-05

- [x] HTML mockup with 6 candidates (current GRAPH_UP, RECEIPT, NOTEBOOK, COINS, TROPHY, GAME_CONTROLLER) — `docs/v11.7.a-paper-trail-icon-mockup.html`
- [x] User picked RECEIPT (B) — literal paper-trail semantic
- [x] New const `icons::RECEIPT` (\u{e3aa})
- [x] `Tab::PaperTrail` icon swapped from GRAPH_UP

### v11.7.B — "Munger phrase variance" — SHIPPED

- [x] **B1** Sell/StrongSell got 6 variants each (was 1) with ticker-hash rotation
- [x] **B2** Density now matches Buffett/Graham/Lynch
- [x] **B3** Munger narrative expansion — shipped in v12.0.C 2026-05-06

### v11.7.C — "OS notifications debug" — SHIPPED (parked)

- [x] **C1** Diagnostic logging on fire_toast invocation + show() result
- [x] **C2** notifications_fired reset on MarkAllAlertsRead verified
- [x] **C3** notify-rust on Windows 11 fails with HRESULT 0x80070005 (AUMID/shortcut limitation, parked for installer)
- [x] **C4** "Send Test Notification" button added to Settings → Alerts card

### v11.7.D — "Chart hover lag" — SHIPPED (improved)

- [x] **D1** Bar-index snap on CursorMoved — only invalidate when crossing into new candle (1000Hz mouse → ~n redraws per pass)
- [ ] **D2** chart_draw_progress cache invalidation — deferred profiling
- [ ] **D3** Animation/static layer separation — deferred
- [ ] **D4** Hover frame cost profiling — deferred

### v11.7.E + 7.F — "Sparkle blinking" — SHIPPED

Same root cause: 8Hz seed re-roll teleporting particles. Fix:
- [x] Slowed sparkle re-seed 8Hz → 1.5Hz
- [x] Added sinusoidal alpha pulse for smooth visibility

---

## Closed — v11.6 "The Persistence" — COMPLETED 2026-05-05

### v11.6.A — "Header redo" — SHIPPED 2026-05-05

**Source:** 18-min post-v11.5 video review (`docs/video-review-v11.5-transcript.txt`) + iterative mockup loop (4 short videos totaling ~22min). Header layout was the loudest unresolved theme — re-spec'd three different ways before convergence.

### v11.6.A — "Header redo" — SHIPPED 2026-05-05

Resolved across 4 mockup iterations. Final layout:
- [x] Tab strip relocated to very top of page (full-width, above ornament rule)
- [x] Hero ticker block: `★ AAPL $278.78 H/L-stacked ⓘ` — star + info SANDWICH the price block
- [x] Right column (420px): search row with magnifier + 4 action icons → Favorites + Recent dropdowns side-by-side
- [x] Hardcoded ticker buttons dropped — 10 demo tickers seeded into `favorites` table on every boot
- [x] Encyclopedia tab dropped from strip — reachable via info-icon only
- [x] Tab::all() resized 8→7; fixed `0..8` hardcoded loop in update::mod (panic on first frame)
- [x] HTML mockup artifact preserved at `docs/v11.6.a-header-mockup.html`

### v11.6.B — "Council de-astro + Munger diversification" (~0.5 day, next)

Per v11.5 review [02:55, 03:35]: *"None of these people would talk about astrology... they wouldn't bring up astrology score to make evaluation"* + *"Munger seems to constantly stay at the same place."*

- [ ] **B1** Buffett `astro_take` — rewrite to pure financial reasoning, drop "stars suggest" / "astrological alignment" prose (`agents.rs:245-262`)
- [ ] **B2** Graham `astro_take` — rewrite to drop "astrological score" / "horoscope" mentions (`agents.rs:396-404`)
- [ ] **B3** Lynch `astro_take` — rewrite to drop "astro reading" / "stars align" mentions (`agents.rs:514-528`)
- [ ] **B4** Munger `astro_take` — rewrite + diversify (currently 3 variants, needs 6 like other personas) (`agents.rs:649-662`)
- [ ] **B5** Strip astrology refs from LLM system prompts (`agents.rs:856, 863, 867, 872`)
- [ ] **B6** `fallback_no_fundamentals` — rewrite per-persona text to drop astrology mentions (`agents.rs:829-832`)
- [ ] **B7** Rename `analysis.astro_take` field display label in `fundamentals.rs` view from "On the stars:" to "Final note:" or remove entirely
- [ ] **B8** Keep astro score visible in metric rows (line 794) — that's data, not persona prose

### v11.6.C — "Natal chart sphere" (~0.5 day)

User: *"I don't know why it keeps on being oval"* [v11.5 review 00:22]

- [ ] **C1** Reduce CAMERA_TILT in `natal_wheel_3d.wgsl` from 0.32 to 0.10 (chart reads as sphere not oval)
- [ ] **C2** Update `R_NATAL` / `R_TRANSIT` overlay constants in `view/astrology_tab.rs` to match new tilt
- [ ] **C3** Verify aspect line hit zones still align after tilt change

### v11.6.D — "Calendar 3-month forward" (~0.5 day)

User: *"It should do at least three months ahead, instead of just one month"* [01:10]

- [ ] **D1** Astro Calendar `< Prev` / `Next >` buttons step ±3 months instead of ±1
- [ ] **D2** Render 3 month-grids stacked vertically OR show wider date range in single grid
- [ ] **D3** Forecast date colors continue across the 3-month window

### v11.6.F — "Lagrange chart polish" (~1 day)

User: *"This chart needs a lot of work, could look nicer. We'll work on that. We will soon have to move towards working that eventually."* [05:20]

- [ ] **F1** Add gridlines + zone bands (Optimal/Favorable/Neutral/Unfavorable/Misaligned) on Lagrange sparkline
- [ ] **F2** Date axis labels every ~15 days
- [ ] **F3** Hover crosshair + value tooltip
- [ ] **F4** Score zone color coding (line color matches current zone)

### v11.6.G — "Sparkle animation upgrade" (~0.5 day)

User: *"Make it like a little star or sparkly animation so you can see it more visibly"* [14:34]

- [ ] **G1** Increase TabSparkle alpha from 0.45 → 0.75 in loading bar overlay
- [ ] **G2** More particles (currently ~3, push to 8+) with bigger radius
- [ ] **G3** Color burst (gold + soft white particles, not single hue)

### v11.6.H — "Score gauge clarity" (~0.5 day)

User: *"Saying 50, but over here it's saying 90... why is it this number?"* [15:33, 16:54]

- [ ] **H1** Disambiguate Crypto F&G / Equities F&G / Ticker Score / Astro / Lagrange — improve gauge titles to make clear which is the headline score
- [ ] **H2** Add "what's THE score" tooltip explaining the 5 different gauge meanings
- [ ] **H3** Possibly bold/highlight the Lagrange (composite) gauge as "primary"

### v11.6.J — "Fetch stuck root cause" (~0.5 day)

User: *"When I fetched, this ended up being stuck like this. So that needs to be resolved."* [01:35]

- [ ] **J1** Add 90-second timeout to fetch task; fall back to "fetch timed out, try again" toast
- [ ] **J2** Log every Phase 1/2/3/4 step duration to identify which step hangs
- [ ] **J3** Investigate Iced 0.14 task scheduling — possibly Task::perform getting stuck on a future that never resolves

### v11.6.K — "Iced 0.14 chart hover perf" (~1 day)

User: *"It lags any time I have to go like this... not nearly as responsive as these [gauges]"* [10:25]

- [ ] **K1** Profile chart hover redraw cost — likely full Canvas re-render per mouse move
- [ ] **K2** Add `iced::widget::canvas::Cache` to PriceChart so static layers (axes, gridlines) don't redraw
- [ ] **K3** Verify hover overlay re-render only touches the crosshair + tooltip layer

---

## Closed — v11.5 "The Explanations" (~10 days, video review 2026-05-05) — COMPLETED 2026-05-05

**Source:** 27-min video review on 2026-05-05 produced ~22 distinct feedback items. Transcript at `docs/video-review-v11.4-transcript.txt`. Dominant theme: pop-up explanations EVERYWHERE (mentioned 15+ times). User wants game-tutorial-style hover + right-click expand for every cryptic abbreviation, gauge, and chart element.

**Sequencing rationale:** ordered by dependency + risk + ROI, NOT quick-wins. Foundation (helper) → Layout (high-risk, do early) → Content (tooltips applied) → Interactivity (chart hover/zoom + notifications) → New surface (Wikipedia tab) → Polish (final coat). OpenBB Wave 7 defers entirely until v11.5 ships.

### v11.5.A — "The Foundation" (~0.5 day, foundational)

Build canonical tooltip helper that subsequent phases reuse. Without this, 12 ad-hoc tooltips → inconsistent style.

- [ ] **A1** New `explain_tooltip()` helper in `view/shared.rs` — Phosphor info icon prefix + container styling matching existing `tip_style` from Wave 3b
- [ ] **A2** Right-click context menu primitive — `mouse_area` + state-tracked overlay for "Explain in detail" with optional URL link
- [ ] **A3** Document pattern in `view/shared.rs` doc comment — future tooltip additions follow this template

### v11.5.B — "The Layout" (~2.5 days, high risk)

Header reshuffle per user spec [09:25-11:24, 15:02-15:47 in transcript]. Done early so all subsequent waves build on stable layout.

- [ ] **B1** Search bar to top strip (where ticker name currently sits) [09:25]
- [ ] **B2** Ticker name + price + H/L to middle band (where search currently sits) [10:14]
- [ ] **B3** Recently-viewed → "Favorites" dropdown (dark button with caret) [10:45-10:57]
- [ ] **B4** Zodiac symbol legend strip relocated above natal chart not below [09:50]
- [ ] **B5** Market Sentiment gauges side-by-side row instead of stacked [11:35]
- [ ] **B6** Settings → modal overlay (not full tab view) [09:11]
- [ ] **B7** Verify XL text size doesn't break layout [08:51 — "this is now broken"]

### v11.5.C — "The Explanations" (~2.5 days, content-heavy)

Apply A1 helper to ~12 tooltip locations. Lowest individual risk per item; pure additive UX.

- [ ] **C1** Universe table columns — Sector, Score, Astro, Zone get tooltips matching Fin/Mac/Sht/Conc pattern [05:10-05:16, 17:24]
- [ ] **C2** Astrology Transits table A/S column header → "Applying / Separating" tooltip [08:00-08:04]
- [ ] **C3** Sector heatmap legend → tooltip explaining color scale + sector ranking
- [ ] **C4** All 5 Overview gauges (Crypto/Equities Sentiment, Score, Astrology, Lagrange) → tooltip on each label [11:51-12:13]
- [ ] **C5** FRED macro indicator labels → tooltip + optional Wikipedia/FRED link [17:24-17:48]
- [ ] **C6** Lagrange Confirm/Deny/Divergence labels → click to expand reasoning [21:02-21:13]
- [ ] **C7** Backtest results context → "is 50% good?" tooltip with benchmark comparison [20:36]
- [ ] **C8** Council verdict labels (StrongBuy/Buy/Hold/Sell/StrongSell) → tooltip explaining persona's threshold

### v11.5.D — "The Interactions" (~2.5 days, isolated risk)

Chart interactivity + system-level notifications. Highest technical risk in v11.5 (aspect line hit-testing).

- [ ] **D1** Aspect line hover/click — currently rendered in WGSL shader (no hit-testing). Either: (a) keep shader + add invisible Iced overlay rectangles for click zones along each aspect line, OR (b) move aspect lines from shader to Iced Canvas widget. Approach (a) is less work but approximate; (b) is cleaner but bigger refactor. Decide during build. [07:21-07:31, 07:12]
- [ ] **D2** Mouse-wheel zoom on natal wheel — Iced 0.14 lacks built-in pinch but `on_scroll` callback workable. Increment `chart_size` enum scale factor in 0.1 steps, clamped 0.5x-2.0x. [06:48-07:00]
- [ ] **D3** OS-level toast notifications for Lagrange Optimal/Misaligned alerts. Add `notify-rust` crate. Trigger from `Message::AlertFired`. [06:06]
- [ ] **D4** Verify alert rendering on multiple monitors / Windows notification settings

### v11.5.E — "The Encyclopedia" (~1.5 days, new feature)

NEW tab pulling Wikipedia data per ticker. Fully independent of A-D.

- [ ] **E1** New `src/scraper/wikipedia.rs` — REST API at `https://en.wikipedia.org/api/rest_v1/page/summary/{title}`. Match company name from `company_metadata` to Wikipedia title via fuzzy lookup or stored Wikidata Q-ID (already have wikidata_enrich data).
- [ ] **E2** Migration `0043_wiki_summary.sql` — table `wiki_summaries` with ticker, summary_text, image_url, last_fetched, wiki_url
- [ ] **E3** New tab `Tab::Encyclopedia` (or rename existing — user open to placement). Phosphor BOOK_OPEN icon.
- [ ] **E4** New `view/encyclopedia.rs` — image (if available), description, key facts table, "Read more on Wikipedia →" link
- [ ] **E5** Wire into refresh pipeline — staleness check (30-day TTL since articles change rarely)
- [ ] **E6** Cache miss handling — show "No Wikipedia article found" placeholder

### v11.5.F — "The Polish" (~1 day, final coat)

Polish items that benefit from being last (less rework risk).

- [ ] **F1** Candle tooltip full-word labels: "Opening / High / Low / Closing / Volume" [14:30-14:40]
- [ ] **F2** Volume label gets tooltip explaining "shares traded that day" [14:48-14:52]
- [ ] **F3** Loading bar shows numeric % alongside fill [14:11]
- [ ] **F4** Loading bar adds sparkle particles overlay [13:38-13:42]
- [ ] **F5** Stuck-at-85% fix — when scraper takes >30s, advance to 90% with "still fetching..." text instead of frozen [14:22]
- [ ] **F6** Dedupe: when fetch is in progress, only show ONE loading indicator (currently can show both bar + spinner) [13:58]
- [ ] **F7** Strategy backtest defaults — pre-fill Buy/Sell threshold inputs based on ticker's actual astro distribution (median ± stddev), not generic 65/35 [25:05-25:15]

---

## Open — Wave 6: Data Reliability + Astro Depth (~900 lines, medium risk)

**Theme:** Pair Track A (financial data) + Track B (astrology engine) so Concordance metric strengthens on both sides. 8 items, 4 sub-waves.

### Wave 6.0 — "The Reliability" (highest impact pair) — SHIPPED 2026-05-04

- [x] **6.A1 Multi-source price fallback** — Yahoo Finance v8 chart API + Stooq CSV cascade. `data_source` column on `price_data`. AV→Yahoo→Stooq order. Migration `0038`.
- [x] **6.B1 Aspect pattern recognition** — Grand Trine, T-Square, Grand Cross, Yod, Mystic Rectangle, Stellium, Kite (7 patterns). `aspect_patterns` JSONB column on `astro_scores`. Cross-chart detection (natal+transit mix). 7/7 unit tests passing. Migration `0037`.

### Wave 6.1 — "The Precision" — SHIPPED 2026-05-04

- [x] **6.A2 Multi-source fundamentals fallback** — `SourcedFundamentals` normalized struct, FMP→Finnhub `/stock/metric?metric=all`→AV `OVERVIEW`. `data_source` column on `fundamental_metrics`. Migration `0039`.
- [x] **6.B2 Aspect strength model** — `body_weight()` table (Sun/Moon=1.5, Jup/Sat=1.3, outers=1.4, inners=1.0, nodes/Chiron=0.8), `mutual_reception_bonus()` (1.15× when bodies in each other's domiciles), `out_of_sign_modifier()` (0.75× when sign-distance mismatches aspect-distance). New `score_aspect_v2()` integrates all multipliers; orb-tightness, applying/separating, dignity already existed pre-Wave 6. 18/18 aspect tests passing.

### Wave 6.2 — "The Depth" — SHIPPED 2026-05-04

- [x] **6.A3 Analyst price targets** — Finnhub `/stock/price-target` endpoint, new `analyst_targets` table (low/median/high/n_analysts), 7-day staleness check, single-ticker + universe-wide variants. Migration `0040`. Earnings calendar already existed via existing finnhub.rs.
- [x] **6.B3 Fixed stars + Arabic Parts** — 8 stars (Regulus/Spica/Antares/Aldebaran/Sirius/Vega/Fomalhaut/Algol) with hardcoded J2000 longitudes + linear precession (~0.014°/year). 1° conjunction orb. Algol carries negative strength (-14). Arabic Parts: Fortune, Spirit, Commerce, Substance computed from ASC + Sun + Moon + Mercury. NatalChart gains `ascendant: Option<f64>` field. Both add to delta_sum pre-sigmoid. 5 new tests passing (precession, conjunction detection, orb edge, Moon skip, Algol negative, Fortune formula, transit-conjunct-Fortune).

### Wave 6.3 — "The Trust" — SHIPPED 2026-05-04

- [x] **6.A4 Data freshness UI badges** — `data_freshness` SQL view aggregating 5-source completeness (prices/fundamentals/news/sentiment/astro) per ticker. `fresh_count: Option<i32>` added to UniverseRow. Universe table badge column rendering ●●●●○ with zone-color tinting (5=optimal, 3-4=favorable, 2=neutral, ≤1=misaligned). Migration `0042`.
- [x] **6.B4 Eclipse cycles + Saros series** — Hardcoded NASA eclipse catalog 2025-2028 (17 eclipses) in both `eclipses.rs::upcoming_eclipses()` const + migration `0041` seed. Activations: natal planets within 6° of any upcoming (12mo) or past (6mo echo) eclipse. Solar negative-strength (-10), lunar (-6), tightness + time-fade scaling. Saros series number stored. 5/5 unit tests passing.

## Open — Future

### House numbers on natal chart
**What:** Add house numbers (1-12) to the 3D natal chart.
**Approach:** Iced 0.14 `pin` overlay (same as planet symbols). No shader text needed.

### Real-time fetch progress bar
**What:** Parse scraper stdout line-by-line to show real progress %. Currently time-based estimate (capped at 0.85).
**Approach:** Iced 0.14 Animation API + subscription stream from subprocess stdout.

### Harmonic charts (H4, H5, H7, H9)
**What:** Compute `(longitude × N) mod 360` derivative charts, run aspect detection on harmonic positions.
**Why:** Reveals patterns invisible in natal chart. H4 = manifestation, H7 = inspiration.
**Deferred from Wave 6** — pure math, no new dependencies, but UI surface unclear.

### Sidereal vs Tropical concordance
**What:** Run aspect detection in both Tropical (current) and Sidereal (`SE_SIDM_LAHIRI`). Surface agreement as new metric.
**Why:** Two-system confirmation = stronger signal. Disagreement = ambiguity, smaller position size.
**Deferred from Wave 6** — depends on B2 strength model first.

### Progressed charts (Secondary + Solar Arc)
**What:** "1 day = 1 year" progressions for long-horizon forecasts.
**Why:** Current 90-day forecast is purely transit-based. Progressions add slow-unfolding layer.

## Open — Infrastructure

### docker-compose.yml for local PostgreSQL
**What:** `docker-compose.yml` to start PostgreSQL. 10-line file, `POSTGRES_PASSWORD=dev`, port 5432.
**Blocked by:** User wants to explore Docker first.

### OpenBB Platform integration — PROMOTED to Wave 7 (2026-05-04)
**Decision (revised 2026-05-04):** Pure-Rust path (Path C from approach analysis). Two phases:
- **Phase 1 (Wave 7) — 10 native Rust provider scrapers**, ~8-10 days
- **Phase 2 (Wave 8, conditional) — Rust sidecar mimicking OpenBB Workspace contract**, ~7 days

No Python dependency. No `openbb` package install. We pick datasets we want, hand-write Rust HTTP wrappers in `src/scraper/sources/` matching existing pattern. Phase 2 only happens if Phase 1 data proves worth presenting via Workspace cloud UI.

See `docs/research-wave7-openbb.md` for full plan.

## Open — Wave 7: Native Rust Provider Library ("The Library")

**Theme:** Add 10 free, high-value data providers as native Rust scrapers. Cherry-pick the datasets that close real gaps in our 20-module existing scraper. 5 sub-waves, ~8-10 days total.

**Selection criteria:** (1) free, (2) closes real gap, (3) plausibly affects scoring, (4) low integration effort.

### Wave 7.0 — "The Macro Foundation" (~2-3 days)
International + sovereign macro. Closes our biggest data gap (FRED is US-only).
- [ ] **7.0.1** `src/scraper/world_bank.rs` — `https://api.worldbank.org/v2/country/{code}/indicator/{id}?format=json`. 1500+ indicators, 200+ countries.
- [ ] **7.0.2** `src/scraper/imf.rs` — `https://www.imf.org/external/datamapper/api/v1/...`. Sovereign macro + IMF forecasts.
- [ ] **7.0.3** `src/scraper/ecb.rs` — `https://data-api.ecb.europa.eu/service/data/...`. EU monetary policy + EUR FX.
- [ ] **7.0.4** Migration `0043_intl_macro.sql` — unified `intl_macro_indicators` table (country_code, indicator_code, date, value, data_source).
- [ ] **7.0.5** Wire into scraper pipeline phase 2.4b (after FRED).
- [ ] **7.0.6** Dashboard surface — extend Research tab with international macro section.

### Wave 7.1 — "The Sentiment Layer" (~2 days)
Futures positioning sentiment. Qualitatively new signal type.
- [ ] **7.1.1** `src/scraper/cftc_cot.rs` — weekly CSV download from `cftc.gov`. Parse commercial vs speculator net positions.
- [ ] **7.1.2** Migration `0044_cftc_cot.sql` — `cftc_positioning` table (commodity, report_date, commercial_net, large_spec_net, small_spec_net).
- [ ] **7.1.3** Wire into scraper pipeline phase 2.5b.
- [ ] **7.1.4** New Research tab section "Futures Positioning" — gold, oil, SPX, treasuries.

### Wave 7.2 — "The Labor + Energy" (~2 days)
US sector-level signal sources.
- [ ] **7.2.1** `src/scraper/bls.rs` — `https://api.bls.gov/publicAPI/v2/timeseries/data/`. Free tier 25 queries/day, 50 series/query. Detailed CPI components, sector employment, productivity.
- [ ] **7.2.2** `src/scraper/eia.rs` — `https://api.eia.gov/v2/`. Energy data: oil/gas prices, production, inventories. Drives energy/utility tickers.
- [ ] **7.2.3** Migration `0045_bls_eia.sql` — `bls_series` + `eia_series` tables.
- [ ] **7.2.4** Wire into pipeline + Research tab.

### Wave 7.3 — "The Stress Index" (~1 day)
Single high-signal composite metric.
- [ ] **7.3.1** `src/scraper/ofr.rs` — Office of Financial Research. Financial Stress Index, money market data. Single endpoint, single number.
- [ ] **7.3.2** Migration `0046_ofr_stress.sql`.
- [ ] **7.3.3** Surface in Universe tab as new column or in macro overview.

### Wave 7.4 — "The Treasury + Crypto" (~2 days)
Round out coverage with two more high-value providers.
- [ ] **7.4.1** `src/scraper/treasury_direct.rs` — `https://api.fiscaldata.treasury.gov/services/api/...`. US Treasury auctions, debt-to-the-penny, yield curve raw.
- [ ] **7.4.2** `src/scraper/coingecko.rs` — `https://api.coingecko.com/api/v3/...`. Crypto prices/market data, 30/min free. Optional but adds asset class.
- [ ] **7.4.3** Migration `0047_treasury_crypto.sql`.
- [ ] **7.4.4** Pipeline + display.

### Wave 7 totals
**10 providers**: World Bank, IMF, ECB, CFTC, BLS, EIA, OFR, Treasury Direct, CoinGecko, plus 1 buffer slot.
**5 migrations**: 0043-0047.
**Estimate**: 8-10 days.

---

## Open — Wave 8 (CONDITIONAL): Rust Sidecar + OpenBB Workspace ("The Showcase")

**Theme:** Build Rust HTTP server (axum) mimicking OpenBB Platform's API contract. OpenBB Workspace cloud UI connects to it as if it were OpenBB. Gives us polished dashboards + sharing for Pursuit demo day. ~7 days.

**Decision gate:** Only ship Wave 8 IF Wave 7 data is worth presenting via Workspace AND Pursuit Fellowship needs polished shareable dashboards. Validate before committing.

### Wave 8.0 — "The Scaffold" (~1 day)
- [ ] **8.0.1** Cargo workspace `[[bin]]` target for `workspace` binary at `src/workspace/main.rs`.
- [ ] **8.0.2** axum scaffold serving `localhost:7100`.
- [ ] **8.0.3** `tower-http` CORS layer allowing `https://pro.openbb.co`.
- [ ] **8.0.4** PAT bearer auth middleware (env var `WORKSPACE_PAT`).
- [ ] **8.0.5** Read-only Postgres role + sqlx connection pool.

### Wave 8.1 — "The Contract" (~1 day)
- [ ] **8.1.1** Implement `GET /widgets.json` returning widget catalog.
- [ ] **8.1.2** Implement `GET /apps.json` for app definitions.
- [ ] **8.1.3** First widget endpoint: `GET /lagrange-scores?ticker=X` returning array of `{ticker, score_date, score, label}`.
- [ ] **8.1.4** Verify Workspace can render the widget end-to-end.

### Wave 8.2 — "The Widget Set" (~2 days)
Add core widgets exposing our proprietary data.
- [ ] **8.2.1** `/aspect-patterns?ticker=X` (Wave 6.B1)
- [ ] **8.2.2** `/eclipse-activations?ticker=X` (Wave 6.B4)
- [ ] **8.2.3** `/fixed-stars?date=D` (Wave 6.B3)
- [ ] **8.2.4** `/universe?zone=X&sector=Y` (paginated universe table)
- [ ] **8.2.5** `/data-freshness?ticker=X` (Wave 6.A4)
- [ ] **8.2.6** `/lagrange-history?ticker=X` (line chart series)

### Wave 8.3 — "The Visuals" (~1 day)
Chart-type widgets, not just tables.
- [ ] **8.3.1** Lagrange-over-time line chart widget.
- [ ] **8.3.2** Sector heatmap widget.
- [ ] **8.3.3** Aspect pattern timeline gantt-style.

### Wave 8.4 — "The Tunnel" (~1 day)
- [ ] **8.4.1** ngrok install + auth token + tunnel `localhost:7100`.
- [ ] **8.4.2** OpenBB Workspace account + PAT generation.
- [ ] **8.4.3** Workspace settings → backend URL = ngrok URL → header `ngrok-skip-browser-warning: true`.
- [ ] **8.4.4** Verify all widgets load in Workspace.

### Wave 8.5 — "The Dashboard" (~1 day)
- [ ] **8.5.1** Build one polished Workspace dashboard combining 6+ widgets.
- [ ] **8.5.2** Screenshot for Pursuit portfolio.
- [ ] **8.5.3** Document in `docs/openbb-workspace-rust.md` — setup, deployment, architecture.

### Wave 8 totals
**~7 days**, ~3000 LOC new Rust, 0 new external dependencies (axum/sqlx/tokio already in tree).
**Conditional ship gate**: validate after Wave 7 ships.

## Open — API Keys Backlog (Wave 6 deferred sources)

Decision 2026-05-04: Wave 6 ships with **free-tier only** (existing AV/FMP/Finnhub/Tiingo + scraping Yahoo/Stooq). Paid keys deferred until validated need.

### Polygon.io — real-time + options
**What:** Real-time prices, options chains, intraday data, news.
**Free tier:** 5 req/min — too tight for daily scraper.
**Paid:** $29/mo Stocks Starter (unlimited 15-min delayed). $79/mo Real-time.
**Why deferred:** Would unlock options-data feature surface (currently zero options data). Reconsider when we want intraday or options chains in the dashboard.

### EODHD — fundamentals (EU + US)
**What:** Different fundamentals universe than FMP, especially strong on EU/Asia tickers.
**Free tier:** 20 req/day — fundamentals universe-wide impossible.
**Paid:** $19.99/mo (100k req/day).
**Why deferred:** FMP already covers our US watchlist. EODHD only matters when we expand to international tickers.

### Quandl/Nasdaq Data Link — alt data
**What:** High-quality macro indicators, alternative datasets, sometimes free academic datasets.
**Free tier:** 50/day — workable for low-frequency macro pulls.
**Paid:** Per-dataset pricing, varies wildly.
**Why deferred:** FRED + DBnomics already cover our macro needs. Reconsider for niche datasets (CFTC, commodity inventories).

### Marketstack — tertiary price fallback
**Free tier:** 1,000/mo — only ~33/day, not useful as primary or secondary.
**Paid:** $9.99/mo (10k/mo).
**Why deferred:** Tiingo + Yahoo + Stooq already give us 3 fallback layers. Marketstack only worth adding if those prove unreliable.

### Polygon News (separate from Polygon Stocks)
**What:** Aggregated news with full article bodies + sentiment scores.
**Why deferred:** Finnhub news + RSS scraping already cover headlines. Article-body access only matters if we add full-text NLP analysis.

## Completed (v6.0-v11.3)

- **v11.3: "The Refinement"** — All 22 video-review feedback items shipped across 5 waves:
  - Wave 1: aspect line thickness, section icons, compact horoscope, scrollbar gutter, galaxy mute
  - Wave 2: header price/H/L, search above ornament, astrology tab reorder, two-column compression
  - Wave 3: sector dropdown, column tooltips, rising sign backfill, per-ticker fetch, planet symbol overlay, hover tooltips
  - Wave 4: council template diversification, chart size enum, fetch progress bar
  - Wave 5: tooltip size setting, scraper retry helper, gauge compass-rose, parchment fiber texture
- **v11.2: "The Foundation"** — Iced 0.13→0.14 framework upgrade (13+ files, 19 breaking API changes: Pipeline trait, wgpu 27, canvas Action, widget renames, application boot). PowerShell gcc PATH fix.
- **v11.1: "The Craft"** — clickable entity links, tab glow bookmark, chart layer toggles, nav redesign
- **v11.0: "The Intelligence"** — 90-day forecast, Big Three summary, smart calculator defaults, zodiac legend, loading shimmer
- **v10.0: "The Signal"** — RSS tone sentiment, Lagrange adaptive weighting, richer agent verdicts
- **v9.3: "The Clarity"** — aspect line contrast, column widths, tab labels bold, scrollbar gutter
- **v9.2: "The Cosmos"** — galaxy background, active tab gold glow + sparkle
- **v9.1: "The Polish"** — backtest crash fix, broken icons, tooltip contrast, disable chart rotation
- **v9.0: "The Performance"** — 9 animation items: planet pulse, aspect shimmer, dust motes, candle draw-in
- **v8.0: "The Observatory"** — 3D natal chart GPU shader (procedural SDF, perspective tilt, 496-byte uniforms)
- **v7.6: "The Consistency"** — gold scrollbars, canvas sparkle, animated transit ring
- **v7.3: "The Grimoire"** — game-book UI, right-side tabs, Canvas ornaments, page transition fade
- **v7.2: "The Motion"** — Phosphor Icons, animation infrastructure, gauge sweep, responsive font scaling
- **v7.1: Spatial polish** — compact header, 1240px max-width, eyebrow labels, Inter font numerics
- **v7.0: Renaissance book UI/UX overhaul** (Parchment/Leather themes, 24-stage circadian)
- **v6.0-6.2: Paper trading engine** ($100K virtual capital, equity curve, NYSE holidays)
