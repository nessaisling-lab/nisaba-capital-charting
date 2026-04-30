# TODOS

## Open — Polish

### Clickable entity links — insider names, holders (v9.2 video review)
**What:** Insider trade names and institutional holder names should be clickable. "Click on this and it does a Google search or LinkedIn."
**Fix:** Wrap entity names in clickable text that calls `open::that(url)` to launch browser with Google/LinkedIn search for that person/institution.

### Chart layer visibility toggles (v9.1 video review)
**What:** Buttons to show/hide natal planets, transit planets, aspect lines, retrograde markers independently. Helps readability when chart is dense. "Select things to see which is which."
**Fix:** Add toggle state bools to Dashboard (show_natal, show_transit, show_aspects, show_retro). Pass as uniforms or control which sections render in view. Add toggle buttons near chart legend.

### Background texture — more earthy/Renaissance (v9.1 video review)
**What:** Background should feel more textured and earthy. "More of a Renaissance book feeling."
**Status:** Grain exists in vignette shader but not enhanced with warm color variation or parchment-like banding.
**Fix:** Add warm color noise variation to grain, subtle paper fiber pattern.

### Tab glow rework — bookmark style with gold border (v9.2 video review)
**What:** Current tab glow (gold bg tint + sparkle) works but user wanted "shining" bookmark-tab shape with gold border. "This is not what I mean by glow."
**Status:** Gold icon + gold bg + sparkle done (v9.2). Bookmark shape + gold border not yet.
**Fix:** Replace 15% alpha gold bg with strong gold border (2-3px) around active tab. Bookmark-tab shape.

## Open — Future

### Nav layout full redesign
**What:** Search bar position, buttons as icons with hover labels, ticker under buttons, "Recent" placement.
**Scope:** Significant layout change to header/nav area. Needs design mockup first.

### House numbers on natal chart
**What:** Add house numbers (1-12) to the 3D natal chart.
**Challenge:** WGSL has no text rendering. Needs SDF glyph rendering in shader or Iced absolute overlay.

### Real-time fetch progress bar
**What:** Parse scraper stdout line-by-line to show real progress %. Currently pulsing shimmer bar.
**Challenge:** Iced Task::perform doesn't support mid-task message updates. Needs subscription or stream.

## Open — Infrastructure

### docker-compose.yml for local PostgreSQL
**What:** `docker-compose.yml` to start PostgreSQL. 10-line file, `POSTGRES_PASSWORD=dev`, port 5432.
**Blocked by:** User wants to explore Docker first.

### v9.0 post-implementation frame time profiling
**What:** Measure actual frame times on astrology tab with all shader effects active. Target: <16ms per frame at 400x400.
**How:** Add temporary `Instant::now()` before/after shader draw call, log to console. Remove after verification.

## Completed (v6.0-v11.0)
- v11.0: "The Intelligence" — 90-day astro forecast timeline, backtest→history event correlation, Sun/Moon/Rising Big Three summary, smart calculator defaults (DCF growth from PEG, Greeks vol from historical data), zodiac sign band + planet symbol legend, pulsing loading bar shimmer, icon-only nav buttons
- v10.0: "The Signal" — RSS tone sentiment (keyword scoring from 25 feeds), Lagrange adaptive weighting (no more 50-default compression), richer agent verdicts (sector-aware, news-informed, 3 headline variants per verdict), fetch-this-ticker button (already existed, marked done)
- v9.3: "The Clarity" — aspect line contrast overhaul (width+alpha+glow), column widths fixed, tab labels bold (Fraunces Bold 16px), scrollbar gutter (20px right padding)
- v9.2: "The Cosmos" — galaxy chart background (purple gradient + nebula + star field), active tab gold glow + sparkle
- v9.1: "The Polish" — [P0] backtest crash fix (Clear Results button), [P0] broken icons (Phosphor X_LG), [P0] tooltip contrast (dark card + cream text), [P0] disable chart rotation
- v9.0: "The Performance" — 9 animation items: planet pulse, orbital trails, aspect shimmer, zodiac glow, dust mote cursor, candlestick draw-in, layered transitions, sparkle tuning, 60fps astrology tick
- v8.0: "The Observatory" — 3D natal chart GPU shader (procedural SDF, perspective tilt, 496-byte uniforms)
- v7.6: "The Consistency" — gold scrollbars, canvas sparkle, animated transit ring
- v7.3: "The Grimoire" — game-book UI, right-side tabs, Canvas ornaments, page transition fade
- v7.2: "The Motion" — Phosphor Icons, animation infrastructure, gauge sweep, toast fade, tab crossfade, responsive font scaling
- v7.1: Spatial polish — compact header, 1240px max-width, eyebrow labels, Inter font numerics
- v7.0: Renaissance book UI/UX overhaul (Parchment/Leather themes, 24-stage circadian)
- v6.0-6.2: Paper trading engine ($100K virtual capital, equity curve, NYSE holidays)
