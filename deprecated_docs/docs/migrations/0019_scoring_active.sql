-- scoring_active: separates "visible in watchlist panel" from "included in
-- Lagrange scoring universe". Defaults to true so all current tickers keep
-- their scores. Set to false to suppress a ticker from scoring without
-- removing it from the tickers table.
ALTER TABLE tickers ADD COLUMN IF NOT EXISTS scoring_active BOOLEAN NOT NULL DEFAULT true;

CREATE INDEX IF NOT EXISTS idx_tickers_scoring_active ON tickers (scoring_active) WHERE scoring_active = true;
