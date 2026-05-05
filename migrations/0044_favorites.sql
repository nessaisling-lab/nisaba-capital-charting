-- v11.5.B2: Favorites — persistent starred tickers, distinct from
-- recently_viewed (LRU cache). Star icon in header toggles membership.
CREATE TABLE IF NOT EXISTS favorites (
    ticker     TEXT PRIMARY KEY,
    starred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS favorites_starred_at_idx
    ON favorites (starred_at DESC);
