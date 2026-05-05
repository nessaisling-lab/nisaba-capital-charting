-- Wave 6.A1 — track which source each price row came from.
-- Lets us score source reliability over time and surface provenance in UI.
-- Defaults to 'alpha_vantage' since that's the existing primary source.

ALTER TABLE price_data
    ADD COLUMN IF NOT EXISTS data_source TEXT NOT NULL DEFAULT 'alpha_vantage';

CREATE INDEX IF NOT EXISTS idx_price_data_source ON price_data(data_source);
