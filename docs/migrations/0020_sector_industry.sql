-- Migration 0020: Add sector and industry columns to company_metadata
-- Required for v1.2.0 Universe Explorer filtering (filter by sector/zone)
-- Populated by fmp_enrich.rs from /v3/profile response during daily enrichment

ALTER TABLE company_metadata
    ADD COLUMN IF NOT EXISTS sector   TEXT,
    ADD COLUMN IF NOT EXISTS industry TEXT;

-- Index for fast sector/industry filtering in the Explorer panel
CREATE INDEX IF NOT EXISTS idx_company_metadata_sector   ON company_metadata (sector)   WHERE sector IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_company_metadata_industry ON company_metadata (industry) WHERE industry IS NOT NULL;

-- Add ticker_count to lagrange_history for Explorer pagination
-- Tracks how many tickers were in the scoring universe on a given day
ALTER TABLE lagrange_history
    ADD COLUMN IF NOT EXISTS ticker_count INTEGER;
