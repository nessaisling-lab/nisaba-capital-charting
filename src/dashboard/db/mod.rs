pub mod ticker_data;
pub mod astro;
pub mod universe;
pub mod portfolio;

use sqlx::PgPool;
use std::sync::Arc;

// Re-export everything so existing `use crate::db::*` imports keep working.
pub use ticker_data::*;
pub use astro::*;
pub use universe::*;
pub use portfolio::*;

// ---------------------------------------------------------------------------
// Database connection
// ---------------------------------------------------------------------------

pub async fn connect_db(url: String) -> Result<Arc<PgPool>, String> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(3)
        .connect(&url)
        .await
        .map(Arc::new)
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Core queries
// ---------------------------------------------------------------------------

pub async fn load_tickers(pool: Arc<PgPool>) -> Result<Vec<String>, String> {
    sqlx::query_scalar::<_, String>(
        "SELECT ticker FROM tickers WHERE active = true ORDER BY ticker ASC",
    )
    .fetch_all(pool.as_ref()).await.map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Ticker autocomplete -- prefix + fuzzy company name search
// ---------------------------------------------------------------------------

pub async fn search_tickers(
    pool: Arc<PgPool>,
    prefix: String,
) -> Result<Vec<(String, String)>, String> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT ticker, COALESCE(company_name, ticker)
         FROM company_metadata
         WHERE ticker ILIKE $1 OR company_name ILIKE $2
         ORDER BY
           CASE WHEN ticker ILIKE $1 THEN 0 ELSE 1 END,
           ticker
         LIMIT 8",
    )
    .bind(format!("{}%", prefix.to_uppercase()))
    .bind(format!("%{}%", prefix))
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Settings key-value store
// ---------------------------------------------------------------------------

pub async fn fetch_settings(
    pool: Arc<PgPool>,
) -> Result<Vec<(String, String)>, String> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT key, value FROM settings ORDER BY key",
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub async fn upsert_setting(
    pool: Arc<PgPool>,
    key: String,
    value: String,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ($1, $2)
         ON CONFLICT (key) DO UPDATE SET value = $2",
    )
    .bind(&key)
    .bind(&value)
    .execute(pool.as_ref())
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
