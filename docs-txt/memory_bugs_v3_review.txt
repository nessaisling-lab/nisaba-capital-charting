---
name: v3.0.6 Post-Release Bug Review (RESOLVED)
description: Bugs found and fixed during v3.0 output review — all 3 bugs fixed, all 16 warnings resolved
type: project
---

## v3.0.6 Post-Release Bug Review (2026-04-22) — ALL RESOLVED

Three bugs + 16 warnings identified and fixed after reviewing scraper and dashboard output.

### Bug 1: Score Polarization (CRITICAL) — FIXED

**Fix:** `natal.rs:204-208` — normalize delta_sum by sqrt(aspect_count), increased SIGMOID_K from 0.04 to 0.10. Produces bell-shaped distribution centered at 50.

### Bug 2: Theme-Score Mismatch (MODERATE) — FIXED

**Fix:** `interpretation.rs:773-778` — expanded reconciliation thresholds from 65+/0-35 to 56+/0-44. Neutral zone now matches score labels (45-55).

### Bug 3: AstroRanking Not Consumed (MINOR) — FIXED

**Fix:** `main.rs:278` renamed `_ranking` to `ranking`. Added `fetch_priority_prices()` to `prices.rs`. Priority tickers get price data before watchlist.

### 16 Compiler Warnings — ALL RESOLVED

- 5 true dead code removed (view.rs, tabs.rs, theme.rs)
- 11 incomplete feature fields suppressed with #[allow(dead_code)]
- Both binaries build with zero warnings
