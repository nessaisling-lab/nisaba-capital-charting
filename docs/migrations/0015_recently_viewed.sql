-- Migration 0015: Recently viewed tickers
-- Tracks the last 10 tickers the user selected in the dashboard.
-- PRIMARY KEY on ticker so each ticker appears at most once (upsert updates viewed_at).

CREATE TABLE IF NOT EXISTS recently_viewed (
    ticker    TEXT        NOT NULL PRIMARY KEY,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
