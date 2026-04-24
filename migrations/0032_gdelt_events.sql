-- GDELT geopolitical events (free, no API key)
CREATE TABLE IF NOT EXISTS gdelt_events (
    id              SERIAL PRIMARY KEY,
    url             TEXT NOT NULL UNIQUE,
    title           TEXT NOT NULL,
    source_country  TEXT,
    tone            REAL,           -- average tone (-10 to +10; negative = negative sentiment)
    themes          TEXT[],         -- GDELT theme codes (e.g., TAX_FNCACT, ECON_TRADE)
    locations       TEXT[],         -- location names mentioned
    domain          TEXT,           -- source domain (e.g., reuters.com)
    published_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_gdelt_published ON gdelt_events (published_at DESC);
