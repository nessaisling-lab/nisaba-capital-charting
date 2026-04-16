-- Migration 0010: Daily planetary transit positions
-- Refreshed once per day by the scraper cron job.
-- Retrograde flag is set when a planet's longitude is decreasing vs the prior day.

CREATE TABLE IF NOT EXISTS daily_transits (
    fetch_date  DATE NOT NULL,
    planet      TEXT NOT NULL,
    longitude   DOUBLE PRECISION NOT NULL,  -- ecliptic longitude 0-360°
    sign        TEXT NOT NULL,
    retrograde  BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (fetch_date, planet)
);

CREATE INDEX IF NOT EXISTS daily_transits_date_idx ON daily_transits (fetch_date DESC);
