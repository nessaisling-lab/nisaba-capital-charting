-- Migration 004: News articles from Finnhub company news API
CREATE TABLE IF NOT EXISTS news_articles (
    id BIGSERIAL PRIMARY KEY,
    ticker TEXT NOT NULL,
    headline TEXT NOT NULL,
    summary TEXT,
    source TEXT,
    url TEXT,
    published_at TIMESTAMPTZ NOT NULL,
    UNIQUE(ticker, url)
);
