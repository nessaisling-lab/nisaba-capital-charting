-- Migration 005: Earnings calendar from Finnhub earnings calendar API
CREATE TABLE IF NOT EXISTS earnings_dates (
    ticker TEXT NOT NULL,
    earnings_date DATE NOT NULL,
    eps_estimate NUMERIC,
    eps_actual NUMERIC,
    revenue_estimate BIGINT,
    PRIMARY KEY (ticker, earnings_date)
);
