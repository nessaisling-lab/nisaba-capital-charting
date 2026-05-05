-- Wave 6.A3 — analyst price targets per ticker.
-- Pulled from Finnhub /stock/price-target endpoint. One row per ticker
-- (latest fetch wins). Used in dashboard to compare current price vs
-- analyst consensus.

CREATE TABLE IF NOT EXISTS analyst_targets (
    ticker          TEXT PRIMARY KEY,
    fetch_date      DATE NOT NULL DEFAULT CURRENT_DATE,
    low_target      NUMERIC(10, 2),
    median_target   NUMERIC(10, 2),
    high_target     NUMERIC(10, 2),
    n_analysts      INTEGER,
    last_updated    TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_analyst_targets_fetched ON analyst_targets(fetch_date);
