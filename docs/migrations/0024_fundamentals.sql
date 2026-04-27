-- v2.2.1: Fundamental financial metrics table
-- Stores key valuation ratios and financial data per ticker per date.
-- Source: FMP /v3/key-metrics and /v3/ratios endpoints.
-- One row per ticker per fetch date; latest row is the "current" fundamentals.

CREATE TABLE IF NOT EXISTS fundamental_metrics (
    id            SERIAL PRIMARY KEY,
    ticker        TEXT NOT NULL,
    fetch_date    DATE NOT NULL DEFAULT CURRENT_DATE,

    -- Valuation ratios
    market_cap       BIGINT,
    pe_ratio         DOUBLE PRECISION,
    pb_ratio         DOUBLE PRECISION,
    ps_ratio         DOUBLE PRECISION,
    ev_ebitda        DOUBLE PRECISION,
    peg_ratio        DOUBLE PRECISION,
    price_to_fcf     DOUBLE PRECISION,

    -- Profitability
    roe              DOUBLE PRECISION,
    roa              DOUBLE PRECISION,
    net_margin       DOUBLE PRECISION,
    operating_margin DOUBLE PRECISION,

    -- Balance sheet
    debt_equity      DOUBLE PRECISION,
    current_ratio    DOUBLE PRECISION,

    -- Cash flow
    fcf              BIGINT,
    operating_cf     BIGINT,

    -- Income statement highlights
    revenue          BIGINT,
    net_income       BIGINT,
    eps              DOUBLE PRECISION,

    -- Dividend
    dividend_yield   DOUBLE PRECISION,

    -- Shares
    shares_outstanding BIGINT,

    UNIQUE (ticker, fetch_date)
);

CREATE INDEX IF NOT EXISTS idx_fundamental_metrics_ticker
    ON fundamental_metrics (ticker, fetch_date DESC);
