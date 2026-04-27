-- Paper trading trade log: every simulated buy/sell
CREATE TABLE IF NOT EXISTS paper_trades (
    id          SERIAL PRIMARY KEY,
    ticker      TEXT        NOT NULL,
    action      TEXT        NOT NULL CHECK (action IN ('BUY', 'SELL')),
    shares      NUMERIC(12,4) NOT NULL,
    price       NUMERIC(12,4) NOT NULL,
    score       REAL,               -- Lagrange score at time of trade
    trade_date  DATE        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_paper_trades_ticker ON paper_trades(ticker);
CREATE INDEX IF NOT EXISTS idx_paper_trades_date   ON paper_trades(trade_date);
