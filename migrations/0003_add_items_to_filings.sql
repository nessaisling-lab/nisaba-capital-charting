-- Migration 003: Add items column to filings for 8-K event type metadata
-- Items is a comma-separated list of SEC item codes e.g. "2.02,9.01"
ALTER TABLE filings ADD COLUMN IF NOT EXISTS items TEXT;
