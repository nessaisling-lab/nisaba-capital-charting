-- Migration 0036: RSS-derived tone sentiment scores per ticker.
-- Keyword-based sentiment from the 25 RSS feeds in rss_articles.
-- Supplements Alpha Vantage NEWS_SENTIMENT (rate-limited to 25 calls/day).
-- tone_score uses the same -1.0 to +1.0 scale as AV for interoperability.

CREATE TABLE IF NOT EXISTS rss_tone_scores (
    ticker        TEXT    NOT NULL,
    score_date    DATE    NOT NULL,
    tone_score    NUMERIC(6,4),      -- -1.0 to +1.0
    tone_label    TEXT,               -- Bearish / Somewhat-Bearish / Neutral / Somewhat-Bullish / Bullish
    article_count INTEGER DEFAULT 0,  -- how many articles matched this ticker
    PRIMARY KEY (ticker, score_date)
);

CREATE INDEX IF NOT EXISTS idx_rss_tone_ticker ON rss_tone_scores (ticker);
