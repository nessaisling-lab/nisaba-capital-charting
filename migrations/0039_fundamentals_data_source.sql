-- Wave 6.A2 — provenance on fundamental_metrics.
-- Tracks which source (FMP, Finnhub, AV) supplied each row so we can
-- score source reliability and surface in UI.

ALTER TABLE fundamental_metrics
    ADD COLUMN IF NOT EXISTS data_source TEXT NOT NULL DEFAULT 'fmp';

CREATE INDEX IF NOT EXISTS idx_fundamental_metrics_source
    ON fundamental_metrics(data_source);
