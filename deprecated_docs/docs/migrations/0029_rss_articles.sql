-- v3.1.1: RSS news aggregation (ported from FinceptTerminal NewsService.cpp)
-- Stores articles from 25+ RSS/Atom feeds across wire services, financial media,
-- central banks, and analysis sources. Separate from Finnhub news_articles
-- because RSS articles are market-wide, not per-ticker.

CREATE TABLE IF NOT EXISTS rss_articles (
    id           SERIAL PRIMARY KEY,
    feed_source  TEXT        NOT NULL,   -- e.g. "Reuters", "SEC", "Fed"
    category     TEXT        NOT NULL,   -- e.g. "wire", "markets", "central_bank", "analysis"
    headline     TEXT        NOT NULL,
    summary      TEXT,                   -- first 300 chars of description, HTML stripped
    link         TEXT        NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    fetched_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (link)
);

CREATE INDEX IF NOT EXISTS idx_rss_published ON rss_articles (published_at DESC);
CREATE INDEX IF NOT EXISTS idx_rss_category ON rss_articles (category);
CREATE INDEX IF NOT EXISTS idx_rss_source ON rss_articles (feed_source);
