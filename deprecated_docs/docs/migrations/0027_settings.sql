-- v3.0.4: Settings key-value store
-- Persists user preferences: theme mode, refresh interval, API keys, etc.

CREATE TABLE IF NOT EXISTS settings (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);

-- Seed defaults
INSERT INTO settings (key, value) VALUES
    ('theme_mode', 'Auto'),
    ('refresh_interval_secs', '30')
ON CONFLICT DO NOTHING;
