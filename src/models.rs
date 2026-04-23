use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::FromRow;

/// One row of daily OHLCV data from price_data table
#[derive(Debug, Clone, FromRow)]
pub struct PriceRow {
    pub ticker: String,
    pub date: NaiveDate,
    pub open: rust_decimal::Decimal,
    pub high: rust_decimal::Decimal,
    pub low: rust_decimal::Decimal,
    pub close: rust_decimal::Decimal,
    pub volume: i64,
}

/// One row from the insider_trades table
#[derive(Debug, Clone, FromRow)]
pub struct InsiderTradeRow {
    pub ticker: String,
    pub insider_name: String,
    pub insider_title: Option<String>,
    pub transaction_date: NaiveDate,
    pub transaction_type: String, // "A" = acquired, "D" = disposed
    pub shares: rust_decimal::Decimal,
    pub price_per_share: rust_decimal::Decimal,
}

/// One row from institutional_holdings (used for 13F display in dashboard)
#[derive(Debug, Clone, FromRow)]
pub struct HoldingRow {
    pub institution_name: String,
    pub report_period: NaiveDate,
    pub shares_held: i64,
    pub market_value: rust_decimal::Decimal,
    pub investment_discretion: Option<String>,
}

/// One row from the filings table (used for 8-K display in dashboard)
#[derive(Debug, Clone, FromRow)]
pub struct FilingRow {
    pub ticker: String,
    pub filed_date: NaiveDate,
    pub items: Option<String>,
    pub edgar_url: String,
}

/// One row from news_articles (Finnhub company news)
#[derive(Debug, Clone, FromRow)]
pub struct NewsArticle {
    pub ticker: String,
    pub headline: String,
    pub summary: Option<String>,
    pub source: Option<String>,
    pub url: String,
    pub published_at: chrono::DateTime<chrono::Utc>,
}

/// One row from earnings_dates (Finnhub earnings calendar)
#[derive(Debug, Clone, FromRow)]
pub struct EarningsDate {
    pub ticker: String,
    pub earnings_date: NaiveDate,
    pub eps_estimate: Option<rust_decimal::Decimal>,
    pub eps_actual: Option<rust_decimal::Decimal>,
    pub revenue_estimate: Option<i64>,
}

/// One row from analyst_ratings (Finnhub recommendation trends)
#[derive(Debug, Clone, FromRow)]
pub struct AnalystRating {
    pub ticker: String,
    pub period: String,
    pub strong_buy: i32,
    pub buy: i32,
    pub hold: i32,
    pub sell: i32,
    pub strong_sell: i32,
}

/// One row from astro_scores
#[derive(Debug, Clone, FromRow)]
pub struct AstroScore {
    pub ticker:          String,
    pub score_date:      NaiveDate,
    pub astro_score:     Option<f64>,
    pub astro_label:     Option<String>,
    pub moon_phase:      Option<String>,
    pub moon_phase_deg:  Option<f64>,
    pub mercury_rx:      Option<bool>,
    // active_aspects decoded separately from JSONB
}

/// One row from natal_positions
#[derive(Debug, Clone, FromRow)]
pub struct NatalPosition {
    pub ticker:     String,
    pub planet:     String,
    pub longitude:  f64,
    pub sign:       String,
    pub degree:     f64,
    pub retrograde: bool,
}

/// One row from daily_transits
#[derive(Debug, Clone, FromRow)]
pub struct DailyTransit {
    pub fetch_date:  NaiveDate,
    pub planet:      String,
    pub longitude:   f64,
    pub sign:        String,
    pub retrograde:  bool,
}

/// One row from sentiment_scores (Alpha Vantage NEWS_SENTIMENT)
#[derive(Debug, Clone, FromRow)]
pub struct SentimentScore {
    pub ticker: String,
    pub fetch_date: NaiveDate,
    pub sentiment_score: Option<rust_decimal::Decimal>,
    pub sentiment_label: Option<String>,
}

/// One row from macro_indicators (FRED economic data)
#[derive(Debug, Clone, FromRow)]
pub struct MacroIndicator {
    pub series_id:   String,
    pub series_name: String,
    pub obs_date:    NaiveDate,
    pub value:       Option<rust_decimal::Decimal>,
}

/// One row from short_interest (Nasdaq Data Link FINRA short interest)
#[derive(Debug, Clone, FromRow)]
pub struct ShortInterest {
    pub ticker:          String,
    pub settlement_date: NaiveDate,
    pub short_volume:    Option<i64>,
    pub total_volume:    Option<i64>,
    pub short_pct:       Option<rust_decimal::Decimal>,
}

/// One row from options_flow (Polygon.io put/call ratio)
#[derive(Debug, Clone, FromRow)]
pub struct OptionsFlow {
    pub ticker:          String,
    pub snapshot_date:   NaiveDate,
    pub call_volume:     Option<i64>,
    pub put_volume:      Option<i64>,
    pub put_call_ratio:  Option<rust_decimal::Decimal>,
    pub call_oi:         Option<i64>,
    pub put_oi:          Option<i64>,
    pub pc_oi_ratio:     Option<rust_decimal::Decimal>,
}

/// Alpha Vantage TIME_SERIES_DAILY response shape
#[derive(Debug, Deserialize)]
pub struct AlphaVantageResponse {
    #[serde(rename = "Time Series (Daily)")]
    pub time_series: std::collections::HashMap<String, OhlcvEntry>,
}

#[derive(Debug, Deserialize)]
pub struct OhlcvEntry {
    #[serde(rename = "1. open")]
    pub open: String,
    #[serde(rename = "2. high")]
    pub high: String,
    #[serde(rename = "3. low")]
    pub low: String,
    #[serde(rename = "4. close")]
    pub close: String,
    #[serde(rename = "5. volume")]
    pub volume: String,
}

/// One row from lagrange_history
#[derive(Debug, Clone, FromRow)]
pub struct LagrangeHistory {
    pub ticker:      String,
    pub score_date:  chrono::NaiveDate,
    pub score:       f32,
    pub label:       String,
    pub fin_score:   Option<f32>,
    pub astro_score: Option<f32>,
    pub macro_score: Option<f32>,
    pub short_score: Option<f32>,
    pub concordance: Option<String>,
}

/// One row from lagrange_alerts
/// SQLx maps DATE → NaiveDate, BOOLEAN → bool automatically.
#[derive(Debug, Clone, FromRow)]
pub struct LagrangeAlert {
    pub id:         i32,
    pub ticker:     String,
    pub alert_date: NaiveDate,
    pub score:      f32,
    pub label:      String,
    pub prev_label: Option<String>,
    pub alert_type: String,
    pub is_read:    bool,
}

/// One row from fundamental_metrics (FMP key-metrics + ratios)
#[derive(Debug, Clone, FromRow)]
pub struct FundamentalMetric {
    pub ticker:            String,
    pub fetch_date:        NaiveDate,
    pub market_cap:        Option<i64>,
    pub pe_ratio:          Option<f64>,
    pub pb_ratio:          Option<f64>,
    pub ps_ratio:          Option<f64>,
    pub ev_ebitda:         Option<f64>,
    pub peg_ratio:         Option<f64>,
    pub price_to_fcf:      Option<f64>,
    pub roe:               Option<f64>,
    pub roa:               Option<f64>,
    pub net_margin:        Option<f64>,
    pub operating_margin:  Option<f64>,
    pub debt_equity:       Option<f64>,
    pub current_ratio:     Option<f64>,
    pub fcf:               Option<i64>,
    pub operating_cf:      Option<i64>,
    pub revenue:           Option<i64>,
    pub net_income:        Option<i64>,
    pub eps:               Option<f64>,
    pub dividend_yield:    Option<f64>,
    pub shares_outstanding: Option<i64>,
}

/// One row from portfolio_positions
#[derive(Debug, Clone, FromRow)]
pub struct PortfolioPosition {
    pub ticker:   String,
    pub shares:   f32,
    pub avg_cost: f32,
    pub notes:    Option<String>,
}
