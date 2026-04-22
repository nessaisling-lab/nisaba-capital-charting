use chrono::{Datelike, NaiveDate, Utc};
use pursuit_week4_automation::astrology::ephemeris::{date_to_jdn, snapshot_all};
use pursuit_week4_automation::astrology::natal::{aspects_to_json, compute_transit_score, NatalChart};
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
        println!("  Natal chart seeded: {ticker} (IPO: {ipo_date})");
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
    let snapshots = snapshot_all(jdn);

    println!("Storing {} planetary positions for {today}...", snapshots.len());

    for snap in &snapshots {
        let result = sqlx::query(
            "INSERT INTO daily_transits (fetch_date, planet, longitude, sign, retrograde) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (fetch_date, planet) DO UPDATE \
             SET longitude = EXCLUDED.longitude, sign = EXCLUDED.sign, \
                 retrograde = EXCLUDED.retrograde",
        )
        .bind(today)
        .bind(snap.planet.name())
        .bind(snap.longitude)
        .bind(snap.sign)
        .bind(snap.retrograde)
        .execute(pool.as_ref())
        .await;

        if let Err(e) = result {
            eprintln!("Failed to store transit for {}: {e}", snap.planet.name());
        }
    }

    println!("Daily transits stored.");
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
            Ok(_) => println!(
                "  {ticker}: {:.0} ({}) — {} aspects, Moon: {}{}",
                score.astro_score,
                score.astro_label,
                score.active_aspects.len(),
                score.moon_phase,
                if score.mercury_rx { ", Mercury Rx" } else { "" },
            ),
            Err(e) => eprintln!("Failed to store astro score for {ticker}: {e}"),
        }
    }

    println!("Astrological scoring complete.");
}
