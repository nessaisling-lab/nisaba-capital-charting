-- Migration 0021: Extended natal chart data for Swiss Ephemeris upgrade
--
-- The natal_positions table already supports NorthNode, SouthNode, and Chiron
-- as regular planet rows (planet TEXT column is freeform). This migration adds:
--   1. natal_angles: Ascendant + MC computed from Swiss Eph house system
--   2. longitude_speed column on daily_transits for applying/separating detection
--
-- No existing data is modified. New columns are nullable for backward compatibility.

-- Natal chart angles (Ascendant + Midheaven) per ticker.
-- Computed using Whole Sign houses at NYSE location (40.71°N, 74.01°W).
CREATE TABLE IF NOT EXISTS natal_angles (
    ticker    TEXT NOT NULL REFERENCES company_metadata(ticker) ON DELETE CASCADE,
    ascendant DOUBLE PRECISION,  -- ecliptic longitude 0-360°
    mc        DOUBLE PRECISION,  -- Midheaven longitude 0-360°
    PRIMARY KEY (ticker)
);

-- Add longitude_speed to daily_transits for applying/separating detection.
-- Positive = direct motion, negative = retrograde. Degrees per day.
ALTER TABLE daily_transits
    ADD COLUMN IF NOT EXISTS longitude_speed DOUBLE PRECISION;

-- Re-seed natal charts: delete existing to force re-computation with Swiss Eph.
-- The scraper's seed_natal_charts() will re-populate with 12-13 bodies
-- (10 classical + nodes + Chiron) at sub-arcsecond accuracy.
-- Uncomment the line below to force a full re-seed:
-- DELETE FROM natal_positions;
