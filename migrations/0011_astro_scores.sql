-- Migration 0011: Per-ticker astrological Fear & Greed scores
-- Computed daily by the scraper after daily_transits is populated.
-- active_aspects stores the list of active transit-to-natal aspects as JSONB
-- for display in the dashboard's transits table.
--
-- active_aspects schema (array of objects):
-- [
--   {
--     "transit_planet": "Jupiter",
--     "natal_planet": "Sun",
--     "aspect": "Trine",
--     "orb": 1.4,
--     "score_delta": 12.3,
--     "effect": "Favorable"
--   },
--   ...
-- ]

CREATE TABLE IF NOT EXISTS astro_scores (
    ticker          TEXT NOT NULL,
    score_date      DATE NOT NULL,
    astro_score     DOUBLE PRECISION,      -- 0-100
    astro_label     TEXT,                  -- 'Extreme Fear' .. 'Extreme Greed'
    moon_phase      TEXT,                  -- 'New Moon', 'Waxing Crescent', etc.
    moon_phase_deg  DOUBLE PRECISION,      -- 0-360 phase angle
    mercury_rx      BOOLEAN DEFAULT false,
    active_aspects  JSONB,
    PRIMARY KEY (ticker, score_date)
);

CREATE INDEX IF NOT EXISTS astro_scores_ticker_date_idx ON astro_scores (ticker, score_date DESC);
