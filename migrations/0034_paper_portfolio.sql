-- Paper trading portfolio: current open positions
CREATE TABLE IF NOT EXISTS paper_portfolio (
    id          SERIAL PRIMARY KEY,
    ticker      TEXT        NOT NULL,
    shares      NUMERIC(12,4) NOT NULL,
    entry_price NUMERIC(12,4) NOT NULL,
    entry_date  DATE        NOT NULL,
    entry_score REAL,               -- Lagrange score at time of buy
    UNIQUE(ticker)                  -- one position per ticker
);
