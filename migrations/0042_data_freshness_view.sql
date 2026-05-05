-- Wave 6.A4 — data freshness view aggregates per-ticker source completeness.
-- 5-source health score: prices (24h), fundamentals (30d), news (7d),
-- sentiment (7d), astro (1d). UI shows "●●●●○" 4/5 indicator on universe table.

CREATE OR REPLACE VIEW data_freshness AS
WITH last_dates AS (
    SELECT
        cm.ticker,
        (SELECT MAX(p.date)::timestamptz FROM price_data p WHERE p.ticker = cm.ticker)
            AS last_price_at,
        (SELECT MAX(f.fetch_date)::timestamptz FROM fundamental_metrics f WHERE f.ticker = cm.ticker)
            AS last_fund_at,
        (SELECT MAX(n.published) FROM news_articles n WHERE n.ticker = cm.ticker)
            AS last_news_at,
        (SELECT MAX(s.fetch_date)::timestamptz FROM sentiment_scores s WHERE s.ticker = cm.ticker)
            AS last_sent_at,
        (SELECT MAX(a.score_date)::timestamptz FROM astro_scores a WHERE a.ticker = cm.ticker)
            AS last_astro_at,
        (SELECT COUNT(DISTINCT p.data_source) FROM price_data p WHERE p.ticker = cm.ticker)
            AS price_source_count
    FROM company_metadata cm
)
SELECT
    ticker,
    last_price_at,
    last_fund_at,
    last_news_at,
    last_sent_at,
    last_astro_at,
    price_source_count,
    -- Each source contributes 1 to score if fresh within its threshold
    (CASE WHEN last_price_at > NOW() - INTERVAL '3 days'  THEN 1 ELSE 0 END +
     CASE WHEN last_fund_at  > NOW() - INTERVAL '30 days' THEN 1 ELSE 0 END +
     CASE WHEN last_news_at  > NOW() - INTERVAL '7 days'  THEN 1 ELSE 0 END +
     CASE WHEN last_sent_at  > NOW() - INTERVAL '7 days'  THEN 1 ELSE 0 END +
     CASE WHEN last_astro_at > NOW() - INTERVAL '2 days'  THEN 1 ELSE 0 END
    ) AS fresh_count
FROM last_dates;
