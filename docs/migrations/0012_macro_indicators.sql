-- FRED macroeconomic indicators (Fed funds rate, CPI, unemployment, etc.)
CREATE TABLE IF NOT EXISTS macro_indicators (
    id          BIGSERIAL PRIMARY KEY,
    series_id   TEXT        NOT NULL,   -- FRED series ID e.g. "FEDFUNDS"
    series_name TEXT        NOT NULL,   -- human label e.g. "Fed Funds Rate"
    obs_date    DATE        NOT NULL,
    value       NUMERIC(18,6),          -- NULL allowed (FRED uses "." for missing)
    fetched_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (series_id, obs_date)
);

CREATE INDEX IF NOT EXISTS idx_macro_series ON macro_indicators (series_id, obs_date DESC);

-- Short interest data (Quandl/Nasdaq Data Link — FINRA short interest by ticker)
CREATE TABLE IF NOT EXISTS short_interest (
    id              BIGSERIAL PRIMARY KEY,
    ticker          TEXT    NOT NULL,
    settlement_date DATE    NOT NULL,
    short_volume    BIGINT,
    total_volume    BIGINT,
    short_pct       NUMERIC(7,4),   -- short_volume / total_volume * 100
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (ticker, settlement_date)
);

CREATE INDEX IF NOT EXISTS idx_short_ticker ON short_interest (ticker, settlement_date DESC);

-- Options flow — put/call ratio and open interest summary (Polygon.io)
CREATE TABLE IF NOT EXISTS options_flow (
    id              BIGSERIAL PRIMARY KEY,
    ticker          TEXT        NOT NULL,
    snapshot_date   DATE        NOT NULL,
    call_volume     BIGINT,
    put_volume      BIGINT,
    put_call_ratio  NUMERIC(8,4),
    call_oi         BIGINT,     -- open interest calls
    put_oi          BIGINT,     -- open interest puts
    pc_oi_ratio     NUMERIC(8,4),
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (ticker, snapshot_date)
);

CREATE INDEX IF NOT EXISTS idx_options_ticker ON options_flow (ticker, snapshot_date DESC);
