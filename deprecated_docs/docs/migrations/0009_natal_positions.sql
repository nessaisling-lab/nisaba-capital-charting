-- Migration 0009: Pre-computed natal planet positions per company
-- Populated by the scraper's one-time natal chart seeder (runs if table is empty).
-- Ecliptic longitude 0-360° based on IPO date + time from company_metadata.
-- Planet names: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto

CREATE TABLE IF NOT EXISTS natal_positions (
    ticker     TEXT NOT NULL REFERENCES company_metadata(ticker) ON DELETE CASCADE,
    planet     TEXT NOT NULL,
    longitude  DOUBLE PRECISION NOT NULL,  -- ecliptic longitude 0-360°
    sign       TEXT NOT NULL,              -- 'Aries', 'Taurus', ..., 'Pisces'
    degree     DOUBLE PRECISION NOT NULL,  -- 0-30, degree within the sign
    retrograde BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (ticker, planet)
);

CREATE INDEX IF NOT EXISTS natal_positions_ticker_idx ON natal_positions (ticker);
