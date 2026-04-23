use chrono::{Datelike, NaiveDate, Utc};
use pursuit_week4_automation::astrology::ephemeris::date_to_jdn;
use pursuit_week4_automation::astrology::interpretation::{generate_horoscope, horoscope_to_json};
use pursuit_week4_automation::astrology::natal::{aspects_to_json, compute_transit_score, NatalChart};
use pursuit_week4_automation::astrology::swisseph_bridge::{
    snapshot_all_precise, longitude_speed, compute_houses_nyse,
};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Natal chart seeder — seeds any ticker missing from natal_positions
// ---------------------------------------------------------------------------

pub async fn seed_natal_charts(pool: Arc<sqlx::PgPool>) {
    let rows: Vec<(String, NaiveDate)> = match sqlx::query_as(
        "SELECT cm.ticker, cm.ipo_date \
         FROM company_metadata cm \
         WHERE cm.ipo_date IS NOT NULL \
           AND NOT EXISTS ( \
               SELECT 1 FROM natal_positions np WHERE np.ticker = cm.ticker \
           ) \
         ORDER BY cm.ticker",
    )
    .fetch_all(pool.as_ref())
    .await
    {
        Ok(r) => r,
        Err(e) => { eprintln!("Failed to load company_metadata: {e}"); return; }
    };

    if rows.is_empty() {
        println!("Natal charts already seeded for all companies, skipping.");
        return;
    }

    println!("Seeding natal charts for {} missing companies...", rows.len());

    for (ticker, ipo_date) in &rows {
        let chart = NatalChart::compute(ticker, *ipo_date);
        for pos in &chart.positions {
            let result = sqlx::query(
                "INSERT INTO natal_positions (ticker, planet, longitude, sign, degree, retrograde) \
                 VALUES ($1, $2, $3, $4, $5, $6) \
                 ON CONFLICT (ticker, planet) DO UPDATE \
                 SET longitude = EXCLUDED.longitude, sign = EXCLUDED.sign, \
                     degree = EXCLUDED.degree, retrograde = EXCLUDED.retrograde",
            )
            .bind(ticker)
            .bind(pos.planet.name())
            .bind(pos.longitude)
            .bind(pos.sign)
            .bind(pos.degree)
            .bind(pos.retrograde)
            .execute(pool.as_ref())
            .await;

            if let Err(e) = result {
                eprintln!("Failed to insert natal position for {ticker}/{}: {e}", pos.planet.name());
            }
        }

        // Store natal angles (Ascendant + MC) using NYSE location
        let jdn = date_to_jdn(ipo_date.year(), ipo_date.month(), ipo_date.day(), 14.5);
        if let Ok(houses) = compute_houses_nyse(jdn) {
            let _ = sqlx::query(
                "INSERT INTO natal_angles (ticker, ascendant, mc) \
                 VALUES ($1, $2, $3) \
                 ON CONFLICT (ticker) DO UPDATE \
                 SET ascendant = EXCLUDED.ascendant, mc = EXCLUDED.mc",
            )
            .bind(ticker)
            .bind(houses.ascendant)
            .bind(houses.mc)
            .execute(pool.as_ref())
            .await;
        }

        println!("  Natal chart seeded: {ticker} (IPO: {ipo_date}, {} bodies)", chart.positions.len());
    }

    println!("Natal chart seeding complete.");
}

// ---------------------------------------------------------------------------
// Daily planetary transit positions
// ---------------------------------------------------------------------------

pub async fn compute_daily_transits(pool: Arc<sqlx::PgPool>) {
    let today = Utc::now().date_naive();

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM daily_transits WHERE fetch_date = $1)",
    )
    .bind(today)
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(false);

    if exists {
        println!("Daily transits already computed for {today}, skipping.");
        return;
    }

    let jdn       = date_to_jdn(today.year(), today.month(), today.day(), 14.5);
    let snapshots = snapshot_all_precise(jdn);

    println!("Storing {} planetary positions for {today} (Swiss Eph)...", snapshots.len());

    for snap in &snapshots {
        // Get speed for applying/separating detection
        let speed = longitude_speed(snap.planet, jdn);

        let result = sqlx::query(
            "INSERT INTO daily_transits (fetch_date, planet, longitude, sign, retrograde, longitude_speed) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (fetch_date, planet) DO UPDATE \
             SET longitude = EXCLUDED.longitude, sign = EXCLUDED.sign, \
                 retrograde = EXCLUDED.retrograde, longitude_speed = EXCLUDED.longitude_speed",
        )
        .bind(today)
        .bind(snap.planet.name())
        .bind(snap.longitude)
        .bind(snap.sign)
        .bind(snap.retrograde)
        .bind(speed)
        .execute(pool.as_ref())
        .await;

        if let Err(e) = result {
            eprintln!("Failed to store transit for {}: {e}", snap.planet.name());
        }
    }

    println!("Daily transits stored (Swiss Ephemeris, sub-arcsecond).");
}

// ---------------------------------------------------------------------------
// Astrological score per ticker
// ---------------------------------------------------------------------------

pub async fn compute_astro_scores(pool: Arc<sqlx::PgPool>) {
    let today = Utc::now().date_naive();

    // Use Option<NaiveDate> — ipo_date is nullable since migration 0017
    let all_rows: Vec<(String, Option<NaiveDate>)> = match sqlx::query_as(
        "SELECT ticker, ipo_date FROM company_metadata ORDER BY ticker",
    )
    .fetch_all(pool.as_ref())
    .await
    {
        Ok(r) => r,
        Err(e) => { eprintln!("Failed to load company_metadata for scoring: {e}"); return; }
    };

    // Only score tickers with a known IPO date — can't build a birth chart without one
    let rows: Vec<(String, NaiveDate)> = all_rows
        .into_iter()
        .filter_map(|(t, d)| d.map(|date| (t, date)))
        .collect();

    println!("Computing astrological scores for {} tickers...", rows.len());

    for (ticker, ipo_date) in &rows {
        let natal        = NatalChart::compute(ticker, *ipo_date);
        let score        = compute_transit_score(&natal, today);
        let aspects_json = aspects_to_json(&score.active_aspects);

        let result = sqlx::query(
            "INSERT INTO astro_scores \
             (ticker, score_date, astro_score, astro_label, moon_phase, \
              moon_phase_deg, mercury_rx, active_aspects) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (ticker, score_date) DO UPDATE \
             SET astro_score    = EXCLUDED.astro_score, \
                 astro_label    = EXCLUDED.astro_label, \
                 moon_phase     = EXCLUDED.moon_phase, \
                 moon_phase_deg = EXCLUDED.moon_phase_deg, \
                 mercury_rx     = EXCLUDED.mercury_rx, \
                 active_aspects = EXCLUDED.active_aspects",
        )
        .bind(ticker)
        .bind(today)
        .bind(score.astro_score as f64)
        .bind(&score.astro_label)
        .bind(&score.moon_phase)
        .bind(score.moon_phase_deg)
        .bind(score.mercury_rx)
        .bind(&aspects_json)
        .execute(pool.as_ref())
        .await;

        match result {
            Ok(_) => {
                // Generate and store horoscope reading
                let reading = generate_horoscope(&score);
                let reading_json = horoscope_to_json(&reading);

                let horo_result = sqlx::query(
                    "INSERT INTO horoscope_readings \
                     (ticker, reading_date, reading, dominant_theme, confidence) \
                     VALUES ($1, $2, $3, $4, $5) \
                     ON CONFLICT (ticker, reading_date) DO UPDATE \
                     SET reading        = EXCLUDED.reading, \
                         dominant_theme  = EXCLUDED.dominant_theme, \
                         confidence      = EXCLUDED.confidence",
                )
                .bind(ticker)
                .bind(today)
                .bind(&reading_json)
                .bind(&reading.dominant_theme)
                .bind(reading.confidence as f64)
                .execute(pool.as_ref())
                .await;

                let horo_status = match horo_result {
                    Ok(_) => format!("horoscope: {}", reading.dominant_theme),
                    Err(e) => format!("horoscope err: {e}"),
                };

                println!(
                    "  {ticker}: {:.0} ({}) — {} aspects, Moon: {}{} | {}",
                    score.astro_score,
                    score.astro_label,
                    score.active_aspects.len(),
                    score.moon_phase,
                    if score.mercury_rx { ", Mercury Rx" } else { "" },
                    horo_status,
                );
            }
            Err(e) => eprintln!("Failed to store astro score for {ticker}: {e}"),
        }
    }

    println!("Astrological scoring and horoscope generation complete.");
}

// ---------------------------------------------------------------------------
// Astro ranking — Top 5 Favorable + Bottom 5 Misaligned
// ---------------------------------------------------------------------------

/// Ranking of tickers by astrological score, used to prioritize financial data fetching.
#[allow(dead_code)] // `total_scored` logged in future scraper summary output
pub struct AstroRanking {
    /// Top 5 most favorably aligned tickers: (ticker, score, dominant_theme)
    pub top_favorable: Vec<(String, f64, String)>,
    /// Bottom 5 most misaligned tickers: (ticker, score, dominant_theme)
    pub bottom_misaligned: Vec<(String, f64, String)>,
    /// Total number of scored tickers
    pub total_scored: usize,
}

impl AstroRanking {
    /// Returns a combined list of priority tickers (top + bottom, for financial data fetching).
    pub fn priority_tickers(&self) -> Vec<String> {
        let mut tickers: Vec<String> = self.top_favorable.iter().map(|(t, _, _)| t.clone()).collect();
        tickers.extend(self.bottom_misaligned.iter().map(|(t, _, _)| t.clone()));
        tickers
    }
}

/// Query the DB for today's astro scores and return the top 5 + bottom 5 rankings.
pub async fn compute_astro_ranking(pool: Arc<sqlx::PgPool>) -> AstroRanking {
    let today = chrono::Utc::now().date_naive();

    let top: Vec<(String, f64, String)> = sqlx::query_as(
        "SELECT a.ticker, a.astro_score, COALESCE(h.dominant_theme, a.astro_label) \
         FROM astro_scores a \
         LEFT JOIN horoscope_readings h ON h.ticker = a.ticker AND h.reading_date = a.score_date \
         WHERE a.score_date = $1 \
         ORDER BY a.astro_score DESC \
         LIMIT 5",
    )
    .bind(today)
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    let bottom: Vec<(String, f64, String)> = sqlx::query_as(
        "SELECT a.ticker, a.astro_score, COALESCE(h.dominant_theme, a.astro_label) \
         FROM astro_scores a \
         LEFT JOIN horoscope_readings h ON h.ticker = a.ticker AND h.reading_date = a.score_date \
         WHERE a.score_date = $1 \
         ORDER BY a.astro_score ASC \
         LIMIT 5",
    )
    .bind(today)
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM astro_scores WHERE score_date = $1",
    )
    .bind(today)
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0);

    // Log the rankings
    if !top.is_empty() || !bottom.is_empty() {
        println!("\n══════════════════════════════════════════════════════════");
        println!("  ASTRO RANKINGS — {}", today);
        println!("══════════════════════════════════════════════════════════");
        println!("  TOP 5 FAVORABLE:");
        for (i, (ticker, score, theme)) in top.iter().enumerate() {
            println!("    {}. {:<6} {:.0}  {}", i + 1, ticker, score, theme);
        }
        println!("  BOTTOM 5 MISALIGNED:");
        for (i, (ticker, score, theme)) in bottom.iter().enumerate() {
            println!("    {}. {:<6} {:.0}  {}", i + 1, ticker, score, theme);
        }
        println!("  Total scored: {}", total);
        println!("══════════════════════════════════════════════════════════\n");
    }

    AstroRanking {
        top_favorable: top,
        bottom_misaligned: bottom,
        total_scored: total as usize,
    }
}
