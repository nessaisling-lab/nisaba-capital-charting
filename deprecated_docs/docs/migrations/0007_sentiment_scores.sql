-- Migration 007: News sentiment scores from Alpha Vantage NEWS_SENTIMENT
CREATE TABLE IF NOT EXISTS sentiment_scores (
    ticker TEXT NOT NULL,
    fetch_date DATE NOT NULL DEFAULT CURRENT_DATE,
    sentiment_score NUMERIC,        -- -1.0 to +1.0
    sentiment_label TEXT,           -- "Bullish", "Somewhat-Bullish", "Neutral", etc.
    PRIMARY KEY (ticker, fetch_date)
);
