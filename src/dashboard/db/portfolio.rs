use crate::error::SqlResultExt;
use nisaba_engine::models::PortfolioPosition;
use sqlx::PgPool;
use std::sync::Arc;

pub async fn fetch_portfolio(pool: Arc<PgPool>) -> Result<Vec<PortfolioPosition>, String> {
    sqlx::query_as(
        "SELECT ticker, shares, avg_cost, notes
         FROM portfolio_positions
         ORDER BY ticker"
    )
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_portfolio")
}

/// Import tickers from a list into portfolio_positions with 0 shares/cost (placeholders).
/// Skips tickers that already exist in portfolio.
pub async fn import_tickers_to_portfolio(
    pool: Arc<PgPool>,
    tickers: Vec<String>,
) -> Result<u64, String> {
    let mut imported = 0u64;
    for ticker in &tickers {
        let result = sqlx::query(
            "INSERT INTO portfolio_positions (ticker, shares, avg_cost, notes)
             VALUES ($1, 0, 0, 'Imported from watchlist')
             ON CONFLICT (ticker) DO NOTHING",
        )
        .bind(ticker)
        .execute(pool.as_ref())
        .await
        .ctx("import_tickers_to_portfolio")?;
        imported += result.rows_affected();
    }
    Ok(imported)
}

// ---------------------------------------------------------------------------
// Portfolio with current prices (for P&L tracking)
// ---------------------------------------------------------------------------

/// Portfolio position with latest price and astro score for P&L display.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct PortfolioPnlRow {
    pub ticker:      String,
    pub shares:      f32,
    pub avg_cost:    f32,
    pub notes:       Option<String>,
    pub last_close:  Option<rust_decimal::Decimal>,
    pub astro_score: Option<f64>,
    pub astro_label: Option<String>,
}

pub async fn fetch_portfolio_pnl(
    pool: Arc<PgPool>,
) -> Result<Vec<PortfolioPnlRow>, String> {
    sqlx::query_as::<_, PortfolioPnlRow>(
        "SELECT pp.ticker, pp.shares, pp.avg_cost, pp.notes,
                p.close AS last_close,
                a.astro_score, a.astro_label
         FROM portfolio_positions pp
         LEFT JOIN LATERAL (
             SELECT close FROM price_data
             WHERE ticker = pp.ticker ORDER BY date DESC LIMIT 1
         ) p ON true
         LEFT JOIN LATERAL (
             SELECT astro_score, astro_label FROM astro_scores
             WHERE ticker = pp.ticker ORDER BY score_date DESC LIMIT 1
         ) a ON true
         ORDER BY pp.ticker",
    )
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_portfolio_pnl")
}

// ---------------------------------------------------------------------------
// Transaction log
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct TransactionRow {
    pub id:         i32,
    pub ticker:     String,
    pub action:     String,
    pub shares:     f32,
    pub price:      f32,
    pub trade_date: chrono::NaiveDate,
    pub notes:      Option<String>,
}

pub async fn fetch_transactions(
    pool: Arc<PgPool>,
) -> Result<Vec<TransactionRow>, String> {
    sqlx::query_as::<_, TransactionRow>(
        "SELECT id, ticker, action, shares, price, trade_date, notes
         FROM transactions ORDER BY trade_date DESC, id DESC",
    )
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_transactions")
}

pub async fn insert_transaction(
    pool: Arc<PgPool>,
    ticker: String,
    action: String,
    shares: f32,
    price: f32,
    trade_date: chrono::NaiveDate,
    notes: Option<String>,
) -> Result<TransactionRow, String> {
    sqlx::query_as::<_, TransactionRow>(
        "INSERT INTO transactions (ticker, action, shares, price, trade_date, notes)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, ticker, action, shares, price, trade_date, notes",
    )
    .bind(&ticker)
    .bind(&action)
    .bind(shares)
    .bind(price)
    .bind(trade_date)
    .bind(notes.as_deref())
    .fetch_one(pool.as_ref())
    .await
    .ctx("insert_transaction")
}

pub async fn delete_transaction(
    pool: Arc<PgPool>,
    id: i32,
) -> Result<(), String> {
    sqlx::query("DELETE FROM transactions WHERE id = $1")
        .bind(id)
        .execute(pool.as_ref())
        .await
        .ctx("delete_transaction")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Named Watchlists -- CRUD for multiple named watchlists
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NamedWatchlist {
    pub id:   i32,
    pub name: String,
}

pub async fn fetch_named_watchlists(
    pool: Arc<PgPool>,
) -> Result<Vec<NamedWatchlist>, String> {
    sqlx::query_as::<_, NamedWatchlist>(
        "SELECT id, name FROM watchlists ORDER BY id",
    )
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_named_watchlists")
}

pub async fn fetch_watchlist_tickers(
    pool: Arc<PgPool>,
    watchlist_id: i32,
) -> Result<Vec<String>, String> {
    sqlx::query_scalar(
        "SELECT ticker FROM watchlist_members WHERE watchlist_id = $1 ORDER BY ticker",
    )
    .bind(watchlist_id)
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_watchlist_tickers")
}

pub async fn create_watchlist(
    pool: Arc<PgPool>,
    name: String,
) -> Result<NamedWatchlist, String> {
    sqlx::query_as::<_, NamedWatchlist>(
        "INSERT INTO watchlists (name) VALUES ($1) RETURNING id, name",
    )
    .bind(&name)
    .fetch_one(pool.as_ref())
    .await
    .ctx("create_watchlist")
}

pub async fn add_to_watchlist(
    pool: Arc<PgPool>,
    watchlist_id: i32,
    ticker: String,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO watchlist_members (watchlist_id, ticker) VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
    )
    .bind(watchlist_id)
    .bind(&ticker)
    .execute(pool.as_ref())
    .await
    .ctx("add_to_watchlist")?;
    Ok(())
}

pub async fn remove_from_watchlist(
    pool: Arc<PgPool>,
    watchlist_id: i32,
    ticker: String,
) -> Result<(), String> {
    sqlx::query(
        "DELETE FROM watchlist_members WHERE watchlist_id = $1 AND ticker = $2",
    )
    .bind(watchlist_id)
    .bind(&ticker)
    .execute(pool.as_ref())
    .await
    .ctx("remove_from_watchlist")?;
    Ok(())
}

pub async fn delete_watchlist(
    pool: Arc<PgPool>,
    watchlist_id: i32,
) -> Result<(), String> {
    sqlx::query("DELETE FROM watchlists WHERE id = $1")
        .bind(watchlist_id)
        .execute(pool.as_ref())
        .await
        .ctx("delete_watchlist")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Recently viewed tickers
// ---------------------------------------------------------------------------

pub async fn fetch_recently_viewed(pool: Arc<PgPool>) -> Result<Vec<String>, String> {
    sqlx::query_scalar::<_, String>(
        "SELECT ticker FROM recently_viewed ORDER BY viewed_at DESC LIMIT 8",
    )
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_recently_viewed")
}

/// Upserts a ticker into recently_viewed and prunes to the 10 most recent.
pub async fn upsert_recently_viewed(pool: Arc<PgPool>, ticker: String) {
    let _ = sqlx::query(
        "INSERT INTO recently_viewed (ticker, viewed_at) VALUES ($1, NOW())
         ON CONFLICT (ticker) DO UPDATE SET viewed_at = NOW()",
    )
    .bind(&ticker)
    .execute(pool.as_ref())
    .await;

    let _ = sqlx::query(
        "DELETE FROM recently_viewed
         WHERE ticker NOT IN (
             SELECT ticker FROM recently_viewed ORDER BY viewed_at DESC LIMIT 8
         )",
    )
    .execute(pool.as_ref())
    .await;
}

// ---------------------------------------------------------------------------
// Wikipedia summary — v11.5.E
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct WikiSummary {
    #[allow(dead_code)]
    pub ticker: String,
    pub title: String,
    pub extract: Option<String>,
    pub thumbnail_url: Option<String>,
    pub wikipedia_url: Option<String>,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

pub async fn fetch_wiki_thumbnail(url: String) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .user_agent("NisabaEngine/0.1 (educational)")
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    Ok(bytes.to_vec())
}

pub async fn fetch_wiki_summary(
    pool: Arc<PgPool>,
    ticker: String,
) -> Result<Option<WikiSummary>, String> {
    let row: Option<(String, String, Option<String>, Option<String>, Option<String>, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            "SELECT ticker, title, extract, thumbnail_url, wikipedia_url, fetched_at
             FROM wiki_summary WHERE ticker = $1",
        )
        .bind(&ticker)
        .fetch_optional(pool.as_ref())
        .await
        .ctx("fetch_wiki_summary")?;
    Ok(row.map(|(ticker, title, extract, thumbnail_url, wikipedia_url, fetched_at)| {
        WikiSummary { ticker, title, extract, thumbnail_url, wikipedia_url, fetched_at }
    }))
}

// ---------------------------------------------------------------------------
// Favorites — v11.5.B2 persistent starred tickers
// ---------------------------------------------------------------------------

pub async fn fetch_favorites(pool: Arc<PgPool>) -> Result<Vec<String>, String> {
    sqlx::query_scalar::<_, String>(
        "SELECT ticker FROM favorites ORDER BY starred_at DESC",
    )
    .fetch_all(pool.as_ref())
    .await
    .ctx("fetch_favorites")
}

/// v11.6.A — demo-favorites seed. Always ensures the 10 demo tickers are
/// present in the favorites table on every boot (per video review request:
/// "I want the tickers that were originally hardcoded to always be in the
/// favorites menu. Like always."). User-added favorites are preserved —
/// this only INSERTs missing rows via ON CONFLICT DO NOTHING. Safe to
/// call on every dashboard launch.
pub async fn seed_default_favorites_if_empty(
    pool: Arc<PgPool>,
) -> Result<Vec<String>, String> {
    const DEMO_FAVORITES: &[&str] = &[
        "AAPL", "AMZN", "GOOGL", "JPM", "META",
        "MSFT", "NVDA", "TSLA", "UNH", "V",
    ];
    for t in DEMO_FAVORITES {
        sqlx::query(
            "INSERT INTO favorites (ticker, starred_at)
             VALUES ($1, NOW())
             ON CONFLICT (ticker) DO NOTHING",
        )
        .bind(*t)
        .execute(pool.as_ref())
        .await
        .ctx("seed demo favorite")?;
    }
    fetch_favorites(pool).await
}

/// Toggles star state. Returns the new fav list after the change.
pub async fn toggle_favorite(pool: Arc<PgPool>, ticker: String) -> Result<Vec<String>, String> {
    let exists: Option<String> = sqlx::query_scalar(
        "SELECT ticker FROM favorites WHERE ticker = $1",
    )
    .bind(&ticker)
    .fetch_optional(pool.as_ref())
    .await
    .ctx("toggle_favorite check")?;

    if exists.is_some() {
        sqlx::query("DELETE FROM favorites WHERE ticker = $1")
            .bind(&ticker)
            .execute(pool.as_ref())
            .await
            .ctx("toggle_favorite delete")?;
    } else {
        sqlx::query(
            "INSERT INTO favorites (ticker, starred_at) VALUES ($1, NOW())",
        )
        .bind(&ticker)
        .execute(pool.as_ref())
        .await
        .ctx("toggle_favorite insert")?;
    }
    fetch_favorites(pool).await
}
