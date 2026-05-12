# v11.5 Research: "The Explanations"

**Date:** 2026-05-05
**Source:** 27-min video review, transcript at `docs/video-review-v11.4-transcript.txt`
**Scope:** 22 distinct feedback items across 6 sub-waves, ~10 days total

---

## Source Material

- **Video:** `Screen Recording 2026-05-05 024144.mp4` (27 min, 2560×1540)
- **Frames:** 54 sampled at 30s intervals (deleted, were in `/tmp/watch-review/`)
- **Audio transcript:** 331 lines, faster-whisper base model, int8 CPU
- **Frame review summary:** all Wave 6 features visibly working — Sun/Moon/Rising populated, planet glyphs + tooltips, data freshness badges, sector dropdown, header price, two-column astrology layout, theme switching all confirmed

## Wave 6 features explicitly approved in audio

| Feature | Quote | Timestamp |
|---------|-------|-----------|
| Aspect line thinning (1a) | *"The lines of thinner like that — yeah this is a bit better"* | 00:26-00:37 |
| Galaxy mute (1e) | *"This is giving more accurate needle charting"* | 00:57 |
| Council diversification (4a) | *"I like the fact they're having their own opinions on things"* | 19:02 |
| Council verdicts | *"I'm liking what I'm saying already, that's nice"* | 20:46 |
| Horoscope reading (2c+2d) | *"I like this so far"* | 21:27 |
| Tab transitions | *"Better way better way better, I prefer that"* | 06:22 |
| Sound on alerts | *"The prop sounds there, that's good"* | 06:19 |

## Why "explanations" dominates feedback

Quoted phrases in transcript by approximate count:
- "should have a pop-up explaining" — 8 mentions
- "I don't know what this means" — 5 mentions
- "explain" / "explained" / "explanation" — 19 mentions
- "tooltip" / "pop-up" — 22 mentions

Game-tutorial model explicitly requested at 17:06: *"Like in a lot of games when you hover something a pop-up appears, and then you could right-click and choose an option to explain."* Right-click → external link (Wikipedia, Investopedia, FRED) for deep dives.

## Sequencing rationale

Three options considered:

1. **Quick-wins first** (rejected) — ship tooltips immediately, then reshuffle layout, but every tooltip would need rewiring after layout changes. Rework cost: ~30%.
2. **Polish-first** (rejected) — candle labels and loading % feel safest but don't address dominant feedback theme. Optimizes for low risk, not user value.
3. **Foundation → Layout → Content → Interactivity → New → Polish** (chosen) — A unblocks C/F, B is highest-risk so do early before wave-on-wave dependency builds, D and E are isolated and can run in parallel if needed.

Each phase scoped to ship independently — break point after any sub-wave is acceptable if priorities shift.

---

## v11.5.A — "The Foundation" (~0.5 day)

### A1 — Tooltip helper in `view/shared.rs`

Wave 3b's `tip_style()` was inlined into `universe.rs`. Generalize:

```rust
// view/shared.rs
pub fn explain_tooltip<'a>(
    inner: Element<'a, Message>,
    label: &'static str,
    detail: &'static str,
) -> Element<'a, Message> {
    let p = theme::palette();
    let info_icon = text(icons::INFO_CIRCLE.to_string())
        .font(icons::PHOSPHOR)
        .size(theme::text_xs())
        .color(Color { a: 0.5, ..p.gold });
    let trigger = row![inner, info_icon].spacing(2).align_y(Alignment::Center);
    tooltip(
        trigger,
        container(text(detail).size(theme::text_xs()))
            .padding([6, 10])
            .style(|_t: &iced::Theme| {
                let p = theme::palette();
                container::Style {
                    background: Some(iced::Background::Color(p.surface)),
                    border: iced::Border { color: p.gold, width: 1.0, radius: 3.0.into() },
                    ..Default::default()
                }
            }),
        tooltip::Position::Bottom,
    ).into()
}
```

Use case: `explain_tooltip(text("Astro"), "Astro", "Astrological score 0-100 derived from active aspects between transit and natal planets")`.

### A2 — Right-click context menu primitive

Iced 0.14 doesn't have native right-click menus but `mouse_area::on_right_press` exposes the event. State-tracked overlay positioned at click point:

```rust
struct ContextMenuState {
    visible: bool,
    position: Point,
    items: Vec<ContextMenuItem>,
}

struct ContextMenuItem {
    label: &'static str,
    action: ContextAction,
}

enum ContextAction {
    OpenUrl(&'static str),    // external link to Wikipedia/Investopedia/FRED
    ShowDetail(&'static str),  // long-form explanation in modal
}
```

State lives on `Dashboard` struct. `mouse_area(content).on_right_press(Message::ShowContextMenu(point, items))`. Render absolute-positioned overlay using `Pin` widget (Wave 3e infrastructure).

### A3 — Document pattern

Add module-level doc on `view/shared.rs::explain_tooltip` with template. Future tooltips just call helper, no per-call style code.

---

## v11.5.B — "The Layout" (~2.5 days, high risk)

### Spec from transcript 09:25-11:24

```
┌─────────────────────────────────────────────────────────┐
│  [🔍 Search any ticker...    ]   [⟳] [↓] [☾]          │ ← Search TOP
│  Recent: AAPL MSFT TSLA META   [⭐ Favorites ▼]        │ ← Recently → Favorites dropdown
├─────────────────────────────────────────────────────────┤
│              AAPL  $277.18  H 280.03  L 276.92          │ ← Ticker MIDDLE
│              ☉ Sun: Sag · ☽ Moon: Aqu · ↑ Rising: Cap   │ ← Big Three (already there)
├─────────────────────────────────────────────────────────┤
│              [decorative ornament]                       │
├─────────────────────────────────────────────────────────┤
│  [Astrology] [Overview] [Universe] [Fundamentals] ...   │ ← Tab bar (unchanged)
└─────────────────────────────────────────────────────────┘
```

Currently swapped: search is mid-band, ticker is on top. Need to literally swap.

### B1+B2 — Search/ticker swap

`view/mod.rs::view()` has the row construction. Reorder.

### B3 — Favorites dropdown

Currently `recently_viewed_row` is inline button list. Wrap in `pick_list` or custom dropdown widget:
- Trigger: dark "⭐ Favorites" button with caret
- On click: opens panel showing recently-viewed + bookmarked
- New state field: `Vec<String> bookmarked_tickers` persisted via existing settings table
- Migration `0044_bookmarked_tickers.sql` (if going with row-based persistence)

### B4 — Zodiac legend relocation

Currently at bottom of natal chart (`build_wheel_legend()` rendered after Shader). User wants it above. Reorder in `view/astrology_tab.rs::wheel_col` column.

### B5 — Gauges side-by-side

In `view/overview.rs`, the 5 gauges (Crypto/Equities/Score + Astrology/Lagrange) currently stacked 3+2. Change to single horizontal `row![]` of 5. May need `Length::FillPortion(1)` for each + horizontal scrollable on narrow windows.

### B6 — Settings as modal overlay

Currently Settings is a `Tab::Settings` view that takes the entire content area. User wants modal triggered from gear icon.

**Approach:** add `Message::ToggleSettings`, state field `settings_open: bool`. When true, render Settings as `stack![main_view, settings_modal]` where modal is centered, dimmed backdrop. Keep `Tab::Settings` for now as fallback.

### B7 — XL text size layout fix

Transcript at 08:51: *"This is now broken"* after switching to XL text. Frame analysis didn't capture the broken state. Likely: text overflow in tab labels or ticker name pushes layout. Investigate by manually setting XL and screenshotting.

Mitigations:
- Tab labels truncate with ellipsis
- Header row uses `wrap()` if too narrow
- Ticker name shrinks to fit available width

---

## v11.5.C — "The Explanations" (~2.5 days, content-heavy)

12 tooltip applications. Pure additive. Per item:

| ID | Location | Tooltip text | Source |
|----|----------|--------------|--------|
| C1 | Universe `Sector` header | "Industry classification (GICS sector)" | [05:10] |
| C1 | Universe `Score` header | "Lagrange composite — 35% astro + 30% fin + 20% macro + 15% sentiment" | [05:16] |
| C1 | Universe `Astro` header | "Astrological score 0-100 from active transit-natal aspects" | [05:16] |
| C1 | Universe `Zone` header | "Lagrange zone: Optimal (>70), Favorable (55-70), Neutral, Unfavorable, Misaligned" | [05:16] |
| C2 | Astrology Transits `A/S` header | "Applying (energy building) vs Separating (energy fading)" | [08:00] |
| C3 | Sector heatmap legend | "Sectors color-graded by average astro score across constituent tickers" | [05:10] |
| C4 | Crypto/Risk Sentiment gauge | "Composite of crypto market signals — VIX, fear/greed indicators" | [12:00] |
| C4 | Equities Sentiment gauge | "Composite of equity sentiment — RSI averages, sector momentum" | [12:00] |
| C4 | (Ticker) Score gauge | "Lagrange composite for this ticker only" | [12:00] |
| C4 | (Ticker) Astrology gauge | "Astrological score for this ticker — derived from natal+transit aspects" | [12:00] |
| C4 | (Ticker) Lagrange gauge | "Combined astro + financial composite — directional confidence indicator" | [12:00] |
| C5 | FRED indicator labels (CPI, Unemployment, etc.) | label-specific descriptions + link to FRED page | [17:24] |
| C6 | Lagrange `Mild Confirm` etc. labels | "Astro and financial signals agree" / "Mild disagreement" / "Sharp divergence" | [21:02] |
| C7 | Backtest `signal_accuracy_pct` row | "Win rate vs random — 50% is baseline, >55% is signal, >65% is strong" | [20:36] |
| C8 | Council verdict labels | "Buffett's threshold: ROE>15% + DE<1.0 + FCF positive" etc. per persona | new |

### Right-click context (A2) wired into priority items

Items C5 (FRED) and C8 (Council) get right-click → opens link to FRED.org or persona-specific Wikipedia page.

---

## v11.5.D — "The Interactions" (~2.5 days, isolated risk)

### D1 — Aspect line hover (HIGHEST TECHNICAL RISK)

**The problem**: aspect lines (cyan/red threads on natal wheel) are rendered in `natal_wheel_3d.wgsl` as fragment shader output. Iced has no awareness of where each line is on screen — the GPU computes pixel-by-pixel and Iced sees only the final composited image. Hover detection requires CPU-side knowledge of line positions.

**Two approaches**:

**Approach A — Invisible Iced overlay rectangles (lower effort)**
- Compute aspect line endpoints in Rust (already have planet positions + `planet_pixel_pos()` from Wave 3e)
- For each aspect, compute midpoint + length + angle
- Place small invisible `mouse_area` rectangle along each line (center, oriented)
- On enter: tooltip shows aspect type + planets + orb
- Pros: ~150 LOC, builds on Wave 3e infrastructure
- Cons: clickable area is an axis-aligned rect, not the diagonal line itself — hover is approximate (within ~10px)

**Approach B — Move aspect lines to Iced Canvas widget (higher effort)**
- Render aspect lines as `canvas::Path` in 2D Canvas widget overlaid on shader (via `stack![shader, canvas]`)
- Canvas has built-in hit-testing
- Lines as actual diagonal hit zones
- Pros: precise hover, exact line geometry
- Cons: ~400 LOC, Canvas re-render every frame, may impact performance

**Recommendation: ship Approach A first.** Validate user satisfaction. Upgrade to B if approximate hover proves frustrating.

### D2 — Mouse-wheel zoom

Iced 0.14 has `mouse_area::on_scroll(|delta| Message::ZoomChart(delta.y))`. Wire to:
- Increment/decrement `chart_size` field (currently enum {Compact, Default, Large})
- Could either: (a) cycle enum on scroll, or (b) replace enum with continuous f32 multiplier 0.5-2.0
- (b) is more natural but breaks Wave 4b's pick_list — keep enum + add scroll bonus multiplier separately

```rust
// new state
pub chart_zoom_bonus: f32,  // 0.5 to 2.0, multiplied into chart_size.pixels()

// on scroll
state.chart_zoom_bonus = (state.chart_zoom_bonus + delta.y * 0.05).clamp(0.5, 2.0);
```

### D3 — OS-level toast notifications

```toml
# Cargo.toml
notify-rust = "4"
```

```rust
// In Dashboard handler when AlertFired triggered
use notify_rust::Notification;
Notification::new()
    .summary("Nisaba Capital Charting-Finance")
    .body(&format!("{} entered Optimal zone (score {:.0})", ticker, score))
    .icon("dialog-information")
    .timeout(5000)
    .show()
    .ok();
```

Cross-platform: Windows Toast, macOS UserNotifications, Linux libnotify.

### D4 — Multi-monitor verification

Manual test step. Check that toasts appear on the active monitor + that Iced dashboard isn't lost behind other windows when an alert fires.

---

## v11.5.E — "The Encyclopedia" (~1.5 days, new feature)

### E1 — Wikipedia REST scraper

```rust
// src/scraper/wikipedia.rs
pub async fn fetch_summary(client: &reqwest::Client, title: &str) -> Result<WikiSummary> {
    let url = format!(
        "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
        urlencoding::encode(title)
    );
    let resp = client.get(&url)
        .header("User-Agent", "PursuitDashboard/11.5 (https://github.com/...)")
        .send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("Wikipedia HTTP {}", resp.status());
    }
    resp.json().await.context("Wikipedia JSON parse")
}

#[derive(Deserialize)]
pub struct WikiSummary {
    pub title: String,
    pub extract: String,             // 200-word summary
    pub thumbnail: Option<Image>,
    pub originalimage: Option<Image>,
    pub content_urls: ContentUrls,
}
```

### E2 — Migration `0043_wiki_summary.sql`

```sql
CREATE TABLE IF NOT EXISTS wiki_summaries (
    ticker         TEXT PRIMARY KEY,
    wiki_title     TEXT NOT NULL,
    extract        TEXT,
    thumbnail_url  TEXT,
    image_url      TEXT,
    wiki_url       TEXT,
    last_fetched   TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_wiki_freshness ON wiki_summaries(last_fetched);
```

### E3 — New `Tab::Encyclopedia` variant

```rust
// src/dashboard/tabs.rs
pub enum Tab {
    Astrology, Overview, Universe, Fundamentals, Research,
    Encyclopedia,  // ← new
    Portfolio, PaperTrail, Settings,
}

impl Tab {
    pub fn icon(self) -> char {
        match self {
            // ...
            Tab::Encyclopedia => icons::BOOK_OPEN,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            // ...
            Tab::Encyclopedia => "Wiki",
        }
    }
}
```

`icons::BOOK_OPEN` may need adding to `view/icons.rs` (Phosphor codepoint U+E0A6).

### E4 — `view/encyclopedia.rs`

```rust
pub fn view_encyclopedia(&self) -> Element<'_, Message> {
    let summary = self.wiki_summary.as_ref();
    match summary {
        Some(s) => column![
            icon_eyebrow(icons::BOOK_OPEN, "ENCYCLOPEDIA"),
            // Two-column: image left, text right
            row![
                image(s.image_url.clone()).width(Length::Fixed(200.0)),
                column![
                    text(&s.title).font(font::DISPLAY).size(theme::text_lg()),
                    text(&s.extract).size(theme::text_base()),
                    link_button("Read more on Wikipedia →", &s.wiki_url),
                ].spacing(8),
            ].spacing(20),
        ].into(),
        None => text("No Wikipedia article found for this ticker.").into(),
    }
}
```

### E5 — Refresh pipeline

Wire into scraper's full pipeline + single-ticker pipeline:

```rust
// scraper/main.rs run_all_fetches phase 3.13 (after EDGAR enrichment)
println!("3.13 Fetching Wikipedia summaries (30-day TTL)...");
wikipedia::fetch_stale_summaries(Arc::clone(&pool), Arc::clone(&client)).await;

// fetch_single_ticker phase 6
if !wikipedia::is_fresh(&pool, ticker, 30).await {
    let _ = wikipedia::fetch_one_and_store(Arc::clone(&pool), Arc::clone(&client), ticker).await;
}
```

### E6 — Cache-miss UI

Some tickers won't match a Wikipedia article (small caps, recent IPOs). Show:
- "No Wikipedia article found for {ticker}"
- "Check back after the company gets coverage"
- Link to Google search as fallback

---

## v11.5.F — "The Polish" (~1 day)

### F1 — Candle tooltip word labels

Currently `O:`, `H:`, `L:`, `C:`, `Vol:`. Change to `Opening`, `High`, `Low`, `Closing`, `Volume`.
File: `src/dashboard/charts.rs` PriceChart::draw tooltip text construction.

### F2 — Volume sub-tooltip

Volume label gets nested tooltip: "shares traded that day — high volume confirms price moves."

Tricky: tooltip-within-tooltip not native to Iced Canvas. Workaround: just include "(shares traded)" inline text after Volume label. User wanted explanation; static text suffices.

### F3 — Loading bar numeric %

Currently bar fill only. Add text overlay: `format!("{:.0}%", progress * 100.0)`.

### F4 — Sparkle particles on loading bar

User said *"sparkly, like a little sparkle"* [13:38]. Reuse `TabSparkle` Canvas widget pattern from v7.6 — render ~3 sparkles drifting along the loading bar.

### F5 — Stuck-at-85% fix

Current Wave 4c caps at 0.85 until `FetchTickerComplete` arrives. If scraper takes >30s, bar appears frozen. Fix:
```rust
let progress = if elapsed > 30s {
    0.85 + ((elapsed - 30s) / 60s).min(0.10)  // creep 0.85→0.95 over 60-90s
} else {
    (elapsed / 30s).min(0.85)
};
let label = if elapsed > 60s { "still fetching..." } else { "" };
```

### F6 — Dedupe loading indicators

Audit `view/mod.rs` for places that render `fetching_ticker` indicator. There may be both pulsing bar + spinner shown simultaneously.

### F7 — Strategy backtest smart defaults

Currently inputs show `65` and `35` placeholders. Compute from ticker's actual astro distribution:
- Pull last 90 days of `astro_scores` for ticker
- Compute median + stddev
- Buy threshold = median + 0.5×stddev (default rounded)
- Sell threshold = median - 0.5×stddev

Implementation: when ticker selected, pre-populate `backtest_buy_input` and `backtest_sell_input` from DB query.

---

## Dependency Graph

```
A (foundation)
  ├──→ C (tooltips)
  └──→ F4-F5 (loading polish reuses sparkle/state primitives)

B (layout)
  ├──→ all subsequent (stable layout assumed)

C (tooltips)
  └─ depends on A1 helper

D (interactivity)
  ├─ D1 independent (chart only)
  ├─ D2 independent (chart only)
  ├─ D3 independent (system layer)
  └─ D4 depends on D3

E (Wikipedia tab)
  └─ fully independent — could ship anytime

F (polish)
  ├─ F1-F2 independent (charts.rs)
  ├─ F3-F6 depend on B6 if Settings refactor changes loading state
  └─ F7 depends on existing astro_scores data
```

**Parallelization opportunity:** D and E could ship in parallel (different files, no overlap). If team grows, D + E in parallel = save 1.5 days.

## Won't Do (in v11.5)

- ❌ Pinch-to-zoom on touch — Iced 0.14 lacks gesture API; mouse-wheel covers same need
- ❌ Re-architect aspect lines as full Canvas widget (D1 Approach B) — only if Approach A proves unsatisfying
- ❌ LLM-generated tooltip explanations — static strings sufficient, LLM adds latency + complexity
- ❌ Right-click on every element — only on items with valuable external links (FRED, Wikipedia)
- ❌ Pinned/floating Settings panel — modal is enough

## Open Questions

1. **Favorites persistence** — settings table or new `bookmarks` table? Likely settings table with comma-separated tickers, simpler.
2. **Wikipedia article matching** — exact company name match often fails (e.g., "Apple Inc." vs "Apple"). Wikidata Q-IDs (already stored) are reliable. Use those when available, fall back to fuzzy company name match.
3. **OS toast deduplication** — if 6 tickers enter Optimal at once, we'd fire 6 toasts. Throttle or batch? Probably batch into "6 tickers entered Optimal: AIIA, APXT, BLSH..." single toast.
4. **D1 Approach A bounding box angle** — Iced's `mouse_area` is axis-aligned. Diagonal aspect lines need a thicker approximation. Empirically choose 16-20px wide rectangles oriented along midpoint angle (won't be tilted, but covers the line + small slop).

## Implementation Order Recap

1. v11.5.A (0.5d) — Build foundation
2. v11.5.B (2.5d) — Reshuffle layout while we still can
3. v11.5.C (2.5d) — Apply tooltips on stable layout
4. v11.5.D (2.5d) — Add interactivity to stable astrology tab
5. v11.5.E (1.5d) — Independent new tab (could be 4 if D feels risky)
6. v11.5.F (1.0d) — Polish coat
7. **Then Wave 7** (OpenBB native providers) once user has stable v11.5

Total v11.5 estimate: ~10.5 days. Conservative buffer: ~12 days for one developer.
