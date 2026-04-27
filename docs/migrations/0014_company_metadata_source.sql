-- Migration 0014: Add data provenance columns to company_metadata
-- data_source: 'manual' (hand-seeded in 0008) vs 'polygon' (bulk seeded from Polygon.io API)
-- seeded_at:   timestamp when natal chart was last computed for this company

ALTER TABLE company_metadata
    ADD COLUMN IF NOT EXISTS data_source TEXT NOT NULL DEFAULT 'manual',
    ADD COLUMN IF NOT EXISTS seeded_at   TIMESTAMPTZ;
