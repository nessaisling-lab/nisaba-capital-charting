-- Migration 001: Initial schema
-- Run with: sqlx migrate run

-- Master watchlist
CREATE TABLE IF NOT EXISTS tickers (
    id         SERIAL PRIMARY KEY,
    ticker     TEXT UNIQUE NOT NULL,
    cik        TEXT,
    name       TEXT,
    sector     TEXT,
    active     BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed AAPL for MVP
INSERT INTO tickers (ticker, name, sector)
VALUES ('AAPL', 'Apple Inc.', 'Technology')
ON CONFLICT (ticker) DO NOTHING;

-- Daily OHLCV from Alpha Vantage
CREATE TABLE IF NOT EXISTS price_data (
    id         SERIAL PRIMARY KEY,
    ticker     TEXT NOT NULL REFERENCES tickers(ticker),
    date       DATE NOT NULL,
    open       NUMERIC(12,4) NOT NULL,
    high       NUMERIC(12,4) NOT NULL,
    low        NUMERIC(12,4) NOT NULL,
    close      NUMERIC(12,4) NOT NULL,
    volume     BIGINT NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (ticker, date)
);

-- EDGAR filing feed (8-K, Form 4, 13F) — stub for post-MVP
CREATE TABLE IF NOT EXISTS filings (
    id               SERIAL PRIMARY KEY,
    cik              TEXT NOT NULL,
    ticker           TEXT,
    form_type        TEXT NOT NULL,
    filed_date       DATE NOT NULL,
    period_of_report DATE,
    accession_number TEXT UNIQUE NOT NULL,
    edgar_url        TEXT NOT NULL,
    fetched_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Form 4 parsed detail — stub for post-MVP
CREATE TABLE IF NOT EXISTS insider_trades (
    id                 SERIAL PRIMARY KEY,
    accession_number   TEXT NOT NULL REFERENCES filings(accession_number),
    ticker             TEXT NOT NULL,
    insider_name       TEXT NOT NULL,
    insider_title      TEXT,
    transaction_date   DATE NOT NULL,
    transaction_type   TEXT NOT NULL,
    shares             NUMERIC(16,4) NOT NULL,
    price_per_share    NUMERIC(12,4) NOT NULL,
    shares_owned_after NUMERIC(16,4),
    fetched_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 13F quarterly holdings — stub for post-MVP
CREATE TABLE IF NOT EXISTS institutional_holdings (
    id                    SERIAL PRIMARY KEY,
    accession_number      TEXT NOT NULL REFERENCES filings(accession_number),
    institution_cik       TEXT NOT NULL,
    institution_name      TEXT NOT NULL,
    report_period         DATE NOT NULL,
    ticker                TEXT,
    cusip                 TEXT NOT NULL,
    shares_held           BIGINT NOT NULL,
    market_value          NUMERIC(18,2) NOT NULL,
    investment_discretion TEXT,
    fetched_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Ingestion audit trail
CREATE TABLE IF NOT EXISTS fetch_log (
    id            SERIAL PRIMARY KEY,
    source        TEXT NOT NULL,
    ticker        TEXT,
    fetch_type    TEXT NOT NULL,
    fetched_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    status        TEXT NOT NULL,
    error_message TEXT
);
