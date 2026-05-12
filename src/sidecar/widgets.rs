//! Wave 8.4 — OpenBB Workspace widget manifest.
//!
//! OpenBB Workspace consumes a `widgets.json` document describing the
//! widgets a backend exposes. Format docs: https://docs.openbb.co/
//! Workspace will GET /widgets.json once on backend registration, then
//! call individual endpoints based on user widget choice.

use axum::Json;

pub async fn manifest() -> Json<serde_json::Value> {
    Json(serde_json::json!([
        {
            "name": "Lagrange Composite Score",
            "description": "Astro-informed composite signal blending astrology, financials, macro, short interest, and sentiment.",
            "category": "Nisaba Engine",
            "widgetId": "pursuit_lagrange",
            "endpoint": "/tickers/{ticker}/lagrange",
            "type": "table",
            "params": [
                {"paramName": "ticker", "value": "AAPL", "label": "Ticker", "type": "ticker"}
            ],
            "data": {
                "table": {
                    "showAll": true,
                    "columnsDefs": [
                        {"field": "date", "headerName": "Date"},
                        {"field": "score", "headerName": "Lagrange"},
                        {"field": "label", "headerName": "Zone"},
                        {"field": "fin_score", "headerName": "Fin"},
                        {"field": "astro_score", "headerName": "Astro"},
                        {"field": "macro_score", "headerName": "Macro"},
                        {"field": "short_score", "headerName": "Short"},
                        {"field": "concordance", "headerName": "Concordance"}
                    ]
                }
            }
        },
        {
            "name": "Astrology Score",
            "description": "Astro Score for a ticker — composite of natal chart, current transits, aspect patterns, fixed stars, and Arabic Parts.",
            "category": "Nisaba Engine",
            "widgetId": "pursuit_astro",
            "endpoint": "/tickers/{ticker}/astro",
            "type": "metric",
            "params": [
                {"paramName": "ticker", "value": "AAPL", "label": "Ticker", "type": "ticker"}
            ]
        },
        {
            "name": "Pursuit OHLCV",
            "description": "Daily OHLCV history (multi-source cascade: AV → Yahoo → Stooq).",
            "category": "Nisaba Engine",
            "widgetId": "pursuit_prices",
            "endpoint": "/tickers/{ticker}/prices",
            "type": "table",
            "params": [
                {"paramName": "ticker", "value": "AAPL", "label": "Ticker", "type": "ticker"},
                {"paramName": "limit", "value": "252", "label": "Days", "type": "number"}
            ]
        },
        {
            "name": "World Bank Indicators",
            "description": "Headline economic indicators (GDP, CPI, unemployment, debt/GDP) for major economies.",
            "category": "Nisaba Engine / Wave 7 providers",
            "widgetId": "pursuit_world_bank",
            "endpoint": "/series/world_bank/{series_id}",
            "type": "table",
            "params": [
                {"paramName": "series_id", "value": "NY.GDP.MKTP.CD", "label": "Indicator", "type": "text"},
                {"paramName": "region", "value": "USA", "label": "Country (ISO)", "type": "text"}
            ]
        },
        {
            "name": "Treasury Yield Curve",
            "description": "US Treasury daily constant-maturity rates (1mo through 30yr).",
            "category": "Nisaba Engine / Wave 7 providers",
            "widgetId": "pursuit_treasury",
            "endpoint": "/series/treasury_direct/{series_id}",
            "type": "table",
            "params": [
                {"paramName": "series_id", "value": "treasury_10y", "label": "Maturity", "type": "text"}
            ]
        },
        {
            "name": "OFR Financial Stress Index",
            "description": "Office of Financial Research composite stress signal (33-component daily index).",
            "category": "Nisaba Engine / Wave 7 providers",
            "widgetId": "pursuit_ofr",
            "endpoint": "/series/ofr/fsi",
            "type": "chart"
        },
        {
            "name": "CoinGecko Crypto",
            "description": "Top 20 cryptocurrencies — price, market cap, 24h volume, 24h % change.",
            "category": "Nisaba Engine / Wave 7 providers",
            "widgetId": "pursuit_coingecko",
            "endpoint": "/series/coingecko/{series_id}",
            "type": "table",
            "params": [
                {"paramName": "series_id", "value": "price_usd", "label": "Series", "type": "text"},
                {"paramName": "region", "value": "BTC", "label": "Symbol", "type": "text"}
            ]
        }
    ]))
}
