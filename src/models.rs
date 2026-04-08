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
