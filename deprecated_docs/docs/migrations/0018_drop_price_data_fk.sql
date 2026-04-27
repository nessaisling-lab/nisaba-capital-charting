-- Drop the FK from price_data → tickers so Tiingo can insert rows for
-- any ticker in company_metadata without requiring a tickers table entry.
-- The tickers table is intentionally limited to the scored watchlist;
-- price_data needs to grow to the full universe.
ALTER TABLE price_data DROP CONSTRAINT IF EXISTS price_data_ticker_fkey;

-- Composite index for efficient date-range and coverage queries used by
-- Tiingo's "day after latest stored price" look-ahead and by dashboard CTEs.
CREATE INDEX IF NOT EXISTS idx_price_data_ticker_date ON price_data (ticker, date DESC);
