//! Shared HTTP retry helper with exponential backoff (v11.3).
//!
//! Use `with_retry` to wrap any async fallible operation in 3 attempts with
//! 2s/8s/32s backoff. Designed to absorb transient network/API failures
//! without burying the underlying error when retries are exhausted.

use std::future::Future;
use std::time::Duration;

/// Run `op` up to 3 times. Backs off 2s, 8s, then propagates the final error.
///
/// `label` is used in the retry log line so you can see which call retried.
pub async fn with_retry<T, E, F, Fut>(label: &str, mut op: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let backoffs = [Duration::from_secs(2), Duration::from_secs(8)];
    for (attempt, sleep) in backoffs.iter().enumerate() {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                eprintln!("[retry] {label} attempt {} failed: {e} — sleeping {:?}",
                    attempt + 1, sleep);
                tokio::time::sleep(*sleep).await;
            }
        }
    }
    op().await
}
