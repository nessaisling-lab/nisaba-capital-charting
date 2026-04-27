-- Migration 0013: Lagrange score history + portfolio positions

-- Daily Lagrange Score snapshots (one row per ticker per day)
CREATE TABLE IF NOT EXISTS lagrange_history (
    id           SERIAL PRIMARY KEY,
    ticker       TEXT        NOT NULL,
    score_date   DATE        NOT NULL,
    score        REAL        NOT NULL,
    label        TEXT        NOT NULL,
    fin_score    REAL,
    astro_score  REAL,
    macro_score  REAL,
    short_score  REAL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (ticker, score_date)
);

-- Portfolio positions (user-editable via portfolio_seed.sql)
CREATE TABLE IF NOT EXISTS portfolio_positions (
    id           SERIAL PRIMARY KEY,
    ticker       TEXT        NOT NULL UNIQUE,
    shares       REAL        NOT NULL DEFAULT 0,
    avg_cost     REAL        NOT NULL DEFAULT 0,
    notes        TEXT,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
