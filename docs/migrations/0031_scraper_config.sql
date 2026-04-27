-- v4.0.4: Move hardcoded WATCHLIST, CIK_MAP, CUSIP_MAP, INSTITUTION_MAP
-- from const arrays in main.rs to DB-backed config tables.
-- Scraper loads from these tables at startup; falls back to defaults if empty.

CREATE TABLE IF NOT EXISTS scraper_watchlist (
    ticker      TEXT PRIMARY KEY,
    cik         TEXT,           -- SEC CIK number for EDGAR lookups
    cusip       TEXT,           -- CUSIP for 13F holdings matching
    active      BOOLEAN NOT NULL DEFAULT true,
    added_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO scraper_watchlist (ticker, cik, cusip) VALUES
    ('AAPL',  '0000320193', '037833100'),
    ('MSFT',  '0000789019', '594918104'),
    ('GOOGL', '0001652044', '02079K305'),
    ('AMZN',  '0001018724', '023135106'),
    ('NVDA',  '0001045810', '67066G104'),
    ('META',  '0001326801', '30303M102'),
    ('TSLA',  '0001318605', '88160R101'),
    ('JPM',   '0000019617', '46625H100'),
    ('V',     '0001403161', '92826C839'),
    ('UNH',   '0000731766', '91324P102')
ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS scraper_institutions (
    cik     TEXT PRIMARY KEY,
    name    TEXT NOT NULL,
    active  BOOLEAN NOT NULL DEFAULT true
);

INSERT INTO scraper_institutions (cik, name) VALUES
    ('0000102909', 'Vanguard Group Inc.'),
    ('0001364742', 'BlackRock Inc.'),
    ('0000093751', 'State Street Corporation'),
    ('0000315066', 'Fidelity Management & Research')
ON CONFLICT DO NOTHING;
