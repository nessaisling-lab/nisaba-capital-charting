-- Paper trading account: tracks simulated capital + idempotency guard
CREATE TABLE IF NOT EXISTS paper_account (
    id            SERIAL PRIMARY KEY,
    initial_capital NUMERIC(12,2) NOT NULL DEFAULT 100000.00,
    cash_balance    NUMERIC(12,2) NOT NULL DEFAULT 100000.00,
    last_sim_date   DATE,          -- idempotency: skip if already simulated today
    created_at      TIMESTAMPTZ   NOT NULL DEFAULT now()
);

-- Seed with a single default account ($100K)
INSERT INTO paper_account (initial_capital, cash_balance)
SELECT 100000.00, 100000.00
WHERE NOT EXISTS (SELECT 1 FROM paper_account);
