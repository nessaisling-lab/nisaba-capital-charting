# Wave 9 — "The Compounding" — Implementation Roadmap

**Theme:** Deepen the astrology engine from "what aspects fire today" → "where in the corporate lifecycle is today happening." Time-lord systems, cycle returns, and narrative depth — the layers serious astrologers (and Hellenistic financial astrologers in particular) use *on top of* the v6 aspect engine.

**Estimated scope:** ~10-14 days across 4 paired sub-waves (Track A: time-lord/cycle systems · Track B: narrative + precision polish) + infrastructure.

**Author:** Aisling Leiva
**Started:** 2026-05-06 (planning)
**Status:** SCOPED, not yet implemented.

---

## Why this wave matters

The current engine answers: *"What aspects are firing today?"*

Serious astrologers ask three more questions the engine can't yet answer:

1. **Where in the company's lifecycle is this transit happening?** (returns + progressions)
2. **Which planet rules this *specific year* of the company's life?** (profections)
3. **What does this exact degree *mean*?** (Sabian + decans)

Wave 9 answers those. Each addition compounds with the existing engine — a Saturn transit on a Jupiter-return year hitting a critical degree with a Sabian symbol of "An ancient bridge over a beautiful stream" is a *much* richer reading than just "Saturn square Sun, orb 1.2°."

## Pre-existing inventory (do not rebuild)

Validated by `Explore` agent scan 2026-05-06. The following are PRODUCTION:

- Natal chart (13 bodies via Swiss Eph + Moshier fallback)
- Daily transits (refreshed daily)
- 9 aspect types + v6.B2 strength model (orb tightness, applying/separating, body weight, dignity, mutual reception)
- 7 aspect patterns (Grand Trine, T-Square, Grand Cross, Yod, Stellium, Mystic Rectangle, Kite)
- 8 fixed stars (Regulus, Spica, Antares, Aldebaran, Sirius, Vega, Fomalhaut, Algol)
- 4 Arabic Parts (Fortune, Spirit, Commerce, Substance)
- Eclipse activations (NASA Five-Millennium Catalog 2025-2028)
- Horoscope template engine + 90-day forecast
- Whole-sign houses (Ascendant + MC via Swiss Eph)
- Backtest engine (per-day, 30-day minimum, signal accuracy %)

## Pre-existing infrastructure gaps

- Swiss Eph: declination not used (only ecliptic longitude)
- House systems: Whole Sign only
- Asteroids: only Chiron loaded (no `.se1` files for Ceres/Pallas/Juno/Vesta)

---

## Wave 9 sub-waves (paired tracks like Wave 6)

### Track A — Time-lord systems (cycle precision)

#### 9.A1 — Solar Return charts
**Files:** `src/astrology/solar_return.rs` (new), migration `0047_solar_returns.sql`

**Concept:** Cast a chart for the exact moment the transiting Sun returns to its natal longitude (within ±0.5°). Annual in scope; valid for 1 year birthday-to-birthday. Astrologers read this as "the year's outlook."

**Implementation:**
1. `compute_solar_return(natal: &NatalChart, target_year: i32) -> SolarReturnChart` — Swiss Eph search to find exact Sun return moment within target year
2. `SolarReturnChart { return_date, planets: Vec<NatalPosition>, ascendant, mc, aspects_to_natal }`
3. Aspects between SR planets and natal planets ("cross-chart aspects")
4. New table `solar_returns` keyed by `(ticker, return_year)`
5. UI: New "Solar Return" sub-section in Astrology tab — current year's SR chart + key aspects to natal

**Effort:** 1.5 days · **Risk:** Low (search-and-snapshot pattern; Swiss Eph can already compute longitudes for arbitrary moments)

#### 9.A2 — Profections (Hellenistic annual time-lord)
**Files:** `src/astrology/profections.rs` (new), migration `0048_profections.sql`

**Concept:** Profections rotate one *house* per year of life starting from the 1st (Ascendant). Each year's "profected house" determines the year's "time-lord" — the planet ruling that house's sign. Hellenistic + Persian astrology hinges on this.

For a stock IPO'd in 2010 (ticker age 16 in 2026):
- Year 0: 1st house, Lord = ASC sign ruler
- Year 16: Profect to 5th house (16 mod 12 = 4, +1 = 5), Lord = 5th house sign ruler

**Implementation:**
1. `compute_profection(natal: &NatalChart, target_date: NaiveDate) -> Profection` — annual + monthly + daily nesting (12-year cycles for year, 1-year for month, 1-month for day)
2. `Profection { age, profected_house, sign, lord_planet, sub_lord, sub_sub_lord, theme }`
3. Lord-planet emphasis weighting: any aspect involving the year's time-lord gets a 1.5× strength multiplier
4. New table `profections_history` (ticker, target_date, profected_house, lord_planet)
5. UI: "Year of [Lord]" badge in Astrology tab header — e.g., "AAPL — Year of Saturn (5th House)"

**Effort:** 2 days · **Risk:** Low-medium (math is simple; integrating lord-emphasis into existing aspect scoring needs care)

#### 9.A3 — Planetary returns (Saturn, Jupiter, Mars)
**Files:** `src/astrology/returns.rs` (new), migration `0049_planetary_returns.sql`

**Concept:** Find every exact moment in a 50-year window when each major planet returns to its natal longitude. Saturn return = 29.5y, Jupiter return = 12y, Mars return = 2y. These are the "milestone" years.

**Implementation:**
1. `find_returns(natal: &NatalChart, planet: Planet, window_years: i32) -> Vec<ReturnEvent>`
2. `ReturnEvent { planet, return_date, exact_orb, days_until, return_chart_aspect_summary }`
3. For Saturn return (the big one): cast a `ReturnChart` like Solar Return — full chart at the moment of exact Saturn return
4. New table `planetary_returns_calendar` (ticker, planet, return_date, return_number)
5. UI: "Upcoming returns" timeline in Astrology tab — "Saturn return in 18 months" / "Jupiter return last week"

**Effort:** 1.5 days · **Risk:** Low (binary search through Swiss Eph)

#### 9.A4 — Secondary progressions
**Files:** `src/astrology/progressions.rs` (new), migration `0050_progressions.sql`

**Concept:** "1 day = 1 year" — to get the chart for year N of a person/company's life, advance the natal chart by N days. Progressed Moon cycles every ~28 years (Saturn-like cadence). Progressed Sun moves ~1° per year — so a progressed Sun ingressing into a new sign every ~30 years is a major signal.

**Implementation:**
1. `compute_progressed_chart(natal: &NatalChart, target_date: NaiveDate) -> ProgressedChart`
2. Detect progressed-to-natal aspects (slow moving — most stay in orb for months/years)
3. Detect progressed Moon sign changes + lunation cycle phase
4. Detect progressed Sun sign ingress (rare: every ~30 years; major)
5. New table `progressions_snapshots` (ticker, snapshot_date, progressed_planets JSONB)
6. UI: "Progressed Sun in [Sign]" badge + "Progressed Moon entering [Sign] in N days"

**Effort:** 2 days · **Risk:** Medium (many edge cases; progressed angles need house-system care)

---

### Track B — Narrative + precision

#### 9.B1 — Decans (3 per sign, 36 total)
**Files:** `src/astrology/decans.rs` (new)

**Concept:** Each 30° sign divides into three 10° "decans," each with its own planetary ruler. Aries: 0-10° Mars-Mars, 10-20° Mars-Sun, 20-30° Mars-Venus (Egyptian / Triplicity systems). Decans give sub-sign nuance — Sun at 5° Aries vs 25° Aries reads differently.

**Implementation:**
1. `decan_for_longitude(lon: f64) -> Decan` — returns `Decan { sign, decan_index, ruler, sub_ruler, theme }`
2. Decan tables (constants — no DB needed): Egyptian system (most common in Hellenistic) + Chaldean fallback
3. Wire into existing `NatalPosition` rendering: append decan info on hover/expand
4. Decan-conjunction-with-natal-planet boost in aspect scoring (1.1× when transit hits same decan as natal)

**Effort:** 1 day · **Risk:** Low (lookup tables only)

#### 9.B2 — Sabian symbols (360 degree-meanings)
**Files:** `src/astrology/sabian.rs` (new), `src/astrology/sabian_data.rs` (data file)

**Concept:** Marc Edmund Jones channeled 360 symbolic images, one per degree of the zodiac. "Sun at 15° Cancer = A group of people who have overeaten and enjoyed it." Used for character/timing readings.

**Implementation:**
1. `sabian_for_longitude(lon: f64) -> SabianSymbol` — returns `SabianSymbol { degree, sign, image, keynote }`
2. Static table of 360 symbols (public domain, derived from Jones 1925 channeling)
3. Wire into UI: hover any planet → tooltip shows Sabian symbol
4. Lagrange score does NOT incorporate Sabian (purely narrative — same reason horoscope isn't quantified)

**Effort:** 0.5 days code + 0.5 days data assembly = 1 day · **Risk:** Low (pure data + lookup)

#### 9.B3 — Aspect strength visual gradient
**Files:** `src/dashboard/shaders/natal_wheel_3d.wgsl`, `src/dashboard/charts.rs`

**Concept:** Currently aspect lines are binary: green (favorable) or red (unfavorable). Replace with continuous gradient driven by the v6.B2 strength score — bright/saturated for tight applying aspects, dim/desaturated for wide separating.

**Implementation:**
1. `AspectStrengthColor` helper in shader — input: strength score 0-100, output: RGBA
2. Color stops:
   - 0-25: dim grey (informational only)
   - 25-50: muted favorability hue
   - 50-75: full saturation favorability hue
   - 75-100: glow intensity (bloom)
3. Aspect line width scales with strength too (1px → 3px)
4. Update wheel WGSL aspect-line painter

**Effort:** 1 day · **Risk:** Low-medium (shader math; visual tuning)

#### 9.B4 — Critical degrees + out-of-bounds detection
**Files:** `src/astrology/critical.rs` (new)

**Concept:**
- **Critical degrees** — sensitive points per sign group: 0°, 13°, 26° of cardinal (Aries/Cancer/Libra/Capricorn); 8-9° + 21-22° of fixed (Taurus/Leo/Scorpio/Aquarius); 4° + 17° of mutable (Gemini/Virgo/Sagittarius/Pisces). Planets here = action degrees, often coincide with major events.
- **Out-of-bounds (OOB)** — planets beyond ±23°27' declination (the Sun's tropic limits) act unpredictably. Outer planets rarely OOB; inner planets (Mercury/Venus/Mars/Moon) often. OOB Moon = wild emotional reactivity. (Requires declination support — see infrastructure.)

**Implementation:**
1. `is_critical_degree(lon: f64) -> Option<CriticalDegreeKind>` — pure math
2. `is_out_of_bounds(declination: f64) -> bool` — `declination.abs() > 23.4367`
3. Annotate planet displays with critical-degree + OOB badges
4. +0.3× strength bonus for transits at critical degrees

**Effort:** 0.5 days for critical + 1 day for OOB (depends on declination infra) = 1.5 days
**Risk:** Critical degrees: trivial. OOB: medium (needs Swiss Eph declination wrapper).

---

### Infrastructure (one-time foundation)

#### 9.I1 — Declination support in Swiss Eph bridge
**Files:** `src/astrology/swisseph_bridge.rs`

**Concept:** Currently the bridge returns longitude + speed. Need to also return declination (latitude in equatorial frame) for parallels + OOB detection. Swiss Eph already computes this — just expose it.

**Implementation:**
1. Add `pub declination: f64` to existing planet snapshot struct
2. Use Swiss Eph `swe_calc_ut` flag `SEFLG_EQUATORIAL` to get RA + declination
3. Update `snapshot_all_precise` signature

**Effort:** 0.5 days · **Risk:** Low

#### 9.I2 — Backtest extension to 5+ years
**Files:** `src/dashboard/backtest.rs`, possibly `src/dashboard/db/lagrange.rs`

**Concept:** Current backtest is per-day with no fixed window. Extend it to optionally aggregate 1-year, 3-year, 5-year windows AND scan for cycle-aligned periods (Saturn return zone, Jupiter conjunction zone). User can backtest "How does AAPL perform in its profected Saturn-year?"

**Implementation:**
1. Extend `BacktestConfig` with `time_window: TimeWindow::Custom(start, end) | OneYear | FiveYear | LifetimeMatching(planet, return_kind)`
2. Wire into UI as Settings → Backtest section
3. Cycle-aligned mode: pre-filter days to those matching the lifecycle event criterion, run backtest only on those days

**Effort:** 1.5 days · **Risk:** Medium (`LifetimeMatching` needs profections + returns wired first)

---

## Sequencing

Like Wave 6, ship paired (one A + one B) for compounding visibility.

| Wave | Pair | Theme | Days |
|------|------|-------|------|
| 9.0 | 9.I1 + 9.B3 | "The Foundation" — declination + visual gradient (sets infra + visible polish) | 1.5 |
| 9.1 | 9.A1 + 9.B1 | "The Year" — Solar Return + Decans | 2.5 |
| 9.2 | 9.A2 + 9.B2 | "The Cycle" — Planetary Returns + Sabian Symbols | 2.5 |
| 9.3 | 9.A3 + 9.B4 | "The Lord" — Profections + Critical Degrees + OOB | 3 |
| 9.4 | 9.A4 + 9.I2 | "The Maturation" — Progressions + Backtest extension | 3.5 |

**Total estimate:** 13 days. Realistic range: 10-15 days given typical task underestimation.

## Verification per sub-wave

Each sub-wave must:
1. Pass `cargo check` clean
2. Have at least 1 unit test for the math/lookup function
3. Render at least 1 user-visible UI change
4. Update CHANGELOG + TODOS
5. Validate against a known reference chart (use AAPL natal: 1980-12-12, 09:30 EST = Saturn ~16° Virgo, Sun ~21° Sagittarius)

## Risk register

| Risk | Mitigation |
|------|------------|
| Swiss Eph asteroid `.se1` files needed for Ceres/Pallas/Juno/Vesta | Defer — not in this wave. Add in v13 if user requests. |
| Declination wrapper introduces FFI bug | Add unit test against known JPL ephemeris values for one date |
| Profections lord-emphasis breaks existing aspect scoring tests | Gate behind feature flag for first 2 days; keep both scoring paths until validated |
| Sabian symbols data set too large for source file | Split into `sabian_data.rs` (360 entries × ~80 chars = ~30KB — well within limits) |
| User finds the new layers confusing → readings get muddier | Default new layers OFF in Settings; user opts in. Solar Return on by default since universally understood. |

## Out of scope (deferred)

- Synastry / composite (chart-to-chart) — too niche for financial dashboards
- Vedic dasha periods — sectarian system clash with Hellenistic
- Lunar mansions — niche
- Asteroid catalog beyond Chiron — needs `.se1` packaging story
- Placidus/Koch alternative house systems — Whole Sign serves
- Vertex / East Point — secondary angles, low ROI

## Documentation deliverables (per sub-wave)

- CHANGELOG entry with theme name + key concepts
- DESIGN.md "Patterns" section addition for any new abstraction
- TODOS.md migration of items to "Closed" with date
- `docs/wave9-N-N.md` design note for any non-obvious choice (e.g., why Egyptian decan system over Chaldean)

## Validation reference: AAPL

Use AAPL throughout. IPO 1980-12-12 09:30 EST (14:30 UTC) — Saturn ~16° Virgo, Sun 21° Sagittarius, ASC ~14° Capricorn.

For 2026-05-06:
- Profected age 45 → 9th house (45 mod 12 = 9), lord = sign ruler of natal 9th
- Last Saturn return: 2010 (Saturn return ~30y after IPO)
- Next Jupiter return: 2028 (Jupiter cycles every 12y from natal Jupiter Sagittarius)
- Solar return 2025: cast for 2025-12-12 — should show Sun at exact 21° Sagittarius

Reference chart values must round-trip through code without drift > 0.1°.

---

## When this plan is done

Engine becomes **production-tier financial astrology**. A user reads a verdict like:

> *"AAPL 2026-05-06: It's the year of Mars (5th-house profection). Saturn (transit) opposes natal Sun at 22° Sagittarius — exact within 0.4° — at 22° Gemini, a critical degree. Sabian symbol for the natal Sun: 'A flag-bearer in a battle.' Mars is out-of-bounds at +24° declination. Solar return shows Mars on the ASC. Saturn return was 2010 (the iPhone 4 era). Verdict: structural test, but the time-lord and the SR support engagement."*

That's the bar. Today's engine could give us "Saturn opposite Sun, orb 0.4°." Wave 9 gets us the rest.
