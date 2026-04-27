-- Migration 0016: Lagrange alert threshold crossings
-- Fires when a ticker enters Optimal (76-100) or Misaligned (0-24) territory.
-- ON CONFLICT DO NOTHING makes the crossing detection idempotent (safe to re-run).

CREATE TABLE IF NOT EXISTS lagrange_alerts (
    id          SERIAL      PRIMARY KEY,
    ticker      TEXT        NOT NULL,
    alert_date  DATE        NOT NULL,
    score       REAL        NOT NULL,
    label       TEXT        NOT NULL,   -- 'Optimal' or 'Misaligned'
    prev_label  TEXT,                   -- zone the ticker was in the day before
    alert_type  TEXT        NOT NULL,   -- 'entered_optimal' | 'entered_misaligned'
    is_read     BOOLEAN     NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (ticker, alert_date, alert_type)
);
