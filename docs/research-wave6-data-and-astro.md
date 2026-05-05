# Wave 6 Research: Data Reliability + Astro Depth

**Date:** 2026-05-04
**Context:** Post-v11.3 expansion plan. Track A (financial data) + Track B (astro engine) paired so Concordance metric strengthens on both sides simultaneously.

---

## Track A — Financial Data Reliability

### Current Single-Points-of-Failure

| Domain | Primary | Fallback today | Failure mode |
|--------|---------|----------------|--------------|
| Prices (OHLCV) | Alpha Vantage (250/day) | None used | Rate-limit → ticker has no price |
| Fundamentals | FMP (250/day shared) | None | Daily budget exhaustion → no metrics |
| News | Finnhub | RSS scraping | Finnhub down → only RSS noise |
| Sentiment | AV NEWS_SENTIMENT | RSS tone | AV slow → stale sentiment |
| Macro | FRED | DBnomics | DBnomics inconsistent |

### Proposed Source Cascade

**Prices** (Wave 6.A1):
1. Alpha Vantage (existing, 250/day)
2. Tiingo (existing scaffolded, 800/hour free tier)
3. Finnhub (existing, 60/min free)
4. Yahoo Finance (new, no API key, scraping)
5. Stooq (new, no API key, EU/Asia coverage)

**Fundamentals** (Wave 6.A2):
1. FMP `/key-metrics-ttm` + `/ratios-ttm` (existing)
2. Finnhub `/stock/metric?metric=all` (new endpoint, existing creds)
3. Alpha Vantage `OVERVIEW` (existing creds, different field set)

### Schema Changes (Wave 6 migrations)

```sql
-- Migration 0026: data_source provenance on prices
ALTER TABLE prices ADD COLUMN data_source TEXT NOT NULL DEFAULT 'alpha_vantage';
CREATE INDEX idx_prices_source ON prices(data_source);

-- Migration 0027: data_source on fundamental_metrics
ALTER TABLE fundamental_metrics ADD COLUMN data_source TEXT NOT NULL DEFAULT 'fmp';

-- Migration 0028: earnings + analyst targets
CREATE TABLE earnings_calendar (
    ticker TEXT NOT NULL,
    report_date DATE NOT NULL,
    estimated_eps NUMERIC,
    fiscal_period TEXT,
    fetch_date DATE NOT NULL DEFAULT CURRENT_DATE,
    PRIMARY KEY (ticker, report_date)
);
CREATE TABLE analyst_targets (
    ticker TEXT PRIMARY KEY,
    fetch_date DATE NOT NULL DEFAULT CURRENT_DATE,
    low_target NUMERIC, median_target NUMERIC, high_target NUMERIC,
    n_analysts INTEGER
);

-- Migration 0029: data_freshness view
CREATE OR REPLACE VIEW data_freshness AS
SELECT
  cm.ticker,
  MAX(p.date)               AS last_price_date,
  MAX(f.fetch_date)          AS last_fund_date,
  COUNT(DISTINCT p.data_source) AS price_source_count,
  SUM(CASE WHEN n.published > NOW() - INTERVAL '7 days' THEN 1 ELSE 0 END) AS news_7d,
  SUM(CASE WHEN ss.fetch_date > CURRENT_DATE - 7 THEN 1 ELSE 0 END) AS sentiment_7d
FROM company_metadata cm
LEFT JOIN prices p USING (ticker)
LEFT JOIN fundamental_metrics f USING (ticker)
LEFT JOIN news_articles n USING (ticker)
LEFT JOIN sentiment_scores ss USING (ticker)
GROUP BY cm.ticker;
```

### New Source Modules

```rust
// src/scraper/sources/mod.rs
#[async_trait::async_trait]
pub trait PriceSource: Send + Sync {
    async fn fetch_prices(&self, ticker: &str) -> Result<Vec<PriceRow>>;
    fn name(&self) -> &'static str;
}

pub async fn fetch_with_fallback(
    sources: &[Box<dyn PriceSource>],
    ticker: &str,
) -> Result<(Vec<PriceRow>, &'static str)> {
    for src in sources {
        match src.fetch_prices(ticker).await {
            Ok(rows) if !rows.is_empty() => return Ok((rows, src.name())),
            Ok(_) => continue,
            Err(e) => eprintln!("[fallback] {} failed for {ticker}: {e}", src.name()),
        }
    }
    anyhow::bail!("All price sources exhausted for {ticker}")
}
```

### Caveats

- **Yahoo Finance scraping** is grey-area legal. Use only as last resort, document in source comment, respect their robots.txt for non-quote URLs.
- **Stooq** has no documented rate limit but reasonable client courtesy applies.
- **Provenance column** must NOT cause re-fetch when source changes — keep `(ticker, date)` PK, just track which source last won.

---

## Track B — Astrology Engine Depth

### Current Engine Snapshot

- **Bodies**: 13 (10 traditional + N Node + S Node + Chiron)
- **Aspect types**: 5 (conjunction, sextile, square, trine, opposition)
- **House system**: Whole Sign, NYSE location
- **Scoring**: fixed-point per aspect type, no orb weighting, no dignity, no patterns
- **Charts**: Natal + Daily Transits only (no harmonics, no progressions, no eclipses)

### Proposed Depth Additions

#### Aspect Patterns (Wave 6.B1)

```rust
// src/astrology/patterns.rs
pub enum AspectPattern {
    GrandTrine    { planets: [Planet; 3] },     // 3× 120° within 4° orb
    TSquare       { focal: Planet, ends: [Planet; 2] }, // opposition + 2 squares
    GrandCross    { planets: [Planet; 4] },     // 4 planets, 2 oppositions, 4 squares
    Yod           { apex: Planet, base: [Planet; 2] },  // 2 sextile, 2 quincunx
    MysticRect    { planets: [Planet; 4] },
    Stellium      { sign: Sign, planets: Vec<Planet> }, // 3+ in same sign
    Kite          { trine: [Planet; 3], opposite: Planet },
}

pub fn detect_patterns(positions: &[Position]) -> Vec<AspectPattern> { /* ... */ }
```

Migration 0030:
```sql
CREATE TABLE aspect_patterns (
    ticker TEXT NOT NULL,
    pattern_type TEXT NOT NULL,
    planets_involved TEXT[] NOT NULL,
    strength NUMERIC NOT NULL,
    fetch_date DATE NOT NULL DEFAULT CURRENT_DATE,
    PRIMARY KEY (ticker, pattern_type, fetch_date)
);
```

#### Aspect Strength Model (Wave 6.B2)

Replace fixed-point scoring with multiplicative:
```rust
pub fn aspect_strength(asp: &Aspect, positions: &[Position]) -> f32 {
    let base = match asp.kind {
        Conjunction => 10.0, Opposition => 8.0,
        Trine => 6.0, Square => 7.0, Sextile => 4.0,
    };
    let orb_factor = 1.0 - (asp.orb / asp.max_orb);          // tighter = stronger
    let applying = if asp.applying { 1.2 } else { 0.8 };
    let body_w = body_weight(asp.body_a) * body_weight(asp.body_b);
    let dignity = essential_dignity(asp.body_a, asp.sign_a)
                * essential_dignity(asp.body_b, asp.sign_b);
    let same_sign = if same_sign_aspect(asp) { 1.0 } else { 0.7 }; // out-of-sign penalty
    base * orb_factor * applying * body_w * dignity * same_sign
}
```

Body weights (Sun/Moon=1.5, Jup/Sat=1.3, outer=1.4, inner=1.0, nodes=0.8) reflect traditional astrologer practice — luminaries dominant, transpersonals slow but heavy, nodes karmic-modal.

Essential dignity table (`src/astrology/dignity.rs`):
| Body | Domicile | Detriment | Exalted | Fall |
|------|----------|-----------|---------|------|
| Sun | Leo | Aquarius | Aries | Libra |
| Moon | Cancer | Capricorn | Taurus | Scorpio |
| Mercury | Gemini, Virgo | Sagittarius, Pisces | Virgo | Pisces |
| Venus | Taurus, Libra | Aries, Scorpio | Pisces | Virgo |
| Mars | Aries, Scorpio | Taurus, Libra | Capricorn | Cancer |
| Jupiter | Sagittarius, Pisces | Gemini, Virgo | Cancer | Capricorn |
| Saturn | Capricorn, Aquarius | Cancer, Leo | Libra | Aries |

Multipliers: domicile 1.5, exalted 1.3, neutral 1.0, fall 0.7, detriment 0.5.

**Mutual reception**: when Body A is in Body B's domicile AND Body B is in Body A's domicile, both gain 1.2× extra (e.g., Mars in Libra + Venus in Aries simultaneously).

#### Fixed Stars (Wave 6.B3)

Swiss Ephemeris built-in: `swe_fixstar(name, jdn, flags, x_position)` returns precessed longitude. Names follow IAU/Bayer conventions.

| Star | Mag | Modern position | Archetype | Use |
|------|-----|-----------------|-----------|-----|
| Regulus | 1.4 | 29° Leo | Royal kingship | Finance leadership |
| Algol | 2.1 | 26° Taurus | Medusa, sudden loss | Volatility warning |
| Spica | 1.0 | 24° Libra | Sheaf of wheat, abundance | Wealth signal |
| Antares | 1.0 | 9° Sagittarius | Heart of Scorpion | Military/finance leadership |
| Sirius | -1.5 | 14° Cancer | Dog star, fame | Media-attention spike |
| Vega | 0.0 | 15° Capricorn | Lyre, artistry | Creative/IP success |
| Aldebaran | 0.9 | 10° Gemini | Bull's eye | Honors, military |
| Fomalhaut | 1.2 | 4° Pisces | Fish mouth, dreams | Transformation moments |

Conjunction within 1° orb to a transiting planet → activation flag stored alongside aspects.

#### Arabic Parts (Wave 6.B3)

Pure derivation from natal positions. Day/night formula split based on whether Sun is above/below horizon at birth.

```rust
pub fn part_of_fortune(asc: f64, sun: f64, moon: f64, is_day: bool) -> f64 {
    if is_day { (asc + moon - sun).rem_euclid(360.0) }
    else      { (asc + sun - moon).rem_euclid(360.0) }
}
pub fn part_of_spirit(asc: f64, sun: f64, moon: f64, is_day: bool) -> f64 {
    if is_day { (asc + sun - moon).rem_euclid(360.0) }
    else      { (asc + moon - sun).rem_euclid(360.0) }
}
pub fn part_of_commerce(asc: f64, sun: f64, mercury: f64) -> f64 {
    (asc + mercury - sun).rem_euclid(360.0)
}
```

Transit aspects to these parts (especially Fortune) historically used for "money timing." Add to forecast computation.

#### Eclipse Cycles (Wave 6.B4)

Swiss Ephemeris functions:
- `swe_sol_eclipse_when_loc(jdn, geopos, ifltype)` — next solar eclipse from given JDN
- `swe_lun_eclipse_when(jdn, ifltype)` — next lunar eclipse
- `swe_sol_eclipse_where(jdn, ifl)` — central path coordinates

Migration 0031:
```sql
CREATE TABLE eclipses (
    eclipse_date TIMESTAMP NOT NULL,
    eclipse_type TEXT NOT NULL CHECK (eclipse_type IN ('solar_total','solar_partial','solar_annular','lunar_total','lunar_partial','lunar_penumbral')),
    longitude NUMERIC NOT NULL,
    magnitude NUMERIC,
    saros_series INTEGER,
    PRIMARY KEY (eclipse_date)
);
```

Algorithm: for each ticker, scan upcoming 12 months of eclipses, flag any natal planet within 6° of eclipse longitude → activation event.

**Saros series**: each eclipse belongs to a numbered family that repeats every 18y 11d 8h. Storing `saros_series` lets us cross-reference historical eclipses in same family ("AAPL's natal Sun was hit by Saros 142 in 2008 — next return 2026").

---

## Build Order

Recommended ship sequence (4 sub-waves, ship + verify each):

1. **6.0 "The Reliability"** = 6.A1 + 6.B1 — multi-source prices + aspect patterns. Highest user-visible.
2. **6.1 "The Precision"** = 6.A2 + 6.B2 — fundamentals fallback + dignity-weighted aspects.
3. **6.2 "The Depth"** = 6.A3 + 6.B3 — earnings calendar + fixed stars/Arabic Parts.
4. **6.3 "The Trust"** = 6.A4 + 6.B4 — data freshness UI + eclipse cycles.

After each sub-wave: `cargo build`, run scraper for one ticker, verify new tables, eyeball UI surface, run backtest comparison.

**Total estimate**: ~10 days. Stops where Concordance metric stops improving (diminishing returns).

---

## Resolved Questions

1. **Paid API keys?** — **Decision 2026-05-04: free tier only.** Polygon/EODHD/Quandl/Marketstack moved to TODOS.md "API Keys Backlog." Wave 6.A1 cascade is AV → Tiingo → Finnhub (existing keys) → Yahoo → Stooq (no key, scraping). Wave 6.A2 cascade is FMP → Finnhub `metric/all` → AV `OVERVIEW`.

## Open Questions

1. Backtest harness: does it support comparing astro_score variants (v11.3 baseline vs Wave 6.B2 dignity-weighted)? If not, build that first.
2. Vedic-style harmonics (H4/H5/H7/H9) deferred from Wave 6 — too speculative for first cut. Revisit after B1+B2 produce measurably different scores.
3. Sidereal vs Tropical concordance also deferred — depends on B2 strength model first.
4. Yahoo Finance scraping fragility — they change response shape periodically. Mitigation: keep AV/Tiingo/Finnhub primary, scraping last resort. Add explicit "Yahoo response format changed" detection + alert log when parse fails.
