# Product

## Register

product

## Users

Three tiers, each a different person (or the same person at different stages):

- **Free (The Curious):** Astrology enthusiasts and finance-curious newcomers. They open the app because horoscopes are fun, stay because the correlations are real. Context: scrolling on their phone, sharing a screenshot of "Astro score predicted Apple's iPhone announcement." Job: learn, be entertained, build intuition about markets through a lens they already trust.

- **Pro (The Player):** Engaged users who want skin in the game without real risk. They've graduated from reading to doing. Context: checking paper trades at lunch, placing astro-informed bets, competing with friends. Job: practice trading and investing decisions using astro signals as an edge, with gamified feedback (paper P&L, streaks, leaderboards).

- **Master (The Professional):** Serious traders and investors who want every tool. They treat astrology the way JP Morgan did: as one more signal in the stack. Context: multi-monitor setup, scanning sectors, running backtests, managing real capital. Job: full-power financial analysis with astro signals integrated as first-class data alongside fundamentals, technicals, and sentiment.

The product graduates users up this ladder. Astrology is the gateway; financial literacy is the destination.

## Product Purpose

A financial intelligence platform that treats astrology as serious, quantifiable market signal alongside traditional analysis. Combines natal charts, planetary transits, and aspect geometry with real market data, LLM-powered verdicts, sentiment analysis, and paper/live trading.

The thesis: astrology and finance have always been intertwined (JP Morgan, Coco Chanel, W.D. Gann). This product makes that connection explicit, data-driven, and accessible to everyone from the curious beginner to the professional trader.

Success looks like: a user who came for their horoscope stays to learn DCF analysis. A professional trader who scoffed at astrology starts checking transit aspects before earnings season.

## Brand Personality

**Arcane. Authoritative. Alive.**

- **Arcane:** The knowledge is old and deep. Renaissance grimoire energy: hand-lettered chapter headings, leather-bound spines, gold leaf. Not whimsical, not novelty. This is serious esoteric tradition.
- **Authoritative:** 1970s/1980s Wall Street black leather ledger energy. The numbers are real, the stakes are real, the tools are professional-grade. No toy dashboards.
- **Alive:** The interface breathes. Planets pulse, aspect lines shimmer, dust motes drift. The cosmos is in motion and so is your portfolio. Wonder without frivolity.

Voice: confident, knowledgeable, slightly mysterious. Speaks like a seasoned trader who reads natal charts on weekends. Never condescending to either discipline.

## Anti-references

- **Casual horoscope apps playing it safe:** Co-Star's minimalist cards, The Pattern's pastel gradients. These treat astrology as entertainment. We treat it as intelligence.
- **Gamified finance apps playing it cute:** Robinhood's confetti, Webull's candy colors. Finance is not a game (even when we gamify parts of it, the stakes feel real).
- **Bloomberg being ugly on purpose:** Utilitarian is fine at Master tier, but never at the expense of craft. Even the densest screen should have atmosphere and intention.
- **Generic SaaS dashboards:** Identical card grids, hero-metric templates, teal accents. This is not a B2B analytics tool.
- **"AI-generated" aesthetic:** Gradient text, glassmorphism, identical card layouts. If it looks like a template, it fails.

What we take from each instead:
- From Co-Star/Pattern: the emotional hook, the shareability, the daily ritual
- From Robinhood/Webull: the accessibility, the onboarding, the mobile-first thinking
- From Bloomberg: the density, the keyboard shortcuts, the "everything on one screen" power
- From none of them: the aesthetic. That's ours alone.

## Design Principles

1. **The grimoire and the ledger.** Every screen lives at the intersection of Renaissance mysticism and Wall Street authority. Neither aesthetic dominates; they reinforce each other. A natal chart should feel as credible as a P&L statement. A financial table should feel as atmospheric as an illuminated manuscript.

2. **Serious about both.** Never condescend to astrology ("just for fun") or gatekeep finance ("too complex for you"). Both disciplines have depth; the product respects that depth at every tier.

3. **Graduate, don't gate.** Free users see the same quality as Master users, just less of it. The experience scales up in capability, not in craft. A free user's first screen should make them want to unlock more.

4. **Alive, not animated.** Motion serves atmosphere and comprehension, not decoration. Planets breathe because the cosmos moves. Candlesticks draw in because time flows. Nothing moves just to move.

5. **Power through wonder.** The densest, most utilitarian screen in the app should still evoke a sense of wonder. Bloomberg meets Borges. If a screen feels like "just a dashboard," it needs more soul.

## Accessibility & Inclusion

- WCAG AA minimum across all tiers. Master tier dense layouts need particular attention to contrast and type sizing.
- Touch targets sized for mobile from the start (48px minimum), even though the current build is desktop-native Rust/Iced.
- Reduced motion support: all atmospheric animations (dust motes, planet pulse, shimmer) must have a still-frame fallback that preserves the aesthetic.
- Color-blind safe: astro aspects, chart elements, and financial signals (green/red) need non-color differentiators (shape, pattern, label).
- Cross-platform target: Rust native now, eventually Android/iOS/Web. Design decisions should not assume mouse-only interaction.

## Monetization Model (Draft)

- **Free:** Astro readings, historical correlations, educational content, basic charts
- **Pro:** Paper trading, gamified features, deeper analysis tools. Microtransaction or subscription (TBD).
- **Master:** Real money trading integration, full Bloomberg-level tooling, all power-user features. Subscription.

## Technical Context

- **Current stack:** Rust, Iced 0.14 (GPU-accelerated), SQLx, PostgreSQL, WGSL shaders, Swiss Ephemeris (sub-arcsecond), 20-module scraper
- **Current state:** v11.4 desktop app (Wave 6 complete), scraper + dashboard binaries
- **Data sources active:** Alpha Vantage, FMP, Finnhub, FRED/DBnomics, EDGAR, Polymarket, GDELT, RSS, Tiingo, Wikidata
- **Wave 6 shipped (v11.4):** multi-source fallback chains for prices (AV→Tiingo→Finnhub→Yahoo→Stooq) + fundamentals (FMP→Finnhub→AV), 7-pattern aspect recognition (Grand Trine, T-Square, Yod, etc.), aspect strength upgrade (body weight, mutual reception, out-of-sign), 8 fixed stars + 4 Arabic Parts, 17 eclipses with Saros tracking, data freshness UI badges. Both Lagrange inputs significantly richer.
- **Wave 7 planned:** OpenBB Platform integration as additive data tier (NOT replacing existing scrapers). 350+ datasets via Python service at localhost:6900. Optional Workspace cloud UI for research. Cross-check existing FRED data for discrepancy detection.
- **Deployment:** Not yet deployed. Cross-platform strategy TBD (Rust → mobile options include Tauri, Dioxus, or native bridges).
- **Class context:** Pursuit NYC Week 4 Fellowship project. Teacher previously critical of UI/UX (pre-v7.0 builds). v11.2 self-reviewed on 2026-04-30 (15-min video review, 22 feedback items captured). Key findings: layout needs tightening, natal chart needs higher visual quality (finer aspect lines, planet symbols, interactivity), Council agents too template-generic, sector filter UX poor. Approved: tab bookmark styling, icon-only nav, Universe legibility. Iced 0.13→0.14 framework upgrade completed same day (19 breaking API changes across 13 files). All 22 items shipped 2026-05-04 as v11.3 "The Refinement" across 5 waves. Wave 6 next — paired data reliability (multi-source fallback) + astrology engine depth (patterns, fixed stars, eclipses, dignity-weighted aspects).
