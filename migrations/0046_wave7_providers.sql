-- Wave 7 — generic provider time-series store. One table for all 10
-- providers (World Bank, IMF, ECB, CFTC, BLS, EIA, OFR, Treasury Direct,
-- CoinGecko, buffer). Composite key (provider, series_id, observation_date)
-- accommodates any indicator from any source.
--
-- Design rationale: each provider has its own indicator naming (NY.GDP.MKTP.CD
-- from World Bank, DEXXEU from FRED, etc). Rather than spawn 10 tables, store
-- everything keyed by provider + series_id. Queries filter by provider when
-- a specific source is needed.

CREATE TABLE IF NOT EXISTS provider_observations (
    provider          TEXT NOT NULL,
    series_id         TEXT NOT NULL,
    observation_date  DATE NOT NULL,
    value             DOUBLE PRECISION,
    label             TEXT,                  -- human-readable series name
    region            TEXT,                  -- ISO country code or region
    unit              TEXT,                  -- "USD", "%", "index", etc
    fetched_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (provider, series_id, region, observation_date)
);

CREATE INDEX IF NOT EXISTS provider_observations_provider_idx
    ON provider_observations (provider, observation_date DESC);

CREATE INDEX IF NOT EXISTS provider_observations_series_idx
    ON provider_observations (series_id, observation_date DESC);
