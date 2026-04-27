-- v3.1.0: DBnomics international economics data
-- Reuses macro_indicators table with DBNOMICS: prefix on series_id.
-- No new table needed — same schema as FRED observations.
-- Series: ECB Euribor, BIS PBoC rate, IMF GDP forecast, Eurostat CPI,
--         OECD Leading Indicator, BIS Total Credit.

-- Partial index for efficient DBnomics-only queries
CREATE INDEX IF NOT EXISTS idx_macro_dbnomics
    ON macro_indicators (series_id, obs_date DESC)
    WHERE series_id LIKE 'DBNOMICS:%';
