-- v11.5.E2: Wikipedia summary cache. One row per ticker. 30-day TTL
-- enforced at fetch time, not via scheduled deletion (history may be
-- valuable even when stale).
CREATE TABLE IF NOT EXISTS wiki_summary (
    ticker         TEXT PRIMARY KEY,
    title          TEXT NOT NULL,
    extract        TEXT,
    thumbnail_url  TEXT,
    wikipedia_url  TEXT,
    fetched_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS wiki_summary_fetched_at_idx
    ON wiki_summary (fetched_at DESC);
