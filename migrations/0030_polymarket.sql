-- v3.1.2: Polymarket prediction markets (ported from FinceptTerminal PolymarketService.cpp)
-- Stores top prediction markets from Polymarket's Gamma API.
-- Tracks question, outcome probabilities, volume, and category.
-- Used for macro sentiment gauges on the Overview tab.

CREATE TABLE IF NOT EXISTS polymarket_markets (
    id             SERIAL PRIMARY KEY,
    market_id      TEXT        NOT NULL,   -- Polymarket market ID
    question       TEXT        NOT NULL,   -- e.g. "Will the Fed cut rates in June 2025?"
    category       TEXT,                   -- e.g. "Economics", "Politics", "Crypto"
    outcome_yes    NUMERIC(6,4),           -- probability of "Yes" (0.0 to 1.0)
    outcome_no     NUMERIC(6,4),           -- probability of "No"
    volume         NUMERIC(18,2),          -- total trading volume in USDC
    liquidity      NUMERIC(18,2),          -- current liquidity
    active         BOOLEAN NOT NULL DEFAULT TRUE,
    end_date       TEXT,                   -- ISO date string or null
    slug           TEXT,
    fetched_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (market_id)
);

CREATE INDEX IF NOT EXISTS idx_polymarket_category ON polymarket_markets (category);
CREATE INDEX IF NOT EXISTS idx_polymarket_volume ON polymarket_markets (volume DESC);
