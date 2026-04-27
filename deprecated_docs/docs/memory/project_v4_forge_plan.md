---
name: v4.0 "The Forge" Implementation Plan
description: Approved 3-phase roadmap (tech debt, UI/UX overhaul, new features) from v3.1.9 to v4.2.0, with Catppuccin theme, Lucide icons, candlestick charts
type: project
---

## v4.0 "The Forge" — Approved 2026-04-23

Three-phase plan synthesized from tech debt audit (20 issues), Iced ecosystem research (iced_fonts, plotters-iced, Catppuccin themes), and FinceptTerminal feature catalog.

**Why:** v3.1.9 completes all backlog items. Next priority is professional-grade UI and structural cleanup before adding more features.

**How to apply:** Work in phase order. Phase 1 (tech debt) has no visual changes and unblocks Phase 2. Phase 2 (UI/UX) sets the visual foundation for Phase 3 (features). Within each phase, items are largely independent.

### Key decisions:
- **Theme:** Catppuccin Mocha (warm dark, #1e1e2e base) with TokyoNight alt and Light mode
- **Icons:** Lucide via iced_fonts (1,400+ clean SVG-as-font icons)
- **Fonts:** Inter (UI) + JetBrains Mono (numbers) via include_bytes!
- **Charts:** plotters-iced 0.11 for candlestick/volume (replaces custom canvas)
- **Update refactor:** 921-line mod.rs -> 5 domain-specific files
- **Unwrap target:** 44 -> <10, with critical fixes in swisseph_bridge and interpretation

### New crates:
- `iced_fonts` 0.1 (Phase 2)
- `plotters-iced` 0.11 (Phase 3)
- `iced_toasts` (Phase 3)

### Total effort: ~3,400 new lines, ~1,800 changed, 12 new files
