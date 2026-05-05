-- Wave 6.B1 — store detected aspect patterns alongside astro scores.
-- Each row in astro_scores gains an aspect_patterns JSONB column listing
-- geometric patterns (Grand Trine, T-Square, Yod, Stellium, etc.) detected
-- between the ticker's natal chart and the day's transits.

ALTER TABLE astro_scores
    ADD COLUMN IF NOT EXISTS aspect_patterns JSONB NOT NULL DEFAULT '[]'::jsonb;

-- GIN index for fast querying patterns by kind ("show me all tickers with a Grand Trine today").
CREATE INDEX IF NOT EXISTS idx_astro_scores_patterns
    ON astro_scores USING GIN (aspect_patterns);
