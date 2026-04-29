# TODOS

## Open — v9.1 "The Polish" (P0 bugs + demo prep)

### [P0] Backtest crashes app, no cancel/exit
**What:** Running astro backtest freezes/crashes the dashboard. No way to cancel or exit the operation.
**Impact:** User must force-quit entire app. Blocks demo if triggered.
**Fix:** Add timeout guard or async cancellation to backtest computation. Show cancel button during long operations.

### [P0] Broken icons in Portfolio tab (showing "X")
**What:** Some icons render as placeholder "X" boxes in the Portfolio/Compare tab area.
**Impact:** Looks broken in demo. Likely missing Phosphor icon codepoints.
**Fix:** Audit icon usage in portfolio view, replace missing glyphs.

### [P0] Hover tooltip unreadable in Parchment mode
**What:** Candlestick chart hover tooltip text too small + low contrast on light Parchment background.
**Impact:** Users can't read OHLCV data on hover in light theme.
**Fix:** Increase tooltip font size, add background card with contrast-safe colors.

### [P0] Disable natal chart rotation
**What:** Chart slowly rotates, making it hard to read planetary positions and aspect lines.
**Impact:** User explicitly said "no rotation, hard to read."
**Fix:** Set `rot` speed to 0.0 in natal_wheel_3d.wgsl (currently `u.time * 0.015`). Optionally add Settings toggle.

### Natal chart too light in Parchment theme
**What:** 3D natal chart colors wash out on light Parchment background. Needs darker ring/planet colors.
**Fix:** Boost sign_color alpha and planet glow intensity when bg lightness > threshold, or add Parchment-specific color overrides.

### Aspect lines need more contrast
**What:** Aspect lines "need to be darker or thicker or brighter" per video review.
**Fix:** Increase asp_alpha values (currently 0.14-0.20 → 0.25-0.35) and/or increase ASPECT_W.

### Universe table column truncation
**What:** Column headers cut off ("Astr o", "Scor e", "Sh ort") because columns too narrow.
**Fix:** Widen columns or abbreviate headers properly ("Astro", "Score", "Short").

### Tab icon text bolder
**What:** Tab label lettering needs to be bolder for readability.
**Fix:** Increase font weight on tab labels.

### Scrollbar overlapping content
**What:** Gold scrollbar overlaps page content instead of having its own gutter space.
**Fix:** Add right padding/margin to page content container to reserve scrollbar space.

### Active tab glow effect (v9.1 video review)
**What:** Active tab icon should glow (emissive gold shimmer) instead of just bold weight. "Make it go glow, almost like shining." Matches gaming aesthetic.
**Fix:** Add gold glow/shadow effect to active tab icon in tab bar render. Could use Canvas overlay or text shadow if Iced supports it.

### Galaxy/constellation background for natal chart (v9.1 video review)
**What:** Natal chart area background should be dark purple/Milky Way with twinkling stars. Chart-local only, rest stays grimoire dark. "Should look like a Galaxy with little stars that flow in and out."
**Fix:** Extend star field in natal_wheel_3d.wgsl — currently stars only render outside R_OUTER. Add denser star field filling entire chart background with purple/blue tint + nebula gradient. Increase star density and add color variation.

### Galaxy bg needs more punch (v9.2 video review)
**What:** Current galaxy bg too subtle. User wants more visible purple, more stars, more "Galaxy in the background." 
**Fix:** Increase galaxy_edge purple (0.08→0.14 R, 0.03→0.05 G, 0.14→0.22 B). Boost nebula mix 0.15→0.30. Star intensity 0.18→0.30. Star density threshold 0.92→0.88. Should feel like looking through a telescope.

### Tab glow rework — bookmark style with gold border (v9.2 video review)
**What:** Current tab glow (gold bg tint) not dramatic enough. User wants "shining" effect, gold border, bookmark-tab shape. "This is not what I mean by glow." Suggested gold border and fire/energy aesthetic.
**Fix:** Replace 15% alpha gold bg with strong gold border (2-3px) around active tab. Make tab shape look like a physical bookmark tab. Consider Canvas-rendered glow halo behind icon for more dramatic shader-like effect.

### Clickable entity links — insider names, holders (v9.2 video review)
**What:** Insider trade names and institutional holder names should be clickable. "Click on this and it does a Google search or LinkedIn."
**Fix:** Wrap entity names in clickable text that calls `open::that(url)` to launch browser with Google/LinkedIn search for that person/institution.

### Chart layer visibility toggles (v9.1 video review)
**What:** Buttons to show/hide natal planets, transit planets, aspect lines, retrograde markers independently. Helps readability when chart is dense. "Select things to see which is which."
**Fix:** Add toggle state bools to Dashboard (show_natal, show_transit, show_aspects, show_retro). Pass as uniforms or control which sections render in view. Add toggle buttons near chart legend.

### Background texture — more earthy/Renaissance (v9.1 video review)
**What:** Background should feel more textured and earthy. "More of a Renaissance book feeling."
**Fix:** Enhance vignette shader noise grain — increase grain_strength, add subtle warm color variation to grain. Consider parchment-like color banding or subtle paper fiber pattern.

## Open — v10.0 "The Signal" (data quality)

### Data sparsity in Lagrange sub-scores
**What:** All 4 sub-scores computed, but sentiment and short-interest data sparse. Most tickers default to neutral 50, compressing scores into 45-73 range.
**Fix paths:** (a) RSS tone sentiment from existing 25 feeds, (b) alternative short-interest feed, (c) buy threshold at 65 as interim.
**Impact:** Paper engine trades with 65 threshold; raise back to 75 once data coverage improves.

### "Fetch this ticker" button
**What:** Single-ticker on-demand scraper fetch from dashboard UI.
**Blocked by:** Needs scraper-side work (currently batch-only pipeline).

### Template agent richer verdicts
**What:** LLM template agent says generic things like "hold" or "decent but not compelling." Needs more fleshed-out analysis per ticker.
**Fix:** Expand template responses with company-specific context, more varied verdicts, actionable reasoning.

## Open — v11.0 "The Intelligence" (deeper analysis)

### Astrological symbols + house numbers on natal chart
**What:** Add zodiac sign glyphs and house numbers to the 3D natal chart. Currently identified by color only.
**Challenge:** WGSL has no text rendering. Options: Iced overlay widget with positioned text, or SDF glyph rendering in shader.

### Sun/Moon/Rising summary per company
**What:** Show company's Sun sign, Moon sign, and rising sign as quick-reference icons near the chart.
**Fix:** Compute from natal positions, display as icon row above or beside the chart.

### Astro forecast timeline (v9.1 video review)
**What:** Project upcoming transits forward in time. Show when next favorable/challenging windows arrive for selected ticker. "Calculate horoscope readings throughout months and years and what's ahead."
**Fix:** Use Swiss Ephemeris to compute future transit positions (30/60/90 day lookahead). Display as timeline below natal chart showing upcoming aspects with dates and predicted effect.

### Astro backtest → company history correlation
**What:** Link astro backtest results to actual historical company events (earnings, product launches, scandals). Show what happened during favorable/unfavorable transit windows.
**Fix:** Cross-reference astro calendar dates with news/filing dates from existing data.

### Recommended defaults for calculators
**What:** DCF, Black-Scholes, and Backtest inputs should show suggested values based on current ticker data.
**Fix:** Auto-fill growth rate from earnings growth, volatility from historical prices, etc.

### Loading progress bar
**What:** Replace popup toast with animated progress bar for data fetching. "Zero to 100 kind of thing."
**Fix:** Add progress state to scraper fetch, render as horizontal bar in header area.

### Nav layout redesign
**What:** Search bar position, buttons as icons with hover labels, ticker under buttons, "Recent" placement.
**Scope:** Significant layout change to header/nav area. Needs design mockup first.

## Open — Infrastructure

### docker-compose.yml for local PostgreSQL
**What:** `docker-compose.yml` to start PostgreSQL. 10-line file, `POSTGRES_PASSWORD=dev`, port 5432.
**Blocked by:** User wants to explore Docker first.

### v9.0 post-implementation frame time profiling
**What:** Measure actual frame times on astrology tab with all shader effects active. Target: <16ms per frame at 400x400.
**How:** Add temporary `Instant::now()` before/after shader draw call, log to console. Remove after verification.

## Completed (v6.0-v9.0)
- v9.0: "The Performance" — 9 animation items: planet pulse, orbital trails, aspect shimmer, zodiac glow, dust mote cursor, candlestick draw-in, layered transitions, sparkle tuning, 60fps astrology tick
- v8.0: "The Observatory" — 3D natal chart GPU shader (procedural SDF, perspective tilt, 496-byte uniforms)
- v7.6: "The Consistency" — gold scrollbars, canvas sparkle, animated transit ring
- v7.3: "The Grimoire" — game-book UI, right-side tabs, Canvas ornaments, page transition fade
- v7.2: "The Motion" — Phosphor Icons, animation infrastructure, gauge sweep, toast fade, tab crossfade, responsive font scaling
- v7.1: Spatial polish — compact header, 1240px max-width, eyebrow labels, Inter font numerics
- v7.0: Renaissance book UI/UX overhaul (Parchment/Leather themes, 24-stage circadian)
- v6.0-6.2: Paper trading engine ($100K virtual capital, equity curve, NYSE holidays)
