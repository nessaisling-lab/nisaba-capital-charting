-- Migration 006: Analyst recommendation trends from Finnhub
CREATE TABLE IF NOT EXISTS analyst_ratings (
    ticker TEXT NOT NULL,
    period TEXT NOT NULL,           -- "YYYY-MM" format from Finnhub
    strong_buy INT DEFAULT 0,
    buy INT DEFAULT 0,
    hold INT DEFAULT 0,
    sell INT DEFAULT 0,
    strong_sell INT DEFAULT 0,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (ticker, period)
);
