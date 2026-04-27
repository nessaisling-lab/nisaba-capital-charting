-- v3.0.3: Portfolio transaction log
-- Buy/sell log that auto-computes positions from transaction history.

CREATE TABLE IF NOT EXISTS transactions (
    id          SERIAL PRIMARY KEY,
    ticker      TEXT NOT NULL,
    action      TEXT NOT NULL CHECK (action IN ('BUY', 'SELL')),
    shares      REAL NOT NULL CHECK (shares > 0),
    price       REAL NOT NULL CHECK (price > 0),
    trade_date  DATE NOT NULL DEFAULT CURRENT_DATE,
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transactions_ticker ON transactions (ticker);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions (trade_date DESC);
