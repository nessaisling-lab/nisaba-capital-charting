-- v2.6.3: Named watchlists — users can create multiple watchlists
-- Each watchlist has a name and contains ticker symbols.
-- The "Default" watchlist inherits the existing active tickers.

CREATE TABLE IF NOT EXISTS watchlists (
    id          SERIAL PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS watchlist_members (
    id            SERIAL PRIMARY KEY,
    watchlist_id  INTEGER NOT NULL REFERENCES watchlists(id) ON DELETE CASCADE,
    ticker        TEXT NOT NULL,
    added_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (watchlist_id, ticker)
);

CREATE INDEX IF NOT EXISTS idx_watchlist_members_wl ON watchlist_members (watchlist_id);

-- Seed the "Default" watchlist from existing active tickers
INSERT INTO watchlists (name) VALUES ('Default') ON CONFLICT DO NOTHING;

INSERT INTO watchlist_members (watchlist_id, ticker)
SELECT w.id, t.ticker
FROM tickers t, watchlists w
WHERE t.active = true AND w.name = 'Default'
ON CONFLICT DO NOTHING;
