//! Structured error types for the dashboard.
//!
//! The primary error type is `DashError`, which wraps database, parse, and
//! domain-specific errors. All variants carry pre-formatted String messages
//! so the type implements `Clone` (required by Iced's Message enum).
//!
//! The `SqlResultExt` trait provides `.ctx("function_name")` as a drop-in
//! replacement for `.map_err(|e| e.to_string())`, adding context to every
//! database error without changing function signatures.

use std::fmt;

/// Dashboard error type. All variants use String so the type is Clone-safe
/// for use in Iced's Message enum (sqlx::Error is not Clone).
#[derive(Debug, Clone)]
pub enum DashError {
    /// Database query or connection error.
    Db { context: String, detail: String },
    /// Data parsing error (dates, numbers, JSON).
    Parse(String),
    /// Expected data was not found.
    NotFound(String),
}

impl fmt::Display for DashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Db { context, detail } => write!(f, "[{context}] {detail}"),
            Self::Parse(msg) => write!(f, "parse error: {msg}"),
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
        }
    }
}

impl From<DashError> for String {
    fn from(e: DashError) -> Self {
        e.to_string()
    }
}

/// Extension trait for `Result<T, sqlx::Error>` that adds context and
/// converts to `Result<T, String>` in one step.
///
/// Replaces the repetitive `.map_err(|e| e.to_string())` pattern with
/// `.ctx("function_name")`, producing errors like `[fetch_prices] ...`.
pub trait SqlResultExt<T> {
    fn ctx(self, context: &str) -> Result<T, String>;
}

impl<T> SqlResultExt<T> for Result<T, sqlx::Error> {
    fn ctx(self, context: &str) -> Result<T, String> {
        self.map_err(|e| format!("[{context}] {e}"))
    }
}
