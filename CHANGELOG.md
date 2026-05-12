# Changelog — Nisaba Capital Charting

**Author:** Aisling Leiva
**Stack:** Rust 2021 · Iced 0.14 (wgpu) · SQLx · PostgreSQL · Swiss Ephemeris · axum · windows-rs
**Development:** 2026-04-07 to 2026-05-12 (capstone build + public publish)
**Latest release:** [v12.2.0 — Capstone Demo](https://github.com/nessaisling-lab/nisaba-capital-charting/releases/tag/v12.2.0)
**Landing page:** [nessaisling-lab.github.io/nisaba-site](https://nessaisling-lab.github.io/nisaba-site/)

---

## Versioning note

Two parallel version tracks ran during development:

- **Cargo semver** (`Cargo.toml`): v11.0.0 (long-stable through April–May development) → v12.1 (toast triage, 2026-05-11) → v12.2 (Nisaba rebrand, 2026-05-11). These are substantive infra/identity changes.
- **Wave release-train** (commit labels v13.0, v13.1, v13.2): UI/feature polish cycles. Wave version stayed ahead of Cargo because Cargo was bumped only when project identity or infrastructure shifted materially.

The two tracks are orthogonal dimensions (infra version vs UI polish wave), not contradictory. The **v12.2.0 git tag** marks the formal public publication on 2026-05-12 — that's the canonical release identifier.

The project was previously named **"Pursuit Astro"** (engine codename) / **"Charting Capital"** (consumer brand). Both retired in the v12.2 rebrand for a unified brand-family architecture under **Nisaba Capital Charting**.

---

## 2026-05-12 — v12.2.0 PUBLIC SHIP

The capstone demo went live. First public release of Nisaba Terminal.

**What shipped:**
- GitHub Release **v12.2.0** on `nessaisling-lab/nisaba-capital-charting` (public repo) with the `NisabaCapitalCharting-Setup.exe` Inno Setup installer attached as a release asset (18 MB · per-machine install).
- Public landing page at [nessaisling-lab.github.io/nisaba-site](https://nessaisling-lab.github.io/nisaba-site/) hosted via GitHub Pages from the new `nisaba-site` repo (~600 lines of HTML+CSS reusing the v13.2 pitch-deck design tokens).
- Release notes attached as `installer/Output/release-notes-v12.2.0.md` (markdown rendered on the Releases page).

**Ship pipeline:** `git tag -a v12.2.0` → push tag → `gh release create v12.2.0 <installer.exe> --notes-file <notes.md>` → `gh repo create nisaba-site --public --source=. --push` → `gh api ... pages` enable. Roughly 45 seconds from auth to all-live.

**Validation:**
- Release page renders the markdown notes correctly + 18 MB installer downloads from CDN
- Site repo's `main` branch pushed (2 commits: initial landing + features/roadmap/FAQ/disclaimer additions)
- GitHub Pages workflow enqueued and built within ~60s of API enable

**What is NOT in this ship:** demo video (placeholder section on landing page, recording planned this week), v12.3 per-user `.env` loader (current install copies `.env` to install dir as workaround), code-signed installer (SmartScreen warns on first run).

---

## v12.2.A — Rebrand stragglers (2026-05-11)

Two stragglers caught after the v12.2 rebrand sweep:

1. **Iced window title** in `src/dashboard/main.rs` still read `"Financial Dashboard"` (placeholder from v1.0, never updated). Bulk find/replace didn't catch it because it wasn't a brand keyword. Changed to `"Nisaba Terminal"` so Alt-Tab and Task Manager identify the app correctly.
2. **Inno Setup `{pf}` deprecation**: `installer/nisaba-capital-charting.iss` used the legacy `{pf}` constant. Inno Setup 6 emits a deprecation warning. Switched to `{commonpf}` (explicit per-machine 64-bit Program Files). Recompile is now warning-free.

**Validation:**
- Inno Setup recompile: 0 warnings, 29.8 sec, 18.08 MB output

---

## v12.2 — "Nisaba Capital Charting" rebrand + brand-family architecture (2026-05-11)

**Theme:** Project rename from "Pursuit Astro" / "Charting Capital" → **"Nisaba Capital Charting"**. Adopting a brand-family architecture inspired by mature fintech product lines (Bloomberg L.P. → Bloomberg Terminal → Bloomberg Mobile; Apple → iPhone → A17 Pro).

Nisaba was the Sumerian goddess of writing, accounts, grain stores, and patron of scribes — fitting origin for a financial-astrology record-keeping terminal.

### Brand architecture

| Layer | Name |
|---|---|
| Parent brand (company / umbrella) | **Nisaba Capital Charting** |
| Codename (Rust core, shared) | **Nisaba Engine** |
| Desktop product (this binary) | **Nisaba Terminal** |
| Mobile (planned, v13.x) | Nisaba Scribe |
| Web SaaS (future) | Nisaba Atlas |
| Public API (future) | Nisaba Codex |
| Research newsletter (future) | Nisaba Almanac |

### Code / identifier changes

- `Cargo.toml`: `name = "nisaba_engine"`, `version = "12.2.0"`
- AUMID: `"NisabaCapitalCharting.Terminal"`
- App display name: `"Nisaba Terminal"`
- Scraper User-Agents: `"NisabaEngine/0.1"` (engine codename for technical headers)
- All `use pursuit_week4_automation::*` → `use nisaba_engine::*` imports
- Bulk PowerShell find/replace across 36 src/**/*.rs files
- `helpers.rs` `.lnk` filename now derived from `APP_DISPLAY_NAME` constant via `format!("{APP_DISPLAY_NAME}.lnk")` — future renames stay in sync

### Docs / artifact changes

- README / DESIGN: umbrella brand "Nisaba Capital Charting"
- `docs/v13.2-{pitch-deck.html, elevator-pitch.md, demo-video-script.md}`: full rebrand
- `docs/v13.2-pitch-deck.pptx`: regenerated from updated builder
- `scripts/build_v13.2_pptx.py`: brand strings updated
- `installer/pursuit-astro.iss` → `installer/nisaba-capital-charting.iss`: rewritten + flipped from per-user to **per-machine** install scope

### Installer scope flip

| | Old (v11.x) | New (v12.2) |
|---|---|---|
| `DefaultDirName` | `{userappdata}\Pursuit Astro` | `{commonpf}\Nisaba Capital Charting\Nisaba Terminal` |
| `PrivilegesRequired` | `lowest` | `admin` (install-time only) |
| Registry scope | HKCU | HKLM (all-users AUMID) |
| Icons | `{userdesktop}` + `{group}` | `{commondesktop}` + `{commonprograms}` |

**Rationale:** developer bounces between two Windows accounts (NessA admin, Aisling standard user). Per-machine install means all accounts share the binary + AUMID registration. App still runs unelevated in each user's session (critical for toast delivery — see v12.1).

### Intentional exclusions

- `CHANGELOG.md` historical entries: preserved (accurate brand history)
- `deprecated_docs/`: preserved (already marked deprecated)

### Validation

- `cargo build --bin dashboard`: clean compile as `nisaba_engine v12.2.0`
- `cargo run --bin dashboard` from Aisling's non-elevated session:
  - `[fire_toast] WinRT toast shown — summary: ACMR → Optimal (+26 more)`
  - Action Center toast displays "Nisaba Terminal" as app name
- Orphan shortcuts cleaned across both NessA + Aisling profiles

---

## v12.1 — "The Toast Triage" — Win11 24H2 toast notification fix (2026-05-11)

**Theme:** Lagrange alert toasts were failing with `HRESULT 0x80070005 (E_ACCESSDENIED)` on Win11 24H2 despite the v12.0.A AUMID-bound shortcut + registry entry. Three compounding code-side issues plus one environmental constraint.

### v12.1.A — Process AUMID stamp

`notify-rust`'s `Notification::app_id()` only stamps the toast XML — it does NOT call `SetCurrentProcessExplicitAppUserModelID`. Win11 24H2's notification broker checks the **process** AUMID before the shortcut AUMID. Without a process stamp, the broker rejects before looking up the shortcut.

Added `unsafe fn set_current_process_aumid()` called at the top of `register_app_user_model_id()` via `windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID`.

### v12.1.B — SHChangeNotify + 300ms settle

Explorer's in-process AppResolver cache (Start-Menu → AUMID lookup) doesn't auto-refresh when a new `.lnk` is written via `IPersistFile::Save`. First toast attempts hit a stale cache miss.

Added `broadcast_shell_change()` that fires `SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST | SHCNF_FLUSH, ...)` after shortcut creation, followed by a 300ms sleep.

### v12.1.C — Direct WinRT toast call

`notify-rust` 4.x → `winrt-notification` path is brittle on 24H2 even with A+B in place. Direct WinRT calls via `windows::UI::Notifications::ToastNotificationManager::CreateToastNotifierWithId` succeed where the wrapper does not.

Added `fire_toast_winrt()` that builds toast XML with the `ToastGeneric` template + optional `scenario="urgent"` for Optimal alerts. `Cargo.toml` added `UI_Notifications` + `Data_Xml_Dom` features. `notify-rust` retained for cross-platform (Linux/macOS) via `cfg(not(windows))`.

### Environmental requirements (no code can fix these)

- Process must NOT be elevated. Elevated processes run in separate security context from interactive desktop session → broker rejects.
- Process must run as SAME user signed into active console desktop. Cross-user toast calls fail with same HRESULT.

The per-app permission entry at `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\<AUMID>` is auto-created by Windows on first successful `Show()` — manual seeding not required when environment is correct.

### Validation

Tested end-to-end on Win11 24H2 from Aisling's non-elevated session: 27-alert Lagrange backfire on dashboard startup, log shows `WinRT toast shown — summary: ACMR → Optimal (+26 more)`, toast renders in Action Center.

---

## v13.2 — "The Showcase" — Capstone presentation prep (shipped 2026-05-06)

**Theme:** Five artifacts targeting Pursuit Fellowship final demo + investor conversations + LinkedIn-tier discoverability.

### v13.2.A — README rewrite

Hero pitch + 60-sec elevator + verified historical-precedent table + features + quickstart + architecture diagram + AAPL validation refs. Citation discipline applied (Morgan quote hedged, disputed Adams 1929 prediction omitted, Quigley/Reagan omitted).

### v13.2.B — Demo video script (3-min walkthrough)

6-section script with timestamps + voiceover lines. Pre-record checklist, production tips, format-specific cuts (YouTube short, LinkedIn 3-min, conference talk, capstone). Reuses Aisling's signature voice.

### v13.2.C — Pitch deck (12-slide HTML, ~5 min)

12 slides with time markers + speaker-cue strips + embedded screenshots from v98 silent recording. Cover · Lineage · What it is · 4 walkthroughs · +37% Proof · Thesis · Numbers · Vision · Close.

### v13.2.D — Elevator pitch toolkit

60s / 30s / 15s versions + 4 audience variants (engineers / finance / fellowship judges / laypeople) + Q&A drilled responses + delivery notes.

### v13.2.G — PowerPoint export

`scripts/build_v13.2_pptx.py` python-pptx builder mirroring the HTML deck. Generates `docs/v13.2-pitch-deck.pptx` (2.95 MB, 12 slides, 16:9 widescreen).

---

## v13.1 — "Polish corrections" (shipped 2026-05-06)

**Theme:** Fix v13.0 regressions per video v96 review + bell visual + restore inline shooting star + Astrology layout revert per v97. 8 sub-items. User v97 verdict: *"Overall huge improvement. Everything's on with exceptions of the lack of consistency in regards to the gutter boundary."* (Gutter font-scale-aware fix deferred to v13.2.)

### v13.1.1 — Forecast color reverted + calendar 5-band palette
v13.0.B2 mistake: changed unfavorable forecast windows from MISALIGNED red to UNFAVORABLE orange. User v96: *"Put these back to red."* Reverted. Plus added 5-band stepped palette to Astro Calendar (`AstroCalendar::score_to_color`) — punchy red below 25, orange 25-35, neutral 35-50, green 50-70, saturated green 70+.

### v13.1.2 — Lifecycle text contrast fix
v13.0 used cream-on-cream colors for Lifecycle SR aspect lines + return countdowns + progressed Sun line. User v96: *"this is so washed out... has to be resolved."* Switched to `theme::palette().ink` (primary) and `theme::palette().ink_soft` (secondary). Theme-aware so both Parchment and Leather render readable.

### v13.1.3 — Scrollbar gutter (partial)
Reverted v13.0.B3 page-level 28px right padding — made it worse per v96. New approach: `tab_bar` container gets explicit 16px right padding so bell/gear clear scrollbar overlay zone regardless of font scale. (v97 confirmed this row works; v13.1.7 extends to compact_nav header.)

### v13.1.4b — Canvas bell with rocking ring (Option C)
v95+v96 confirmed Phosphor BELL codepoint `\u{e0ce}` renders as 4-bar hamburger glyph in our shipped TTF, not a bell. User picked Option C from `mockups/v13.1-bell-options.html` (5 options shown). New `BellIcon` Canvas widget in `ornaments.rs` — bell silhouette via 3 stroke paths (body bezier, handle, clapper arc), 2.4s rocking animation when active alert count > 0 (still 70% / damped oscillation 30% per cycle).

### v13.1.5 — Inline shooting star fetch animation restored
User v96: *"I miss my little star shooting across the border here."* Restored as fixed-height (18px) row below the page header. Always renders the row — empty Space when idle, ShootingStar canvas + "Fetching {ticker} ({secs}s)" label when fetching. Fixed height = no push-down (the original v12.1 motivator). Animation tick subscription extended to keep shader_time advancing during fetch.

### v13.1.6 — Astrology layout revert (Shrink + Fill)
v13.0.A2 tried `FillPortion(5/4)` on wheel + transits columns to fix "graphical overlap." Wrong root cause — change centered all content. User v97: *"you shifted everything toward the center where the birth chart is — I want it to go back to the left side."* Reverted: `wheel_col` Shrink-default (natural natal-wheel width), `transits_col` `Length::Fill` (consumes remaining width).

### v13.1.7 — Header gutter (compact_nav right padding)
User v97: *"as you get smaller, the further it goes under the gutter for the scroll bar."* Added 16px right padding to `compact_nav` matching the tab strip's pattern. PARTIAL FIX — works at fixed Compact size, but the issue is that action_icons are right-aligned via `Length::Fill` spacer. At smaller font scales the spacer grows proportionally less than the icons need. **Real fix deferred to v13.2** — needs font-scale-aware right padding (`SPACE_LG * font_scale()` or similar dynamic value).

### Validation

- Release build clean
- All tests pass

### Deferred to v13.2 polish

- Font-scale-aware gutter for compact_nav (current 16px not enough at Compact size)
- v13.0.D2 forecast section content expansion
- Research tab tooltips ("explanations everywhere" recurring theme)
- Universe column tooltip re-verify

---

## v13.0 — "Polish & Performance" (shipped 2026-05-06)

**Theme:** First polish wave after Wave 9 + 9.5 + 9.6 engine completion. Driven by video v95 review (12-min self-review). User confirmed engine is shipped + working — directive: "polish and performance phase."

### v13.0.A1 — Lifecycle cache (biggest perf win)
Astrology tab was triggering ~6 Swiss Ephemeris computations per render frame because `build_lifecycle_section()` ran the full Solar Return Newton search + 3 planetary return scans + secondary progression cast on every view tick. At 60fps animation that's 360 Swiss Eph calls/sec while the tab is visible — root cause of reported lag.

Fix: new `LifecycleSnapshot` cached in `state.lifecycle_cache: Option<LifecycleSnapshot>`. Built once via `rebuild_lifecycle_cache()` in update/astro.rs when both natal IPO + positions arrive. View reads pre-built strings — zero per-render compute. Invalidated on TickerSelected.

User v96 follow-up: *"the performance on the astrology tab is going really well. Perfect."*

### v13.0.A2 — Wheel + transits FillPortion split
Wheel column had no width constraint; transits column had Length::Fill. Caused overlapping graphical bugs at certain font scales. Fix: explicit `FillPortion(5)` for wheel, `FillPortion(4)` for transits.

### v13.0.B1 — Bell glyph size up
Bell button used `text_md` size; per user "I don't know why that's not a bell icon." Bumped to `text_lg` for visual prominence. NOTE: codepoint still wrong — confirmed in v96 video, regressed to v13.1.

### v13.0.B2 — Forecast color logic (LATER REVERTED)
Changed unfavorable forecast windows from MISALIGNED red to UNFAVORABLE orange. User v96 follow-up: misunderstood — they wanted forecast to STAY red AND extend red marking to calendar. Reverted in v13.1.1.

### v13.0.B3 — Scrollbar gutter 20→28px (LATER REVERTED)
Tried defensive bump of right padding. User v96 follow-up: made it worse, more sizes broken. Reverted in v13.1.3.

### v13.0.C1 — Year-of-Lord tooltip
Wrapped the `Year of Venus (10th house · Libra)` badge in a tooltip with full plain-English explanation: profection method, age, house number/sign/ruler, house theme, lord flavor, and the +50% Lagrange weight callout. Renders below the badge on hover, max 420px width.

### v13.0.C2 — Persistent notification counter
Bell button now always shows a numeric badge (active deque + history total, capped 99). Bright gold-on-black pill when active count > 0, dim cream when only history. Chrome-level signal independent of pill stack TTL.

### v13.0.D1 — Toast expiry on animation ticks
`expire_toasts()` was only called on the 30-second data-refresh tick path, not the animation tick. So during animation traffic, toasts stuck around far past their 4-second TTL. User v95 review: *"this is not going away."* Fix: added expire_toasts to animation tick.

### Validation

- 132/132 lib tests + 8/8 backtest tests pass
- Release build clean
- Astrology tab perf restored (user-confirmed in v96)

### Regressions ID'd (fixed in v13.1)

- Bell codepoint still wrong (v95 + v96 confirmed)
- Forecast color reversal (B2 misunderstood)
- Scrollbar gutter direction wrong (B3)
- Lifecycle text washed out at certain themes (cream-on-cream)
- Shooting star fetch animation removed in v12.1, user wants it back

---

## Wave 9.5 — "UI integration" (shipped 2026-05-06)

**Theme:** Surface every Wave 9 engine layer in the dashboard. Engine was callable but invisible. Now visible-by-default in the Astrology tab.

### 9.5.1 — Year of [Lord] badge
Gold-outline pill below the chart title showing `Year of Venus (10th house · Libra)`-style summary. Computes via `profections::compute_profection(ipo, ascendant, today)` whenever both natal IPO date + Ascendant are loaded.

### 9.5.2 — Planet hover enrichment
Each natal-planet glyph tooltip on the wheel now shows decan (Mars-Saturn + theme), Sabian symbol per degree, critical-degree classification (World / Cardinal / Fixed / Mutable), and OOB flag with declination. Approximate declination (β=0) used for natal positions since `natal_positions` table doesn't store latitude.

### 9.5.3 + 9.5.4 — Lifecycle section (combined)
New "LIFECYCLE" section between Natal Chart and Calendar:
- Current Solar Return summary
- Upcoming Saturn / Jupiter / Mars return countdowns ("in 14y 2mo")
- Progressed Sun position + lead progressed-natal aspect

### 9.5.6 — Backtest TimeWindow picker
`pick_list` between thresholds and Run button: All time / Last 5 years / Saturn return ±1y / Jupiter return ±6mo. Cycle-aligned modes resolve return dates via `find_returns()` at RunBacktest time using the selected ticker's IPO chart.

### Database plumbing
- New `fetch_ipo_date(pool, ticker) -> Result<Option<NaiveDate>, String>` query
- New `Message::IpoDateLoaded` + `state.natal_ipo_date: Option<NaiveDate>` field
- Wired into `Dashboard::fetch_all()` so every ticker selection pulls IPO date

### Deferred (9.5.5)
Progressed Sun ingress pill emit on `NatalChartLoaded`. Engine ready (`upcoming_sign_ingresses()`), emit path is one handler hook. Bonus polish, defer to Wave 9.6 or next polish cycle.

---

## Wave 9 — "The Compounding" (shipped 2026-05-06)

**Theme:** Production-tier financial astrology engine. Time-lord systems + cycle returns + narrative depth + visual precision. 5 paired sub-waves shipped against `docs/wave9-plan.md`. AAPL validation reference (IPO 1980-12-12 09:30 EST) round-trips through every module without > 0.1° drift.

### Wave 9.0 — "The Foundation"

- **9.I1 Declination support** in Swiss Ephemeris bridge. New `OBLIQUITY_J2000` constant (23.4367°), `ecliptic_to_declination()` helper, `is_out_of_bounds()` predicate (|δ| > ε). New `declination: f64` field on `PlanetSnapshot`. Public `declination(planet, jdn)` accessor in bridge. 5 PlanetSnapshot construction sites updated.
- **9.B3 Aspect strength visual gradient.** Replaced binary green/red aspect lines with continuous orb-tightness gradient. Each aspect branch in `natal_wheel_3d.wgsl` computes `tight = 1 - orb/max_orb` → alpha scales 0.55→1.0, width 0.65×→1.25×. Tight aspects (orb < 1°) glow noticeably brighter than wide aspects. Opposition (180°) added — magenta variant with shimmer speed 3.0.

### Wave 9.1 — "The Year"

- **9.A1 Solar Return charts** (`src/astrology/solar_return.rs`, ~270 lines). `compute_solar_return(natal, target_year)` searches the exact Sun-return moment via Newton's method (~10 iterations to < 0.001° tolerance — Sun is monotonic prograde). Casts full chart at the return moment + computes cross-aspects with natal. New `calc_sun_longitude_for_search()` in bridge for tight Newton loops. AAPL 2026 SR returns near 2026-12-12 with SR Sun matching natal to 0.0001°.
- **9.B1 Decans** (`src/astrology/decans.rs`, ~190 lines). 12 signs × 3 decans = 36 entries with Egyptian primary ruler + Chaldean sub-ruler + theme. `decan_for_longitude(lon) -> Decan` pure lookup. AAPL natal Sun (Sagittarius decan 3, Mars-Saturn flavored Jupiter ruler) verified.

### Wave 9.2 — "The Cycle"

- **9.A2 Planetary Returns** (`src/astrology/returns.rs`, ~290 lines). Find every Saturn (29.5y) / Jupiter (12y) / Mars (2y) return in a window. Coarse scan + bisection root-finder for retrograde-aware roots. Cluster-merges retrograde triple-passes within `synodic_years/3` threshold. Skips first 70% of one synodic to avoid natal-epoch wobble. AAPL Saturn return ~2010 ✓, Jupiter returns ~11.86y apart ✓.
- **9.B2 Sabian Symbols** (`src/astrology/sabian.rs`, ~430 lines). All 360 Marc Edmund Jones 1925 symbols with image + keynote. AAPL natal Sun → "A child and a dog wearing borrowed eyeglasses" (Sagittarius 21°).

### Wave 9.3 — "The Lord"

- **9.A3 Profections** (`src/astrology/profections.rs`, ~250 lines). Hellenistic annual time-lord rotation. `traditional_ruler(sign_index)` uses Hellenistic-only rulers (Scorpio→Mars, Aquarius→Saturn, Pisces→Jupiter — no modern outers). `compute_profection(natal_date, ascendant_lon, target_date)` handles pre-anniversary age decrement. Yearly + monthly sub-lords. `TIME_LORD_MULTIPLIER = 1.5` for aspect strength boost when time-lord involved. AAPL 2026-05-06 = **Year of Venus (10th house · Libra)**.
- **9.B4 Critical Degrees + OOB** (`src/astrology/critical.rs`, ~310 lines). World degrees (0° cardinal), cardinal 13°/26°, fixed 8°-9°/21°-22°, mutable 4°/17°. `PrecisionFlags { critical, oob }` with combined strength multiplier capped at 2.0×. `OobState` enum (Normal/OobNorth/OobSouth) consuming the new declination field.

### Wave 9.4 — "The Maturation"

- **9.A4 Secondary Progressions** (`src/astrology/progressions.rs`, ~270 lines). "1 day = 1 year" advancement. `compute_progressed_chart(natal, target_date)` casts a chart at `natal_jd + years_elapsed`. AAPL 2026-05-06 → 45.4 years elapsed → equivalent date ~1981-01-26 → progressed Sun moved ~44.7° to ~6° Aquarius. Outer planets nearly stationary in progression. Sign ingress detection for Sun + Moon.
- **9.I2 Backtest extension** (`src/dashboard/backtest.rs`). New `TimeWindow` enum: `All`, `LastYears(u32)`, `Custom(start, end)`, `ReturnZone { planet, return_dates, zone_days }`. `BacktestConfig::filter_days()` pre-filters before min-30-day check. The cycle-aligned mode is the bridge that lets users measure "Does Lagrange work better in Saturn-return-zone years?"

### Validation

- 132/132 lib tests pass (65 new in Wave 9)
- 8/8 dashboard backtest tests pass
- Release build clean, zero warnings
- AAPL natal chart round-trip across all modules without > 0.1° drift

### Out of scope (deferred)

Synastry / composite, Vedic dashas, lunar mansions, asteroid catalog beyond Chiron, alternative house systems, Vertex / East Point. See `docs/wave9-plan.md` risk register.

---

## v12.2 — "The Drawer + Polish" (shipped 2026-05-06)

**Theme:** Tighten v12.1 + close quick wins. 5 sub-items polish the pill notification system and clear residual dead code.

### v12.2.1 — Settings tab scrollable
User flagged in video v8j: *"I'm noticing you can't scroll down the settings menu."* Wrapped `view_settings()` body in `scrollable()` so settings content can scroll independently when it exceeds viewport.

### v12.2.2 — Dead code trim (warnings → 0)
Removed fields/state that v11.9/v12.1 retired but kept "for migration safety":
- `alert_pill_until: Option<Instant>` (replaced by per-pill `expires_at`)
- `fetch_ticker_error: Option<String>` (errors now flow through pill system)
- `show_settings_modal: bool` (Settings tab is sole entry point)
- `ForecastDay.label: String` (set but never read; reconstruct band at render time)

`cargo check` now produces **zero warnings** for the dashboard binary.

### v12.2.3 — Click-pill-to-dismiss
Every pill is now click-actionable. New `Message::NotificationClicked(u64)` handler dismisses the pill, then dispatches the pill's stored `on_click` (if present) via `Task::done(msg)` — single click both routes (e.g. → Universe) and clears chrome. Plain pills (Error/Success/Info) just dismiss.

### v12.2.4 — Notification drawer
Added bell icon between pill stack and gear in tab strip. Click → drawer overlay (via `stack!`) showing `notification_history` newest-first. Each entry shows variant icon + emphasis + body + relative time-ago ("just now", "5m", "2h ago"). Header has "Clear all" + close X. Bell badge shows total count (capped 99).

State: `notifications_drawer_open: bool`. Messages: `ToggleNotificationDrawer`, `ClearAllNotifications`. Drawer uses same overlay pattern as v11.9 toast (proven non-reflowing layer).

### v12.2.5 — Transit pills emit
On `RetroEventsLoaded`, scan retrograde station events within ±7 days of today. For each new station signature (`{planet}:{station}:{date}` deduped via `transit_pill_keys: HashSet<String>`), emit a `Transit` pill ("Mars stations retrograde in 3d") with click → Astrology tab. 12s TTL.

---

## Wave 9 — "The Compounding" — PLAN ONLY (planning shipped 2026-05-06)

**Status:** Implementation roadmap delivered, code not yet started. See `docs/wave9-plan.md` for the full 13-day scope.

**Theme:** Deepen the astrology engine from "what aspects fire today" → "where in the corporate lifecycle is today happening." Time-lord systems (Solar Return, Profections, Planetary Returns, Progressions) + narrative depth (Decans, Sabian Symbols) + visual precision (aspect strength gradient, critical degrees, OOB).

**Sub-wave sequence (paired tracks A+B like Wave 6):**

| Wave | Pair | Theme | Days |
|------|------|-------|------|
| 9.0 | I1 + B3 | "The Foundation" — declination + aspect strength gradient | 1.5 |
| 9.1 | A1 + B1 | "The Year" — Solar Return + Decans | 2.5 |
| 9.2 | A2 + B2 | "The Cycle" — Planetary Returns + Sabian Symbols | 2.5 |
| 9.3 | A3 + B4 | "The Lord" — Profections + Critical Degrees + OOB | 3 |
| 9.4 | A4 + I2 | "The Maturation" — Progressions + Backtest extension | 3.5 |

**Out of scope (deferred):** Synastry / composite, Vedic dashas, Lunar mansions, asteroid catalog beyond Chiron, alternative house systems, Vertex / East Point.

**Validation reference:** AAPL chart (IPO 1980-12-12 09:30 EST). All time-lord computations must round-trip without > 0.1° drift. See plan doc for known reference values.

---

## v12.1 — "The Pill" (shipped 2026-05-06)

**Theme:** Universal pill-based notification system. Replaces the v11.9 ad-hoc `fetching_pill` + `alert_pill` chrome and the inline `fetch_error_banner` that pushed the page header layout down on every fetch. Driven by video review v8j (2026-05-05): *"this keeps some popping down. That's got to stop. If we need notifications, it should pop up here. Just like how this pop up comes to Mars Transit, Aries. That sparkly thing — that could be used for notifications as well."*

### What shipped

- New module `src/dashboard/notifications.rs` (~210 lines) with `Notification` struct + 6 variants (`Sparkly` · `Alert` · `Transit` · `Error` · `Success` · `Info`) + `render_pill` + `render_pill_stack`.
- New state fields: `notifications: VecDeque<Notification>`, `notification_history: Vec<Notification>`, `next_notification_id`, `alerted_lagrange_ids`, `fetch_notification_id`.
- New helpers on Dashboard: `push_notification`, `next_notif_id`, `dismiss_notification`, `expire_notifications`, plus `notify_error` / `notify_success` / `notify_info` shortcuts.
- New Messages: `DismissNotification(u64)`, `NotificationsTick`.
- `Tick` now calls `expire_notifications` + keeps the 60fps subscription active while pills exist (so sparkly + alert glyph keep twinkling).

### Layout impact

- **Killed** `fetch_error_banner` from main column flow — page header no longer reflows on fetch.
- **Killed** v11.9 `fetching_pill` + `alert_pill` ad-hoc blocks (~115 lines) — replaced with one `render_pill_stack(&self.notifications, shader_time)` call between right spacer and gear.
- Tab strip layout unchanged. Chart, header, ornaments untouched.

### Emit sites wired

- `FetchThisTicker` → sticky `Sparkly` pill. Replaced on `FetchTickerComplete`.
- `FetchTickerComplete(Ok)` → `Success` pill (4s TTL).
- `FetchTickerComplete(Err)` → `Error` pill (15s TTL). Replaces old push-down banner.
- `AlertsLoaded` → `Alert` pill per *new* unread Lagrange alert (deduped via `alerted_lagrange_ids`). Click → Universe tab.

### Variant TTLs

| Variant | Default TTL | Use |
|---|---|---|
| Sparkly | none (sticky) | Fetch in progress, celebratory |
| Alert | 8s | Lagrange unread |
| Transit | 12s | Astrology event (wired in v12.2) |
| Error | 15s | Fetch failure, rate limit |
| Success | 4s | Fetch complete, save confirm |
| Info | 8s | General info |

### Caps

`MAX_VISIBLE_PILLS = 3` (deque cap), `MAX_HISTORY = 50` (audit log cap).

### Confirmed working

User screenshots 020627 / 020645 / 020657 / 020725 — 3 alert pills stacking, fetch sparkly mid-fetch, success post-fetch, layout staying put across all 4 frames. *"Overall huge win, very happy with the progress we made."*

---

## v12.0.C — Munger narrative expansion (shipped 2026-05-06)

**Theme:** Deferred B3 from v11.7. v11.7.B added 6 headline variants per Sell/StrongSell verdict but `build_munger_narrative` was thinner than peers — mid-range tickers got short, sparse paragraphs. v12.0.C makes Munger combinatorially deep so different tickers exercise different code paths.

### Coverage expanded

| Section | Before | After |
|---|---|---|
| Sector mental model | 6 sectors, keyword-only | 11 sectors, each with a Munger aphorism quote |
| ROE | 1 band (>25%) | 5 bands: exceptional / good / mediocre / poor / negative |
| Operating margin | 2 bands | 3 bands |
| EV/EBITDA | absent | 4 bands (negative / cheap / fair / premium) |
| Debt/equity | absent | 4 bands (extreme / heavy / fortress / mid-ignored) |
| Free cash flow | absent | 3 bands |
| News tone | 2 cases | 4 cases (Bullish / Somewhat-Bullish / Bearish / Somewhat-Bearish) |
| Astro angle | absent | 2 bands (agreement / divergence) |
| Score-band closer | 2 bands | 6 bands |

Voice preserved: mental models, inversion, terse wit. Quotes retained — *"three things ruin smart people"*, *"voting machine vs weighing machine"*, *"treadmill that pays in pennies"*, *"charity not investment"*.

---

## v12.0.B — Chart hover full profiling (shipped 2026-05-06)

**Theme:** v11.6.K cache split + v11.7.D bar-snap improved chart hover but user kept flagging lag (v11.6 review *"still keeps on lagging"* + v94 review *"not real-time yet"*). Profiled the redraw path. Found two hot spots in `draw()`:

1. Hover-frame path recomputed `min`/`max` over entire data + ALL `rows_chrono.high.to_string().parse::<f32>()` + BB bands on every bar transition. ~127k Decimal→String→f32 allocations per cursor sweep.
2. Cache-side candle paint did the same Decimal→f32 string roundtrip per row per data load.

### Fix

Two new fields on `PriceChart`:
- `price_min: f32`, `price_max: f32` — precomputed range
- `ohlc_f32: Vec<(f32, f32, f32, f32)>` — precomputed OHLC tuples

Two new helpers: `PriceChart::precompute_ohlc()` + `PriceChart::compute_price_range()`. Wired in `view/overview.rs` → run once per chart construction → hover redraw becomes O(1) field reads.

### Result

User confirmed (v94 video): chart hover *"more responsive"* but not yet real-time. Universal perf win regardless. Real-time follow-up deferred (suspect: cosmic-text glyph shaping cost in `fill_text` per redraw).

---

## v12.0.D — Chart hover real-time fix attempt 2 (shipped 2026-05-06)

**Theme:** v11.7.D bar-snap returned `None` on intra-bar moves but still mutated `*state = new_pos`. In Iced 0.14, canvas state mutation alone triggers redraw — so 1000Hz mouse events still produced 1000 redraws/sec on intra-bar travel.

### Fix

One-line change in `update()` handler — skip state mutation when `prev_bar == next_bar`. Crosshair already snaps to `bar_x = x_of(bar_i)` so state precision below bar resolution is irrelevant.

### Result

User confirmed (v94 follow-up): still not real-time vs gauges/sparkline. Deferred for prioritization. Both v12.0.B + v12.0.D fixes retained — universal perf wins, no regression.

---

## Wave 8 — "The Showcase" (shipped 2026-05-06)

Rust axum sidecar — third binary alongside dashboard + scraper. Exposes the same data surface via REST so external clients (OpenBB Workspace, browser dashboards, custom integrations) consume what the desktop dashboard sees.

### Endpoints

- `GET /health` — service + DB liveness
- `GET /widgets.json` — OpenBB Workspace widget manifest (7 widgets)
- `GET /providers` — distinct providers populated in `provider_observations`
- `GET /providers/:p` — series under one provider
- `GET /series/:p/:s?region=X&limit=N` — observations for a series
- `GET /tickers` — active tickers
- `GET /tickers/:t/prices?limit=N` — OHLCV history
- `GET /tickers/:t/lagrange` — Lagrange composite + sub-scores
- `GET /tickers/:t/astro` — astrology score snapshot
- `GET /tickers/:t/fundamentals` — latest fundamentals

### Configuration

- `SIDECAR_PORT` env (default `8765`)
- `SIDECAR_API_KEY` env (optional X-API-Key header check; `/health` and `/widgets.json` always public)
- CORS open for browser-based OpenBB Workspace consumption

### OpenBB Workspace widgets (7)

Lagrange Composite Score, Astrology Score, Pursuit OHLCV, World Bank Indicators, Treasury Yield Curve, OFR Financial Stress Index, CoinGecko Crypto.

Run: `cargo run --bin sidecar` → `http://localhost:8765/widgets.json` is the OpenBB Workspace registration URL.

---

## Wave 7 — "The Library" (shipped 2026-05-05/06)

10 native Rust provider scrapers — OpenBB-tier data depth without the Python runtime. All routed into `provider_observations` (one unified table, one schema). ~6500 new datapoints per scraper run.

### Providers shipped

| # | Module | Source | Key needed | Indicators |
|---|--------|--------|------------|------------|
| 7.1 | `world_bank.rs` | api.worldbank.org | no | 10 × 12 countries (GDP, CPI, unemployment, debt/GDP, FDI, exports/imports, population, industry/GDP) |
| 7.2 | `coingecko.rs` | api.coingecko.com | no (free tier) | top-20 coins (price, market cap, volume, 24h %) + global stats (BTC/ETH dominance, total mcap) |
| 7.3 | `treasury_direct.rs` | home.treasury.gov | no | 13 maturities (1mo, 2mo, 3mo, 4mo, 6mo, 1yr, 2yr, 3yr, 5yr, 7yr, 10yr, 20yr, 30yr) daily |
| 7.4 | `imf.rs` | datamapper API | no | 5 × 11 countries (real GDP growth, inflation, unemployment, debt/GDP, current account) |
| 7.5 | `ecb.rs` | data-api.ecb.europa.eu (SDMX) | no | Euribor 3M, ECB MRR, EUR/USD, EUR/GBP, EUR/CHF, EUR/JPY |
| 7.6 | `bls.rs` | api.bls.gov v2 | optional `BLS_API_KEY` | unemployment, nonfarm, CPI All-Urban + Food/Housing/Gasoline |
| 7.7 | `eia.rs` | api.eia.gov v2 | required `EIA_API_KEY` (skip if absent) | WTI, Brent, Henry Hub, US gasoline |
| 7.8 | `cftc_cot.rs` | cftc.gov/dea | no | non-commercial net positioning on E-mini S&P, Nasdaq, DXY, gold, WTI |
| 7.9 | `ofr.rs` | financialresearch.gov | no | OFR Financial Stress Index (33-component daily composite) |
| 7.10 | `sec_recent.rs` | sec.gov/cgi-bin/browse-edgar | no | filing pulse (8-K, 10-K, 10-Q, S-1, 13D, 13G, Form 4 daily counts) |

### Architecture

`provider_observations` table — composite key `(provider, series_id, region, observation_date)`. Stores any time-series from any source. Cross-source queries via `WHERE provider IN (...) AND series_id LIKE ...`.

Migration: `0046_wave7_providers.sql`.

Scraper main.rs phases 3.14–3.23 wire each provider into the regular pipeline. All providers fail-soft: errors logged + pipeline continues.

---

## v12.0.A — OS toast notification real fix (shipped 2026-05-06)

Resolved HRESULT 0x80070005 via two layers:

1. **Runtime fallback** (`update/helpers.rs`) — windows-rs IShellLinkW + IPropertyStore.SetValue(PKEY_AppUserModel_ID) on a Start Menu `.lnk` at boot. Works for `cargo run` dev workflow.
2. **Installer** (`installer/pursuit-astro.iss`) — Inno Setup .exe installer. `[Icons] AppUserModelID:` parameter binds the AUMID property natively during install. Triggers shell-cache refresh via standard installer SHChangeNotify. Output: `installer/Output/PursuitAstro-Setup.exe` (user-mode install to `%APPDATA%\Pursuit Astro\`).

Cargo dep added (Windows-only target): `windows = "0.58"` with Win32_UI_Shell + Win32_UI_Shell_PropertiesSystem + Win32_Storage_EnhancedStorage features.

Toast notifications work after running the installer once (no manual sign-out/in required).

---

## v11.9 — "The Convergence" (shipped 2026-05-05)

Long iteration arc: 4 video reviews + 1 HTML mockup + 1 written research-plan post collapsed v11.7→v11.8→v11.9 into a final layout where every chrome element is in the right place. User confirmed: *"working and is exactly how I wanted!"*

### Final v11.9 layout

- **Tab strip (right side)**: tabs → spacer → alert_pill → fetching_pill → gear icon
- **Loading bar**: thin gold bar (4px) inline below header, twinkling tip-star at fill seam, "Loading… N%" label. Inside the column flow but small enough not to feel intrusive.
- **Toast overlay**: rendered as `stack![main_view, toast_overlay]` (was `column![]` — caused the white-strip-pushes-content-down bug across multiple reviews). Toasts now float above page without consuming layout.
- **Fetching pill**: chrome-anchored left of gear, ShootingStar canvas + "Fetching {ticker} ({s}s)" text. Lives only while `fetching_ticker == true`. Replaces the old `push_toast("Fetching data for…")` call which expired before fetch completed AND caused layout shift.
- **Alert pill**: 8s TTL via `alert_pill_until: Option<Instant>` field. Set on AlertsLoaded with new unread > 0; pill renders only while `Instant::now() < alert_pill_until`. Auto-dismisses without user action — same ephemeral pattern as fetching pill.
- **Settings**: Tab::Settings variant retained for view dispatch + gear-button click target, but NOT in `Tab::all()` so doesn't render as 8th tab. Single entry point via gear icon.
- **OS notifications**: parked as installer-deferred. AUMID registration + Start Menu shortcut creation via PowerShell ships, but HRESULT 0x80070005 persists for unpackaged Rust apps. In-app toasts are the primary alert path.

### v11.9 iteration history (5 mockup/code rounds, ~60min total feedback)

1. **v11.9 plan** — three-fix layout mockup (loading-bar overlay, alert pill left of gear, settings own panel) → user approved all three
2. **v11.9 implementation** — settings restored as 8th tab, loading bar absolute overlay, alert pill in chrome → user revisions: drop settings tab (gear only), revert loading bar to inline
3. **v11.9 revisions** — Tab::Settings out of all(), loading bar back inline → user "everything working" with two remaining issues: top-pushes-down still happening, alert pill not animated/ephemeral
4. **v11.9 research-plan post** — investigated toast `column!` layout shift root cause, planned `stack!` fix + chrome fetch pill with ShootingStar + alert pill alongside → user approved + asked alert pill alongside fetch pill
5. **v11.9 final** — toast `column! → stack!`, redundant push_toast removed, fetch_pill (ShootingStar) + alert_pill coexist with TTL → final user verdict: *"working and exactly how I wanted!"*

Mockup artifact: `docs/v11.9-layout-mockup.html`
Transcripts: `docs/video-review-v11.{7,8-issues,8.H-corrections,8.I-corrections,9-mockup-request}-transcript.txt`

---

## v11.8 — "The Persistence" (shipped 2026-05-05)

Eight sub-waves A-I shipped from post-v11.6 video review, then layered iterative corrections across three follow-up reviews.

### v11.8 sub-waves

- **8.A** — Settings prominence (later revised in 8.I)
- **8.B** — Header right-side polish + tighter spacing
- **8.C** — Paper Trail icon mockup round 3 — user picked TARGET (concentric circles, harmonizes with natal-wheel motif). RECEIPT (round 2) rejected as "out of nowhere."
- **8.D** — Windows AppUserModelID registration via reg.exe at boot (DisplayName + IconUri)
- **8.E** — Drop inline loading bar (later reversed in 8.H)
- **8.F** — Shooting star animation (replaced sparkle puffs with streaking comet trail)
- **8.G** — Per-ticker fetch button now triggers Wikipedia phase 7 in scraper single-ticker flow
- **8.H** — Restored inline loading bar with tip-star + dropped popup pill per "I want this gone"; in-app toast fallback for alerts
- **8.I** — PowerShell Start Menu shortcut creation alongside AUMID registry; gear glyph (no label) at right end of tab strip; settings was modal still

Mockup: `docs/v11.7.a-paper-trail-icon-mockup.html` (round 1: GRAPH_UP/RECEIPT/NOTEBOOK/COINS/TROPHY/GAME), `docs/v11.8.c-paper-trail-icon-mockup-round3.html` (round 2: RECEIPT/CARDS/STRATEGY/MEDAL/TARGET/TREE)
Transcript: `docs/video-review-v11.8-issues-transcript.txt`

---

## v11.7 — "The Resolution" (shipped 2026-05-05)

Triggered by 6.5-min v11.6 production review (`docs/video-review-v11.6-transcript.txt`). User: *"Overall, huge improvement. Proper circling."* Council de-astro confirmed working — *"all the usables are getting hit differently. Much more diverse and much more meaningful."* Hero header silently approved (no comments). 6 issues + 1 mockup request:

### v11.7.A — "Paper Trail icon redesign" — SHIPPED 2026-05-05

User: *"Reimagine or find another icon. Less noise. Mock it up via HTML. I actually like that flow."* HTML mockup with 6 candidates (current GRAPH_UP, RECEIPT, NOTEBOOK, COINS, TROPHY, GAME_CONTROLLER). User picked **RECEIPT (B)** — literal paper-trail semantic. New icon const `\u{e3aa}` (ph-receipt) replaces GRAPH_UP in `Tab::PaperTrail` icon.

Mockup artifact: `docs/v11.7.a-paper-trail-icon-mockup.html`

### v11.7 backlog (5 items, ~3 days)

| Sub-wave | Theme | Source |
|----------|-------|--------|
| **7.B** | Munger phrase variance — vary actual verdict-level phrases, not just astro_take cycling | v11.6 review [02:48-03:25]: *"Munger keeps on saying stuff like 'doesn't meet my quality bar.' He has not changed."* |
| **7.C** | OS notifications debug — clearly broken in v11.6 despite shipping | v11.6 review [01:50]: *"We have yet to get a notification on any of these. That's clearly broken."* |
| **7.D** | Chart hover lag — 6.K Cache split insufficient, lag persists | v11.6 review [00:43]: *"Still keeps on lagging. Used to change literally real time as I moved my mouse."* |
| **7.E** | Sparkles visibility audit — user couldn't find them | v11.6 review [00:32]: *"There was a little animation for the sparkles. Oh."* |
| **7.F** | Blinking element root cause — unexplained UI flicker | v11.6 review [05:50, 06:07]: *"Don't know why he does that blinking."* |

---

## v11.6 Plan — "The Persistence" (shipped 2026-05-05)

9 sub-waves shipped from post-v11.5 video review. Triggered by 18-min review (transcript: `docs/video-review-v11.5-transcript.txt`). User confirmed ~10 v11.5 features working but flagged 12 fresh issues + 3 persistent ones — most loudly the header layout (re-spec'd three different ways across the review).

### v11.6.A — "Header redo" — shipped 2026-05-05

Resolved through 4 mockup iterations totaling ~22 minutes of video review feedback (vs estimated 6+ hours of code-then-rework cycles). Final approved layout:

- **Tab strip moved to very top of page** (above ornament rule, full-width across border)
- **Hero ticker block left-anchored**: `★ AAPL $278.78 H/L-stacked ⓘ` — star and info-icon SANDWICH the price block (favorites toggle on left, Encyclopedia jump on right)
- **Right column** (fixed 420px): search row with magnifier + 4 action icons inline → Favorites + Recent dropdowns side-by-side beneath
- **Hardcoded ticker buttons dropped** — 10 demo tickers (AAPL/AMZN/GOOGL/JPM/META/MSFT/NVDA/TSLA/UNH/V) seeded into `favorites` table on every boot via `seed_default_favorites_if_empty` (idempotent ON CONFLICT DO NOTHING)
- **Encyclopedia tab dropped from strip** — variant retained, reachable only via the info-circle icon on the ticker hero
- **Tab::all() now 7 entries** (was 8); fixed accompanying `0..8` hardcoded loop in `update::mod` that panicked on first frame after the change

Mockup artifact: `docs/v11.6.a-header-mockup.html` (parchment/gold theme, Fraunces+Inter, annotation panel).

### v11.6.B-K — shipped 2026-05-05

- **6.B** — Council de-astro: 4 persona astro_take rewrites, LLM prompt anti-astro guards, fallback notes scrubbed, Munger 4-cycle diversification. View label "On the stars:" → "Closing thought:". Validated in v11.6 review.
- **6.C** — Natal sphere: CAMERA_TILT 0.32→0.10 in WGSL uniform + matching overlay constant. Wheel reads as sphere not oval.
- **6.D** — Calendar 3-month forward: Prev/Next steps ±3 months instead of ±1; renders 3 month-grids stacked vertically; data fetch range expanded to match.
- **6.F** — Lagrange chart polish: zone-color line tinting (line color matches current zone), 5-anchor date axis labels (mm-dd format), zone tags on right edge (Opt/Fav/Neu/Unf/Mis).
- **6.G** — Sparkle upgrade: 8→12 particles, alpha 0.45→0.85, gold + soft-white mix, 4-pointed star-cross shapes for big particles, animated seed.
- **6.H** — Score gauge clarity: titles renamed (Market: Crypto F&G, Market: Equities F&G, Technical: TICKER, Astrology: TICKER, ★ LAGRANGE: TICKER); Lagrange gets star prefix + sharper "★ THE PRIMARY SCORE" tooltip.
- **6.J** — Fetch stuck timeout: 90s `tokio::time::timeout` wrapping scraper subprocess; toast fallback "Fetch timed out — try again."
- **6.K** — Chart hover Cache split: `Arc<canvas::Cache>` on Dashboard threads into PriceChart; static layers (background, grid, BB bands, SMAs, candles, volume bars, astro markers) painted inside `cache.draw(...)`; hover overlay (crosshair + OHLCV tooltip) painted fresh on a separate Frame each tick. Cache cleared on `DataLoaded` + ticker change.



Triggered by post-v11.5 18-min video review (transcript: `docs/video-review-v11.5-transcript.txt`). User confirmed ~10 v11.5 features working in production but flagged 12 fresh issues + 3 persistent ones — most loudly the header layout (re-spec'd three different ways across the review).

### v11.6.A — "Header redo" (shipped 2026-05-05)

Resolved through 4 mockup iterations totaling ~22 minutes of video review feedback (vs estimated 6+ hours of code-then-rework cycles). Final approved layout:

- **Tab strip moved to very top of page** (above ornament rule, full-width across border)
- **Hero ticker block left-anchored**: `★ AAPL $278.78 H/L-stacked ⓘ` — star and info-icon SANDWICH the price block (favorites toggle on left, Encyclopedia jump on right)
- **Right column** (fixed 420px): search row with magnifier + 4 action icons inline → Favorites + Recent dropdowns side-by-side beneath
- **Hardcoded ticker buttons dropped** — 10 demo tickers (AAPL/AMZN/GOOGL/JPM/META/MSFT/NVDA/TSLA/UNH/V) seeded into `favorites` table on every boot via `seed_default_favorites_if_empty` (idempotent ON CONFLICT DO NOTHING)
- **Encyclopedia tab dropped from strip** — variant retained, reachable only via the info-circle icon on the ticker hero
- **Tab::all() now 7 entries** (was 8); fixed accompanying `0..8` hardcoded loop in `update::mod` that panicked on first frame after the change

Mockup artifact: `docs/v11.6.a-header-mockup.html` (parchment/gold theme, Fraunces+Inter, annotation panel).

### v11.6 backlog (post-A, 10 items, ~6 days)

| Sub-wave | Theme | Items | Source |
|----------|-------|-------|--------|
| **6.B** | Council de-astro + Munger diversification | persona templates strip astrology mentions; Munger gets 6 headline variants | v11.5 review [02:55, 03:35] |
| **6.C** | Natal chart sphere | reduce CAMERA_TILT 0.32→0.10 in WGSL — chart reads as sphere not oval | v11.5 review [00:22] |
| **6.D** | Calendar 3-month forward | Astro Calendar steps += 3 instead of += 1 | v11.5 review [01:10] |
| **6.F** | Lagrange chart polish | better gridlines/legend on sparkline | v11.5 review [05:20] |
| **6.G** | Sparkle animation upgrade | larger particles, brighter alpha, more visible on loading bar | v11.5 review [14:34] |
| **6.H** | Score gauge label clarity | clarify which score is "the" score (gauge/Astro/Lagrange disambiguation) | v11.5 review [15:33] |
| **6.J** | Fetch stuck root cause | log + add timeout when fetch hangs mid-session | v11.5 review [01:35] |
| **6.K** | Iced 0.14 chart hover perf | investigate redraw cost; possibly add `Cache` widget | v11.5 review [10:25] |

---

## v11.5 Plan — "The Explanations" (planning, 2026-05-05)

Triggered by 27-min video review on 2026-05-05 — first end-to-end test of full Wave 6 stack on 2028-ticker universe. Transcript saved at `docs/video-review-v11.4-transcript.txt` (331 lines).

### Dominant theme: pop-up explanations EVERYWHERE

User mentioned "should have a pop-up explaining" 15+ times across the video. Game-tutorial UX explicitly requested at 17:06: *"Like in a lot of games when you hover something a pop-up appears, and then you could right-click and choose an option to explain."*

### What worked (validated)

User explicitly approved during the video:
- Aspect line thinning (Wave 1a) [00:26]
- Galaxy background mute (Wave 1e) [00:57]
- Sound effects on alerts (existing) [06:19]
- Council verdict diversification (Wave 4a) [19:02 *"I like the fact they're having their own opinions on things"*]
- Horoscope reading layout (Wave 2c+2d) [21:27]
- Initial state at compact + default text size [00:01]

### Sequencing rationale

Ordered by **dependency + risk + ROI**, not quick-wins:
1. **A "The Foundation"** — tooltip helper (~0.5d) — unblocks B/C/D/F
2. **B "The Layout"** — header reshuffle, gauges side-by-side, Settings → modal (~2.5d) — high-risk, do early
3. **C "The Explanations"** — apply helper to 12 tooltip locations (~2.5d) — additive, lowest risk per item
4. **D "The Interactions"** — aspect line hover, chart zoom, OS notifications (~2.5d) — isolated to astrology + system layer
5. **E "The Encyclopedia"** — new Wikipedia tab + scraper module (~1.5d) — fully independent
6. **F "The Polish"** — candle word labels, loading %/sparkles, strategy defaults (~1d) — final coat

Total ~10 days. **OpenBB Wave 7 defers entirely** — no user pressure for more data sources, plenty of pressure for tooltips/layout/Wikipedia.

### v11.5 sub-waves (6 total, ~22 items)

- **A1-A3**: tooltip helper, right-click menu primitive, doc pattern
- **B1-B7**: search/ticker swap, favorites dropdown, zodiac legend relocation, gauges side-by-side, Settings modal, XL-size layout fix
- **C1-C8**: tooltips on Universe columns, A/S header, sector heatmap, 5 gauges, FRED indicators, Lagrange labels, backtest context, council verdicts
- **D1-D4**: aspect line hover (shader vs Canvas decision needed), mouse-wheel zoom, `notify-rust` toast, multi-monitor verify
- **E1-E6**: Wikipedia REST scraper, migration `0043_wiki_summary`, new Tab variant, view module, refresh pipeline integration, cache-miss UI
- **F1-F7**: candle full-word labels, Volume tooltip, loading %, sparkles, stuck-at-85% fix, dedupe indicators, strategy smart defaults

See `docs/research-v11.5-explanations.md` for architecture, video timestamps, technical decisions per item.

---

## Wave 7 + 8 Plan — Pure-Rust OpenBB Alternative (planning, 2026-05-04, revised)

Initial Wave 7 plan was Python sidecar via `pip install openbb`. Revised to pure-Rust two-phase approach (Path C) after analyzing real provider gap (~10, not 350) and Workspace contract feasibility.

### Decision: Two-Phase Pure Rust

- **Wave 7 — "The Library"** — 10 native Rust provider scrapers, ~8-10 days.
- **Wave 8 — "The Showcase"** (conditional) — Rust axum sidecar mimicking OpenBB Workspace API contract, ~7 days. Decision gate after Wave 7.

No Python dependency. No `pip install openbb`. We hand-write Rust HTTP wrappers for the specific datasets we want.

### Wave 7: 10 Native Rust Providers

| # | Provider | Domain | Wave |
|---|----------|--------|------|
| 1 | World Bank | International macro (200+ countries) | 7.0 |
| 2 | IMF DataMapper | Sovereign macro | 7.0 |
| 3 | ECB Statistical Warehouse | EU monetary policy | 7.0 |
| 4 | CFTC Commitment of Traders | Futures positioning sentiment | 7.1 |
| 5 | BLS | Detailed US labor data | 7.2 |
| 6 | EIA | US energy data | 7.2 |
| 7 | OFR Financial Stress Index | Composite stress metric | 7.3 |
| 8 | Treasury Direct | Treasury auctions, yield curve | 7.4 |
| 9 | CoinGecko | Crypto prices | 7.4 |
| 10 | Buffer | TBD based on need | — |

5 migrations (0043-0047). Existing `src/scraper/sources/` pattern (Wave 6.A1/A2) reused.

### Wave 8: Rust Sidecar (axum) Mimicking OpenBB Workspace

NEW binary `cargo run --bin workspace` serves on localhost:7100. axum + tower-http + sqlx (read-only role). Implements OpenBB Workspace HTTP contract:
- `GET /widgets.json` — widget catalog
- `GET /<widget-endpoint>?<params>` — data arrays
- Bearer PAT auth + CORS for `pro.openbb.co`
- ngrok tunnel exposes localhost:7100 publicly
- Workspace cloud UI connects via the tunnel

Surfaces our proprietary Lagrange / astro patterns / eclipses / fixed stars / data freshness as Workspace widgets. Polished cloud dashboards + shareable URLs for Pursuit demo day.

Decision gate: only ship Wave 8 if Wave 7 data is signal-bearing AND Pursuit Fellowship needs shareable dashboards.

### Why Pure-Rust Over Python OpenBB

- Real provider gap = ~10, not 350. Effort to port 10 in Rust ≈ effort to install/learn OpenBB Python.
- Stays in Rust (project's defining trait).
- Each provider isolated, our quality control.
- No Python operational footprint, no two-language maintenance.
- Workspace integration (Wave 8) keeps the cloud UI value without giving up Rust purity.

### Won't Do

- ❌ Install OpenBB Python package
- ❌ Replace existing scrapers
- ❌ Embed Workspace in Iced dashboard
- ❌ Add paid-tier providers in Wave 7 (those stay in API key backlog)

See `docs/research-wave7-openbb.md` for architecture, full provider analysis, sub-wave breakdown, risks.

---

## v11.4.0-w6.3 — "The Trust" (2026-05-04)

**Theme:** Final Wave 6 sub-wave. Track A surfaces data quality directly in the UI; Track B adds the loudest astrological events (eclipses) to the engine.

### Track A — 6.A4 Data Freshness UI Badges

Migration `0042_data_freshness_view.sql`: new `data_freshness` SQL view computes per-ticker:
- `last_price_at` — MAX(price_data.date)
- `last_fund_at` — MAX(fundamental_metrics.fetch_date)
- `last_news_at` — MAX(news_articles.published)
- `last_sent_at` — MAX(sentiment_scores.fetch_date)
- `last_astro_at` — MAX(astro_scores.score_date)
- `price_source_count` — DISTINCT data_source values (provenance from Wave 6.A1)
- `fresh_count` — 0-5 score: each source contributes 1 if fresh within threshold (prices 3d, fundamentals 30d, news 7d, sentiment 7d, astro 2d)

`UniverseRow` gains `fresh_count: Option<i32>` field. Universe table query LEFT JOIN's the view. New "Data" column with badge: `●●●●○` rendered via Unicode `\u{25CF}` (filled) + `\u{25CB}` (outline) repeats. Color tinting follows zones: 5=Optimal green, 3-4=Favorable, 2=Neutral, ≤1=Misaligned red. Tooltip on header explains the 5 sources.

### Track B — 6.B4 Eclipse Cycles + Saros Series

New `src/astrology/eclipses.rs` module + migration `0041_eclipses.sql` with 17 eclipses (NASA Five-Millennium Catalog 2025-2028) hardcoded in both:

- **DB seed** (`migrations/0041`) — table `eclipses` with date PK, type CHECK constraint (solar_total/partial/annular/hybrid + lunar_total/partial/penumbral), longitude, magnitude, saros_series, notes. Index on upcoming dates.
- **Const fallback** (`upcoming_eclipses()` fn) — same data hardcoded so transit scoring works without DB round-trip. Sync needed when adding entries.

Why both: `compute_transit_score` is sync (called from many code paths) and adding async DB lookup would cascade refactors. Const list keeps scoring fast; DB table available for UI listing/queries later.

`detect_activations(eclipses, natal_positions, score_date)` returns activations within 6° orb. Time window: next 12 months OR past 6 months (echo fade). Strength model:

- Solar = -10.0 base (identity-driving stress)
- Lunar = -6.0 base (emotional/relational)
- Tightness factor: 1.0 at 0° orb → 0.3 at 6° orb (linear)
- Time factor:
  - Within 90 days: 1.0
  - 90-365 days: ramps 1.0 → 0.5 (anticipation)
  - Past, in echo window: 1.0 → 0.0 over 180 days

`TransitScore` gains `eclipse_activations: Vec<EclipseActivation>` field. Activation total added to delta_sum pre-sigmoid alongside patterns/stars/Arabic-parts (Wave 6.B1/B3 stack).

Each activation tagged with `saros_series` so future feature can cross-reference what happened to the ticker last time same Saros family hit (cycle = 18y 11d 8h).

5 new unit tests: orb detection, distance rejection, far-future window exclusion, lunar < solar magnitude, past-echo window. 67/67 lib tests passing.

**Files modified:** 5 (`src/astrology/mod.rs`, `src/astrology/natal.rs`, `src/dashboard/db/universe.rs`, `src/dashboard/view/universe.rs`) + 3 new (`src/astrology/eclipses.rs`, migrations `0041` + `0042`)

---

## v11.4.0-w6.2 — "The Depth" (2026-05-04)

**Theme:** Wave 6.2 paired Track A (analyst price targets) + Track B (fixed stars + Arabic Parts). Both add new dimensions of signal: forward-looking analyst consensus, and traditional astrological reference points the engine was missing.

### Track A — 6.A3 Analyst Price Targets

New `src/scraper/analyst_targets.rs` calls Finnhub `/stock/price-target` endpoint. Response shape: `targetHigh / targetLow / targetMedian / numberOfAnalysts` (camelCase). All fields optional — tickers without analyst coverage return all-None response, treated as no-data and skipped.

Migration `0040_analyst_targets.sql`: new `analyst_targets` table keyed on ticker (single row, latest fetch wins). Fields: low/median/high (NUMERIC(10,2)), n_analysts, fetch_date, last_updated. Index on fetch_date.

Two API surfaces:
- `fetch_analyst_targets(pool, client, key)` — universe-wide pull, 30 ticker batch budget per run, 7-day staleness check. 1.1s sleep between calls = 55/min stays inside Finnhub free 60/min limit.
- `fetch_one_and_store(pool, client, key, ticker)` — single-ticker variant for FetchThisTicker flow.

Wired into `run_all_fetches` (phase 2.3b after general Finnhub) and `fetch_single_ticker` (phase 4b).

### Track B — 6.B3 Fixed Stars + Arabic Parts

**Fixed stars** (`src/astrology/fixed_stars.rs`): 8 stars catalog with J2000 ecliptic longitudes hardcoded, linear precession applied (`50.29″/year ≈ 0.01397°/year`). Activation = transit planet within 1° orb. Strength tightness-scaled (full at 0° orb, half at 1° orb). Catalog:

| Star | J2000 Lon | Strength | Archetype |
|------|-----------|----------|-----------|
| Regulus | 149.83° | +10 | Kingship, finance success |
| Spica | 203.88° | +12 | Wealth, abundance |
| Antares | 249.83° | +8 | Leadership, finance/military |
| Aldebaran | 69.93° | +6 | Honors, recognition |
| Sirius | 104.28° | +8 | Fame, media attention |
| Vega | 285.38° | +5 | Artistry, IP success |
| Fomalhaut | 334.08° | +4 | Transformation, dreams |
| Algol | 56.35° | -14 | Sudden loss, danger |

Approximation rationale: precise positions require `swe_fixstar2` via raw FFI (the safe wrapper of `swiss-eph` doesn't expose it). For 1° orb activations, linear precession from J2000 is sufficient (~0.36° drift in 26 years, well within orb tolerance). Documented in module comment.

**Arabic Parts** (`src/astrology/arabic_parts.rs`): pure formula derivations.
- **Part of Fortune** = ASC + Moon - Sun (day formula; IPO charts always day)
- **Part of Spirit** = ASC + Sun - Moon
- **Part of Commerce** = ASC + Mercury - Sun
- **Part of Substance** = ASC + 30° (2nd-house cusp approximation, Whole Sign)

Transit aspects to Parts use 3° orb (sensitive degrees, not bodies). Aspects scored: conjunction (+4), sextile (+2), square (-3), trine (+3.5), opposition (-3.5). Part of Fortune carries 1.0× weight; others 0.5× (advisory).

`NatalChart` struct gains `ascendant: Option<f64>` field. `NatalChart::compute` calls `compute_houses_nyse(jdn)` and stores result. Updates breaking change to `helpers.rs::build_natal_from_snapshots` (sets `ascendant: None`).

`compute_transit_score` adds star + Arabic Part deltas to `delta_sum` pre-sigmoid. `TransitScore` gains `star_activations`, `arabic_parts`, `part_activations` fields.

5 new unit tests (precession arithmetic, conjunction detection, orb edge, Moon skip filter, Algol negative-strength); 3 Arabic Part tests (Fortune formula, no-ascendant fallback, transit-conjunct-Fortune detection). 57/57 lib tests passing total.

**Files modified:** 6 (`src/astrology/mod.rs`, `src/astrology/natal.rs`, `src/scraper/main.rs`, `src/dashboard/update/helpers.rs`) + 4 new (`fixed_stars.rs`, `arabic_parts.rs`, `analyst_targets.rs`, migration `0040`)

---

## v11.4.0-w6.1 — "The Precision" (2026-05-04)

**Theme:** Wave 6.1 paired Track A (fundamentals fallback chain) + Track B (aspect strength model upgrades). Both inputs to Lagrange composite score get richer simultaneously.

### Track A — 6.A2 Multi-Source Fundamentals Fallback

New `src/scraper/sources/fundamentals.rs` with `SourcedFundamentals` normalized struct (20 optional fields covering everything in `fundamental_metrics`):

- **Finnhub `/stock/metric?metric=all`** — 60-field response covering market cap (in millions, multiplied to USD), P/E TTM, P/B, P/S, EV/EBITDA, PEG, P/FCF, ROE/ROA (returned as %, divided by 100), margins, debt/equity (with awkward `totalDebt/totalEquityAnnual` field name), current ratio, EPS, dividend yield.
- **Alpha Vantage `OVERVIEW`** — string-typed PascalCase fields. "None" / "-" sentinel handling. Third-tier fallback only (AV rate limit shared with prices).

`fundamentals::fetch_and_store` refactored: extracted `fetch_fmp` + `insert_fmp` + new `insert_sourced` for fallback rows. New `fetch_and_store_with_fallback(ticker, pool, client, fmp_key, finnhub_key, av_key)` is the cascade entry. Old `fetch_and_store` retained as thin wrapper for legacy callers (passes None for fallback keys).

Migration `0039_fundamentals_data_source.sql`: adds `data_source TEXT NOT NULL DEFAULT 'fmp'` + index. Single-ticker fetch in `main.rs` now routes through cascade with full fallback chain.

### Track B — 6.B2 Aspect Strength Model

Added three multiplicative modifiers to score pipeline:

- **`body_weight(planet) -> f64`** — Sun/Moon=1.5 (luminaries drive identity), Jupiter/Saturn=1.3 (slow + heavy), Uranus/Neptune/Pluto=1.4 (transpersonals last years), Mercury/Venus/Mars=1.0 (fast inner planets), nodes/Chiron=0.8 (modal points). Weight applied as mean of both bodies in aspect.
- **`mutual_reception_bonus(p1, sign1, p2, sign2)`** — 1.15× when `p1` is in `p2`'s domicile AND `p2` is in `p1`'s domicile (e.g. Mars in Libra + Venus in Aries). Both bodies effectively "support" each other across signs.
- **`out_of_sign_modifier(lon_a, lon_b, aspect)`** — 0.75× when angular separation matches aspect within orb but sign-distance doesn't match aspect's expected sign-count (e.g. trine in 5 signs apart instead of 4). Out-of-sign aspects lack elemental support.

New `score_aspect_v2()` integrates all modifiers. `score_aspect_full()` retained as wrapper for backward compat. `compute_transit_score` switched to `score_aspect_v2` with full longitude + sign context.

Existing modifiers already in place pre-Wave 6: orb tightness (linear from 1.0 to 0.25), applying/separating (1.5×/0.7×), essential dignity (1.2×/0.8×). 18/18 aspect tests passing including new `test_body_weight_luminaries_heaviest`, `test_mutual_reception_mars_venus`, `test_out_of_sign_penalty`, `test_v2_includes_body_weight`.

**Files modified:** 6 (`src/astrology/aspects.rs`, `src/astrology/natal.rs`, `src/scraper/main.rs`, `src/scraper/fundamentals.rs`, `src/scraper/sources/mod.rs`, migration `0039` + new `src/scraper/sources/fundamentals.rs`)

---

## v11.4.0-w6.0 — "The Reliability" (2026-05-04)

**Theme:** Wave 6.0 — paired financial-data + astrology-engine expansion. Track A removes single-points-of-failure in price data. Track B adds geometric pattern recognition the engine was missing.

### Track A — 6.A1 Multi-Source Price Fallback

New `src/scraper/sources/` module with two adapters and a cascade dispatcher:
- **Yahoo Finance** (`sources/yahoo.rs`) — v8 chart API at `query1.finance.yahoo.com`. JSON shape: `chart.result[0].timestamp[] + indicators.quote[0].{open,high,low,close,volume}`. Browser-style User-Agent required (Yahoo blocks default reqwest UA). 3-month default range, daily interval.
- **Stooq** (`sources/stooq.rs`) — CSV at `stooq.com/q/d/l/?s={ticker}.us&i=d`. Header parsing strict ("Date,Open,High,Low,Close,Volume" required). Returns "No data" for unknown tickers.
- **Cascade** (`sources/mod.rs`) — `fetch_fallback_chain(ticker, client)` tries Yahoo then Stooq, returns `(rows, source_name)`. Each failure logged with source identity for operator diagnostics.

`prices::fetch_and_store` refactored: AV primary path → on rate-limit/error → cascade fallback. Provenance tagged at insert via new `data_source` column. AV writes `'alpha_vantage'`, fallback writes `'yahoo'` or `'stooq'`.

Migration `0038_price_data_source.sql`: adds `data_source TEXT NOT NULL DEFAULT 'alpha_vantage'` column + GIN index on `price_data`.

### Track B — 6.B1 Aspect Pattern Recognition

New `src/astrology/patterns.rs` (~470 lines) detecting 7 geometric configurations from combined natal + transit positions:

| Pattern | Geometry | Strength |
|---------|----------|----------|
| Grand Trine | 3 planets, each pair 120° (orb 4°) | +15 |
| T-Square | opposition + 2 squares to apex (orb 6°) | -12 |
| Grand Cross | 4 planets, 2 oppositions + 4 squares (orb 6°) | -18 |
| Yod | 2 sextile + apex 150° from both (orb 3-4°) | -8 |
| Stellium | 3+ planets in same sign | +6 base, +4 per extra body |
| Mystic Rectangle | 4 planets, 2 sextiles + 2 trines + 2 oppositions (orb 4°) | +10 |
| Kite | Grand Trine + opposition from 4th planet | +18 |

Each pattern marked `is_cross` when bodies span both natal + transit (1.0× multiplier vs 0.6× for intra-chart). Tightness factor (0.5 + 0.5 × tightness) modulates final strength based on average orb across defining aspects.

`compute_transit_score` adds `pattern_score_total(&patterns)` to `delta_sum` BEFORE sigmoid normalization, so patterns meaningfully shift score without being washed out by aspect_count division.

`TransitScore` gains `patterns: Vec<AspectPattern>` field. New `patterns_to_json()` serializer in `natal.rs`. Scraper `compute_astro_scores` + `compute_astro_score_one` write JSON to new `aspect_patterns` JSONB column.

Migration `0037_aspect_patterns.sql`: adds `aspect_patterns JSONB DEFAULT '[]'` to `astro_scores` + GIN index for "show me all tickers with a Grand Trine" queries.

7/7 unit tests passing: exact Grand Trine, T-Square, Stellium, Yod (with corrected geometry — apex must be 150° from BOTH sextile ends, not 150° from the start), cross-chart marking, intra-chart marking, no false positives on random angles.

**Files modified:** 8 (`src/astrology/mod.rs`, `src/astrology/natal.rs`, `src/astrology/patterns.rs` new, `src/scraper/main.rs`, `src/scraper/astrology.rs`, `src/scraper/prices.rs`, `src/scraper/sources/mod.rs` new, `src/scraper/sources/yahoo.rs` new, `src/scraper/sources/stooq.rs` new, migrations `0037` + `0038`)

---

## v11.3.0 — "The Refinement" (2026-05-04)

**Theme:** All 22 video-review feedback items shipped across 5 waves. Polish pass on UI density, layout flow, chart overlays, council templates, gauges, and background atmosphere.

### Wave 1 — Quick Visual Wins
- **1a Aspect line thickness** — `ASPECT_W` 0.005 → 0.003 in `natal_wheel_3d.wgsl`. Reverts v9.3 thickening that produced "lines spitting at each other."
- **1b Section icons** — Phosphor icons (GLOBE/GRAPH_UP/CALENDAR/MOON_STARS/LIGHTNING) on Astrology tab section headers. New `icon_eyebrow()` helper in `shared.rs`.
- **1c Compact horoscope** — Moon/Mercury/Timing collapsed into single `row![]` at `text_xs`. "Mercury: Direct — clear communications" trimmed to "Mercury: Direct".
- **1d Scrollbar gutter audit** — New `gutter_scroll()` helper wraps content with 20px right padding + gold style. Applied to 13 sub-scrollables across 6 view files.
- **1e Galaxy background mute** — Desaturated nebula purples, fewer stars (threshold 0.92→0.93), slower twinkle (1.2→0.8). Less "screaming nebula."

### Wave 2 — Layout Restructure
- **2a Header price + H/L** — Ticker block now shows last close (gold) + day high/low pulled from `self.rows.last()`. No state changes needed.
- **2b Search above ornament** — `compact_nav` moved above `PageHeaderOrnament`. Ornament now divides header from tab bar.
- **2c Astrology tab reorder** — Horoscope extracted to standalone section. New flow: Natal → Calendar+Forecast (row) → Horoscope → Backtest → Strategy.
- **2d Two-column compression** — `row![calendar_col, forecast_col]` with `Length::FillPortion(1)` each. ~250px vertical compression.

### Wave 3 — UX + Chart Overlays (Iced 0.14 features)
- **3a Sector dropdown** — Button row → `pick_list` with "All" sentinel mapping to `None`.
- **3b Column header tooltips** — `tooltip()` widget on Fin/Mac/Sht/Conc with full names + descriptions. Custom `tip_style` container.
- **3c Rising sign backfill** — Seeder WHERE clause now matches tickers missing EITHER `natal_positions` OR `natal_angles`. Fixes blank Rising sign for tickers seeded pre-angles.
- **3d Per-ticker fetch scope** — New `seed_natal_chart_one` + `compute_astro_score_one` for `--ticker` mode. Fixes universe-wide loops on single-ticker fetches.
- **3e Planet symbols overlay** — `stack![shader, pin(text_glyph)]` with Unicode astrology glyphs (☉☽☿♀♂♃♄⛢♆♇☊☋⚷). Position math in `planet_pixel_pos()` accounts for camera tilt.
- **3f Chart hover tooltips** — Each glyph wrapped in `tooltip()` showing planet+sign+degree on hover.

### Wave 4 — Council Fix + Chart Polish
- **4a Council diversification** — Headline pools 3→6 variants per persona/verdict. Ticker-specific fundamental injection (ROE, P/E, PEG, op margin). New `headline_variant()` hashes char codepoints + score so AAPL/MSFT/META no longer share strings.
- **4b Chart size enum** — `ChartSize::{Compact 320, Default 400, Large 520}`. State field + pick_list in Astrology tab and Settings. `planet_pixel_pos` parametrized by `chart_px`.
- **4c Fetch progress bar** — `fetch_start_time: Option<Instant>` drives time-based fill (cap 0.85 over 30s). `FillPortion` split: gold filled / 15%-alpha track. Replaces indefinite pulse.

### Wave 5 — Deferred Polish
- **5a Tooltip size setting** — `TooltipSize::{Small/Default/Large}` enum. State field + Settings UI. Tuple `(font_px, box_w, box_h)` passed into `PriceChart` via new `tooltip_dims` field.
- **5b Scraper retry helper** — New `src/scraper/retry.rs` with `with_retry()` higher-order async helper (3 attempts, 2s/8s backoff). Wired into FMP fundamentals fetch (key-metrics + ratios endpoints).
- **5c Gauge reimagination** — Compass-rose detailing: outer gilt arc (45% gold), sundial tick marks (major every 25pt, minor every 5pt), gold-backed needle, 8-point star center cap.
- **5d Background texture** — Vignette shader gains 3 new layers: 8x60 horizontal "fiber" pattern (chain lines), 5x5 sepia-warm aging blotches, retains existing per-pixel grain. Renaissance parchment feel.

**Files modified:** 23 (`shaders/natal_wheel_3d.wgsl`, `shaders/vignette.wgsl`, `view/shared.rs`, `view/astrology_tab.rs`, `view/mod.rs`, `view/universe.rs`, `view/overview.rs`, `view/research.rs`, `view/fundamentals.rs`, `view/settings.rs`, `state.rs`, `agents.rs`, `gauges.rs`, `charts.rs`, `astrology.rs`, `update/astro.rs`, `update/data.rs`, `scraper/main.rs`, `scraper/astrology.rs`, `scraper/fundamentals.rs`, `scraper/retry.rs` new)

---

## v11.2.0 — "The Foundation" (2026-04-30)

**Iced 0.13 → 0.14 framework upgrade.** Major dependency migration touching 13+ source files across 19 breaking API changes.

### Breaking API Changes Resolved
| Category | Count | Change |
|----------|-------|--------|
| Shader system | 6 | `Storage` pattern → `Pipeline` trait (auto-creates on first frame) |
| wgpu 22 → 27 | 8 | `entry_point` now `Option`, new `compilation_options`/`cache`/`depth_slice` fields |
| Canvas events | 2 | `update()` returns `Option<Action<Message>>` instead of `(Status, Option<Message>)` |
| Widget renames | 17 | `horizontal_rule` → `rule::horizontal`, `Space::with_width` → `Space::new().width()` |
| Text alignment | 32 | `horizontal_alignment` → `align_x`, `vertical_alignment` → `align_y` |
| Application boot | 1 | First arg now boot fn, title via builder, `.run()` replaces `.run_with()` |
| Keyboard | 1 | `on_key_press(f)` → `keyboard::listen().filter_map()` |
| Palette | 1 | New `warning` field required |
| Scrollable | 1 | `Scroller.color` → `Scroller.background`, new `auto_scroll` field |
| Button style | 1 | New `snap` field required |

### Environment Fix
- Added `C:\msys64\ucrt64\bin` to Windows user PATH — `swiss-eph` C compilation now works from PowerShell (was only working from bash/MSYS2)

### Unblocked by Upgrade
- `pin` widget: absolute (x,y) positioning for planet labels over shader
- `float` widget: floating overlays with dynamic positioning
- `stack` improvements: `push_under` for shader-behind-UI layering
- Animation API: built-in animation primitives for hover transitions
- cosmic-text 0.15: better Unicode symbol rendering (zodiac glyphs)
- wgpu 27: modern GPU backend

**Files modified:** 19 (`Cargo.toml`, `shaders/mod.rs`, `charts.rs`, `main.rs`, `update/mod.rs`, `theme.rs`, `view/mod.rs`, `view/shared.rs`, `view/overview.rs`, `view/portfolio_tab.rs`, `view/universe.rs`, `view/paper_trail.rs`, `view/settings.rs`, `view/fundamentals.rs`, `view/astrology_tab.rs`, `calendar.rs`, `astrology.rs`, `gauges.rs`, `heatmap.rs`)

---

## v11.1.0 — "The Craft" (2026-04-30)

- **Clickable entity links:** Insider names and institutional holder names in Research tab now open Google search on click. Reusable `link_button()` helper in `shared.rs` with gold hover styling.
- **Tab glow rework:** Active tab now has 2px gold border with bookmark shape (rounded top corners, flat bottom) replacing the 15% alpha gold background fill. Hover tabs show faint gold border preview.
- **Chart layer visibility toggles:** 4 toggle buttons (Natal/Transit/Aspects/Retro) above the natal wheel. Eye/eye-slash Phosphor icons. State flows through Dashboard bools -> shader uniforms -> WGSL conditionals. Retrogrades handled in Rust packing (no extra shader uniform needed).
- **Nav layout redesign:** Two-row header: search bar (280px) left, ticker name centered, icon-only action buttons (refresh/fetch/theme) right. Second row: ticker DB buttons + recently viewed. Theme button changed from text to moon-stars icon.

**Files modified:** 8 (`view/shared.rs`, `view/research.rs`, `view/mod.rs`, `view/astrology_tab.rs`, `state.rs`, `update/astro.rs`, `shaders/mod.rs`, `shaders/natal_wheel_3d.wgsl`, `icons.rs`)

| Feature | Before | After |
|---------|--------|-------|
| Entity names | Plain text | Gold clickable links (Google search) |
| Active tab | Gold bg fill + sparkle | 2px gold border bookmark shape |
| Chart layers | Always all visible | 4 toggleable layers (eye/eye-slash) |
| Header layout | Single row, text buttons | Two-row, icon-only actions, search-left |

---

## v11.1 Video Review (2026-04-30)

15-minute screen recording review of v11.1 build. Audio transcribed via faster-whisper.

**Approved:** Tab bookmark borders, icon theme toggle, Universe legibility, Council verdict accuracy, overall layout direction.

**21 feedback items captured** across 5 categories:
- P0 Layout (5): Header price/high-low, search position, forecast-calendar merge, horoscope reposition, reduce dead space
- P0 Chart (5): Aspect lines too thick, galaxy bg rework, planet symbols, interactivity, chart size
- P1 UX (5): Sector dropdown, column tooltips, tooltip sizing, Rising sign bug, horoscope formatting
- P1 Visual/Data (5): Section icons, scrollbar gutter, gauge redesign, progress bar, data reliability
- P1 Bug (1): Council template responses too generic

Full feedback structured in TODOS.md with video timestamps.

---

## v11.0.0 — "The Intelligence" (2026-04-29)

- **90-day astro forecast:** Computed from natal positions + transit ephemeris, displayed as colored timeline events (favorable/unfavorable date ranges with aspect descriptions)
- **Big Three summary:** Sun/Moon/Rising signs displayed prominently above natal chart
- **Smart calculator defaults:** DCF growth% auto-fills from PEG ratio, Options Greeks vol% from historical volatility
- **Zodiac sign band + planet symbol legend:** Visual legend below natal chart showing planet glyphs with natal/transit/retro color coding
- **Pulsing loading bar:** Shimmer animation during data fetch operations
- **Icon-only nav buttons:** Action buttons converted to Phosphor icons

---

## v10.0.0 — "The Signal" (2026-04-29)

- **RSS tone sentiment:** Keyword-based sentiment scoring from 25 news feeds
- **Lagrange adaptive weighting:** Removed 50-default compression, signals now scale to their actual range
- **Richer agent verdicts:** Sector-aware, news-informed verdicts with 3 headline variants
- **Fetch-this-ticker button:** One-click data fetch for selected ticker

---

## v9.3.0 — "The Clarity" (2026-04-29)

- **Aspect lines overhaul:** Base width 0.003→0.005 (67% thicker). Alpha values boosted: conjunction 0.20→0.45, sextile 0.14→0.30, square 0.16→0.35, trine 0.20→0.40. Colors more saturated. Conjunctions now 2× base width (was 1.5×). Squares 1.3× width (new). Added outer glow halo (4× line width, 15% alpha) for luminous bleeding effect against galaxy background.
- **Universe table columns widened:** Astro/Score 56→64px, Fin/Macro/Short 44→52px, Concordance 90→100px. Headers no longer truncated ("Astr o" → "Astro").
- **Tab labels bolder:** Active tab label uses Fraunces Bold at 16px (was SemiBold at 14px). New `DISPLAY_BOLD` font constant. Active tabs visually heavier and more readable.
- **Scrollbar gutter:** Page content right padding 10→20px. Scrollbar no longer overlaps content text.

**Files modified:** 5

| Fix | Before | After |
|-----|--------|-------|
| Aspect lines | Thin (0.003), faint (0.14-0.20 alpha), no glow | Thick (0.005), bold (0.30-0.45 alpha), luminous glow halo |
| Universe headers | Truncated "Astr o", "Scor e" | Full "Astro", "Score" at proper width |
| Active tab label | Fraunces SemiBold 14px | Fraunces Bold 16px |
| Scrollbar | Overlaps content | Own gutter space (20px right padding) |

---

## v9.2.0 — "The Cosmos" (2026-04-29)

- **Galaxy background:** Natal chart background replaced from flat bg_color to procedural galaxy field. Deep space gradient (near-black center → dark purple edges) with nebula swirl (layered sine noise in purple/blue) and dense twinkling star field across entire chart. Stars have color variation: cool white (common), blue (medium), gold (rare). 60.0 grid density vs 45.0 for outer-only stars.
- **Active tab glow:** Active tab icon now renders in gold (was ink color), with warm gold background glow (15% alpha) and persistent subtle sparkle shimmer (2 particles). Tab feels "shining" when selected.

**Files modified:** 3

| Feature | Before | After |
|---------|--------|-------|
| Chart background | Flat bg_color (dark brown/cream) | Galaxy gradient + nebula swirl + dense colored star field |
| Active tab | Bold icon + gold underline | Gold icon + gold glow bg + persistent sparkle + gold underline |

---

## v9.1.0 — "The Polish" (2026-04-29)

- **[P0] Disable chart rotation:** Natal chart no longer spins, making planetary positions readable. Removed `u.time * 0.015` rotation transform from natal_wheel_3d.wgsl
- **[P0] Backtest crash fix:** Removed early `return` from backtest view that swallowed entire astrology tab. Added "Clear Results" button so users can dismiss backtest output
- **[P0] Broken watchlist icons:** Replaced Unicode "✕" with Phosphor `X_LG` icon for watchlist remove buttons. Previous character rendered as broken box
- **[P0] Tooltip contrast fix:** OHLCV hover tooltip now uses dark background card (0.12/0.10/0.08 RGB) with warm cream text + gold border accent. Readable in both Parchment and Leather themes. Font size 9→10px, tooltip wider (90→106px)

**Files modified:** 6

| Bug | Before | After |
|-----|--------|-------|
| Natal chart | Slowly rotating, hard to read | Static, readable positions |
| Backtest results | Swallows entire tab, no dismiss | Shows inline with Clear button |
| Watchlist icons | "✕" renders as broken box | Phosphor X_LG icon |
| Chart tooltip | White text on light bg, tiny | Dark card + cream text, gold border |

---

## v9.0.1 — Hotfix (2026-04-29)

- **WGSL array fix:** Changed orbital trail `trail_alphas` from `let` to `var` — WGSL `let` arrays cannot be indexed by loop variable, only `var` arrays support dynamic indexing. Caused shader validation crash on launch.
- **Roadmap update:** Added v9.1/v10.0/v11.0 milestones from video review (15min, 284 subtitle entries, 903 frames analyzed)
- **TODOS overhaul:** 4 P0 bugs + 16 items organized by milestone version

---

## v9.0.0 — "The Performance" (2026-04-29)

- **Planet pulse/breathe:** Natal planets sinusoidally modulate radius + halo intensity with per-planet phase offset (1.7 rad stagger) for organic non-synchronized breathing
- **Orbital transit trails:** 5 ghost dots per transit planet at progressively earlier angular positions with fading alpha (0.08→0.60), comet-tail effect behind drifting transits
- **Aspect line shimmer wave:** Traveling alpha pulse along each aspect line, speed varies by type (conjunction 1.0, sextile 1.5, trine 2.0, square 4.0) — red squares shimmer fastest
- **Zodiac segment glow:** Active sign (containing current Sun transit) gets 30% brightness boost + subtle 1.5Hz pulse. Computed from Sun's ecliptic longitude / 30
- **Dust mote cursor interaction:** Vignette dust motes push away from mouse cursor within 0.15 UV radius. Cursor position passed via VignetteUniforms mouse_pos field
- **Candlestick chart draw-in:** On ticker switch, candles grow from price midpoint with left-to-right stagger (60% stagger / 40% growth over 500ms). Uses ease_out_cubic per candle
- **Layered page transitions:** Background settles first ~100ms (3× alpha speed), content follows over full 300ms. Gold glow fires during fast background phase
- **Tab sparkle tuning:** 8 particles (up from 5), varied sizes 1.5-4.0px, faster burst (0.08 stagger), downward gravity drift during fade
- **60fps astrology tab:** Tick model fix — `still_animating |= active_tab == Astrology` keeps shader_time advancing at 60fps when astrology tab visible
- **Uniform buffer growth:** NatalWheel3DUniforms 496→512 bytes (active_sign + padding). VignetteUniforms field reuse (mouse_pos replaces 2 pad floats, stays 64 bytes)

**Files modified:** 12

| Feature | Before (v8.0.0) | After (v9.0.0) |
|---------|-----------------|----------------|
| Natal planets | Static gold dots | Breathing pulse (sinusoidal radius modulation) |
| Transit planets | Single dot per planet | Dot + 5-ghost orbital trail fade |
| Aspect lines | Solid colored segments | Shimmer wave (traveling alpha pulse) |
| Zodiac ring | Uniform brightness | Active sign highlighted + pulsing |
| Dust motes | Lissajous drift only | Cursor-reactive repulsion |
| Candlestick chart | Instant render | Grow-from-midpoint staggered draw-in |
| Page transitions | Flat 250ms fade | Layered 300ms (fast bg + delayed content) |
| Tab sparkle | 5 particles, fixed | 8 particles, gravity drift, varied sizes |
| Astrology tick | 30s when idle | 60fps when tab visible |

**Project stats:** ~20,800 Rust+WGSL source | 2 GPU shaders | 4 canvas ornaments | 70 tests | 0 warnings

---

## v8.0.0 — "The Observatory" (2026-04-28)

- **3D natal chart shader:** Replaced Canvas-based `NatalWheel` with GPU-rendered `NatalWheel3DProgram` using procedural SDF fragment shader
- **Perspective-tilted zodiac:** Y-axis foreshortening + slow rotation creates convincing 3D tilted disc without vertex buffers
- **12 colored zodiac segments:** Element-based sign colors (fire/earth/air/water) rendered via SDF arc regions with anti-aliased edges
- **Glowing planet dots:** Natal planets = gold halos + hot center core, transit planets = blue/red with 0.5°/sec animated drift
- **Aspect line computation in WGSL:** Natal×transit O(n²) loop computes conjunction/sextile/square/trine with correct orbs (8°/6°/8°/8°)
- **Directional lighting:** Top-bright/bottom-dark gradient simulates overhead illumination on tilted disc
- **Rim glow:** Pulsing gold shimmer on outer ring edge (0.22 intensity, 0.10 radius), sinusoidal time modulation
- **Star field:** Twinkling procedural stars outside zodiac ring using hash-based noise + sinusoidal twinkle
- **Perspective tuning:** 32% Y-foreshortening (camera_tilt=0.32) for pronounced 3D depth
- **496-byte uniform buffer:** 13 natal + 13 transit planets packed as `[[f32; 4]; 13]` arrays with longitude, retrograde flag, planet index

**Files modified:** 4 + 1 new

| Feature | Before (v7.6.0) | After (v8.0.0) |
|---------|-----------------|----------------|
| Natal chart renderer | Canvas 2D (`canvas::Program`) | GPU shader (`shader::Program`) |
| Ring perspective | Flat circle | Tilted ellipse (32% foreshortening) |
| Zodiac segments | Canvas arc paths | SDF-rendered anti-aliased arcs |
| Planet rendering | Canvas circles + text glyphs | SDF dots + glow halos (no text) |
| Aspect computation | Rust loop in `draw()` | WGSL loop in fragment shader |
| Background | Solid fill | Star field + vignette |
| Animation | Transit drift only | Drift + rotation + rim pulse + star twinkle |

---

## v7.6.0 — "The Consistency" (2026-04-28)

- **Gold sub-scrollbar styling:** Extracted `gold_scrollbar_style` helper, applied to all 15 data-table scrollables across 4 view files
- **Concordance column fix:** Universe table "Conc" column width 50→90px, "Strong Confirm" no longer truncated
- **Animated transit ring:** Transit planets drift 0.5°/sec on natal chart, driven by `shader_time` — "heavens in motion" effect
- **Canvas sparkle particles:** Replaced Unicode ✦ with canvas-rendered gold particle burst (5 dots, staggered fade-in per tab)
- **Fetch error guidance:** "Scraper not found" message now includes `cargo build --bin scraper` instruction
- **Ornament contrast (v7.5.1):** Boosted alpha on all 3 canvas ornaments for Parchment theme visibility
- **Ticker-specific empty states (v7.5.1):** 8 "for this ticker" messages now interpolate actual ticker name

**Files modified:** 12

| Feature | Before (v7.5.0) | After (v7.6.0) |
|---------|-----------------|----------------|
| Sub-scrollbars | Default gray | Gold scroller, translucent rail |
| Concordance | Truncated at 50px | Full text at 90px |
| Transit ring | Static positions | 0.5°/sec animated drift |
| Tab sparkle | Unicode ✦ character | Canvas particle burst |
| Ornaments | Low alpha (barely visible) | Boosted alpha (visible on cream) |
| Empty states | "for this ticker" | "for AAPL" / "for MSFT" |

---

## v7.5.0 — "The Polish" (2026-04-28)

- **Scrollbar styling:** Gold scroller on translucent rail, right padding prevents content overlap
- **Fetch error display:** Persistent orange warning banner for errors, pre-flight scraper check, gold loading bar
- **Gauge grid:** 5 gauges in 3+2 grid layout (two rows), no horizontal scrollbar
- **Leather vignette warmth:** `grimoire_outer_bg()` multipliers increased (0.15→0.25), shader center brightened (1.2→1.5)
- **Natal chart beautified:** Element-colored zodiac ring segments, gold glow halos on natal planets, planet glyphs, 300→400px canvas
- **Tab sparkle:** Gold ✦ character fades in during hover with delayed alpha ramp
- **Active tab visibility:** Gold-colored label always visible, 3px gold underline, surface background. Three-tier: active/hovered/default

**Files modified:** 8

| Feature | Before (v7.4.1) | After (v7.5.0) |
|---------|-----------------|----------------|
| Scrollbar | Default, overlaps | Gold scroller, right padding |
| Fetch errors | Toast only | Persistent banner + pre-check |
| Gauges | Horizontal scroll | 3+2 grid |
| Natal chart | 300px, flat | 400px, colored zodiac, glyphs |
| Active tab | Icon only, 2px | Gold label visible, 3px |

---

## v7.4.1 — "The Grimoire — Header Redesign" (2026-04-28)

- **Horizontal tab bar:** Moved 8 tabs from right-side vertical strip to horizontal bar under header ornament
- **Icon-only at rest:** Tabs show icon only, label fades in on hover via `tab_hover_progress` animation
- **Gold bottom underline:** Active tab gets 2px gold bottom border + surface background
- **Transparent button chrome:** Custom `button::Style` with `background: None` so container styling shows through
- **Layout simplification:** `row![spine, book_page]` — right-side dark strip removed entirely
- **Dead code cleanup:** `build_grimoire_tabs()` deleted (~110 lines), replaced by `build_tab_bar()`

**Files modified:** 1 (`src/dashboard/view/mod.rs`)

| Feature | Before (v7.3–7.4) | After (v7.4.1) |
|---------|-------------------|----------------|
| Tab position | Right-side vertical column | Horizontal bar, top of page |
| Tab shape | Square containers + stagger | Inline icons + gold underline |
| Layout | `row![spine, page, tabs]` | `row![spine, page]` (tabs inside page) |

---

## v7.4.0 — "The Atmosphere" (2026-04-28)

- **GPU vignette shader:** Radial darkening (lighter center, dark edges) via wgpu `Shader` widget
- **Noise grain:** Static hash-based texture, luminance-adaptive strength
- **Dust motes:** 12 procedural golden particles with Lissajous drift, frozen at idle
- **Gold edge glow:** Book border glows gold during page transitions
- **Stack compositing:** `stack![vignette_shader, padded_book]` replaces flat container
- **Power-efficient:** `shader_time` only advances during 16ms animation ticks
- **Bug fix:** Parchment vignette too dark (LCD purple distortion below RGB 0.05)
- **Bug fix:** Tab icon colors inverted in both themes

**Files modified:** 7 + 2 new (`shaders/mod.rs`, `shaders/vignette.wgsl`)

---

## v7.3.0 — "The Grimoire" (2026-04-27)

- **Right-side book tab dividers:** 8 tabs moved from horizontal top bar to right-side vertical column, styled as physical book dividers with staggered cascade
- **Hover-to-expand tabs:** `mouse_area` hover detection — icon-only (48px) expands to icon+label (168px) on hover with `ease_out_back` elastic overshoot animation
- **Dark atmospheric outer frame:** Deep circadian-aware background behind book (grimoire_outer_bg)
- **Book spine:** Canvas-rendered vertical binding strip with cross-stitch marks and diamond endcaps
- **Page header ornament:** Canvas Renaissance-style flourish with central lozenge, sine-wave scrollwork, extending rules
- **Page border corners:** Canvas decorative corner brackets with perpendicular arms and gold diamond vertices
- **Page transition:** 250ms "materializing from darkness" fade-in when switching tabs
- **Compact navigation:** Merged header + nav into single slim row, reduced chrome
- **New easing:** `ease_out_back` (elastic overshoot) for playful game-feel interactions

**Files modified:** 9 + 1 new (`ornaments.rs`)

| Feature | Before (v7.2) | After (v7.3) |
|---------|---------------|--------------|
| Tab position | Horizontal top bar | Right-side vertical dividers |
| Tab hover | None | Icon→icon+label expand animation |
| Layout | column![header, tabs, content] | row![spine, book_page, grimoire_tabs] |
| Outer frame | None | Dark atmospheric background |
| Decorations | None | Canvas spine, header ornament, corner brackets |
| Tab switch | Instant | 250ms page transition fade |

---

## v7.2.0 — "The Motion" (2026-04-27)

- **Phosphor Icons:** Replaced Bootstrap Icons with Phosphor (1530 icons, regular + bold weights)
- **Animation infrastructure:** Easing functions, adaptive tick (16ms/60fps during animation, 30s at rest)
- **Gauge sweep:** Fear/Greed needle sweeps old→new score over 600ms (ease_out_cubic)
- **Toast fade-out:** Opacity fades 1.0→0.0 over last 500ms of lifetime
- **Tab indicator crossfade:** Gold underline fades between tabs over 200ms
- **Responsive font scaling:** Viewport-aware auto-scale (<1024px: 0.85, 1440+: 1.05, 1920+: 1.1)
- **Bug fix:** TECHNICAL INDICATORS vertical text wrapping (6→2×3 grid layout)
- **Bug fix:** Recently-viewed overflow (capped at 6)
- **Version control:** Cargo.toml synced to actual version, git tags created, CHANGELOG.md added

**Files modified:** 13 + 3 new

---

## v7.1.0 — "The Ledger — Spatial Polish" (2026-04-27)

- Fix paper trail buy threshold text 75 → 65
- Spacing constants (`SPACE_XS/SM/MD/LG/XL`, `MAX_WIDTH`, `RADIUS_CARD`) + layout primitives (`max_container`, `eyebrow`, `section_rule`)
- Compact 2-row header (~200px → ~80px), remove status text
- 1240px max-width centered container
- 38 eyebrow labels + ~30 section rules across all 8 tabs
- ~65 `.font(font::INTER)` on numeric values across 7 files
- Overview restructure — vertical flow, full-width 300px hero chart

**Files modified:** 11 (`theme.rs`, `shared.rs`, `mod.rs`, + 8 view files)

---

## v7.0.0 — "The Ledger" (2026-04-26)

- LedgerPalette engine: 11-channel semantic palette with RwLock cache
- 8 anchor palettes (4 Parchment + 4 Leather) with 24-stage circadian lerp
- ThemeMode: Auto/Parchment/Leather (removed TokyoNight)
- Circadian preview slider in Settings (0-23 hour override)
- Four-role typography: Fraunces (display), Source Serif 4 (body), Inter (numerics), JetBrains Mono (tabular)
- Shared component restyling: card borders, gold tab indicator, toast overlay
- 43+ heading instances updated to `.font(font::DISPLAY)`

---

## v6.2.0 — "The Priority Queue" (2026-04-26)

- `collect_priority_tickers()` merges paper positions into fetch pipeline
- All Phase 2 data sources (sentiment, Finnhub, short, EDGAR) cover paper tickers
- Tiingo bulk SQL includes paper_portfolio at tier-0 priority

---

## v6.1.0 — "The Benchmark" (2026-04-26)

- SPY benchmark comparison (Sharpe, max drawdown, alpha)
- NYSE holiday calendar (2022-2030) — no trades on market holidays
- 25% position cap rebalancing
- 15% trailing stop-loss exits
- Win rate, avg holding days, closed trade statistics

---

## v6.0.0 — Paper Trading Engine (2026-04-26)

- Paper trading account with $100K initial capital
- Buy when Lagrange score > threshold, sell when < 35
- Equity curve chart with daily portfolio valuation
- Trade log, open positions, performance statistics
- Paper Trail tab in dashboard

---

## v5.0.1 — Polish (2026-04-25)

- Replace `.unwrap()` with safe extraction in SetAgentMode handler
- UTF-8 safe error truncation in LLM API path
- `fetch_single_ticker` now returns Result for proper exit codes
- Reset notification flag on mark-all-read
- Replace raw Color values with theme constants
- Log portfolio import errors instead of silently discarding

---

## v5.0.0 — "The Council" (2026-04-25)

- Eliminated all 47 compiler warnings
- "Fetch this ticker" button — dashboard spawns scraper subprocess with `--ticker` CLI mode
- LLM-backed agent analysis via Anthropic Claude API with Template/LLM mode toggle

---

## v4.2.0 — "The Expansion" (2026-04-24)

- OHLC candlestick charts replacing area fill
- Black-Scholes Options Greeks calculator + IV solver
- Server-side sortable Universe table (6 columns)
- In-app toast notifications
- GDELT geopolitical events in Research tab

---

## v4.1.0 — "The Glass" (2026-04-24)

- Catppuccin Mocha/Latte/Tokyo Night theme system
- Bootstrap Icons integration
- Card-based layout with section headings
- Fear & Greed gauge widgets
- Keyboard shortcuts (Ctrl+1..7 tabs, Ctrl+T search, Ctrl+R refresh)

---

## v4.0.0 — "The Forge" (2026-04-23)

- Modular update dispatcher (5 domain files)
- Extracted helpers, db modules, view modules
- Removed dead code, fixed clippy warnings
- Moshier ephemeris fix (NaN and state corruption)

---

## v3.1.x — "The Network" (2026-04-22 to 2026-04-23)

- Strategy builder, backtesting, transaction log
- Named watchlists, portfolio P&L
- Concordance detection, extended aspects
- Font scale setting, astro priority scrape
- Polymarket prediction markets integration
- RSS news aggregation from 25 sources
- DBnomics international economics scraper

---

## v3.0.x — Bug fixes and UX (2026-04-22)

- 6 bug fixes + 4 UX improvements
- Astrology engine (Swiss Ephemeris, natal charts, transits, horoscopes)
- Lagrange composite scoring system
- Universe Explorer with 1,700+ tickers
- Insider trades, filings, holdings, earnings, sentiment
- DCF intrinsic value calculator

---

## v1.0.0 — v1.1.0 (2026-04-22)

- Enrichment pipeline, Tiingo integration, alerts, recently-viewed
- Theme color tokens, chart/sparkline theming
- Type scale, section hierarchy, table spacing

---

## v0.6.0 — v0.7.0 (2026-04-08 to 2026-04-16)

- Lagrange history sparkline, portfolio tracker, CPI YoY%
- Lagrange Score, expanded data sources, signal intelligence

---

## v0.1.0 — Scaffold (2026-04-07 to 2026-04-08)

- Scaffold two-binary financial dashboard (scraper + Iced GUI)

---

## Project Stats (v8.0.0)

| Metric | Value |
|--------|-------|
| Commits | 50+ |
| Rust source | ~20,500 lines across 2 binaries |
| SQL migrations | 32 |
| Tests | 70 (48 lib + 17 dashboard + 5 scraper) |
| Compiler warnings | 0 |
| Crate deps | 26 |
| Font assets | ~2.7MB (Fraunces, Source Serif 4, Inter, JetBrains Mono, Phosphor, Phosphor Bold) |
| GPU shaders | 2 (vignette.wgsl, natal_wheel_3d.wgsl) |
| Canvas widgets | 4 (BookSpine, PageHeaderOrnament, PageBorderCorner, TabSparkle) |
| Git tags | 9 (v4.0.0 - v7.3.0) |
| Development | 22 days (Apr 7 - Apr 28, 2026) |