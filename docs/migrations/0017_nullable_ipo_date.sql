-- Migration 0017: make ipo_date nullable so we can insert tickers from Polygon
-- even when list_date is missing, then fill dates later via AV OVERVIEW enrichment.
ALTER TABLE company_metadata ALTER COLUMN ipo_date DROP NOT NULL;
