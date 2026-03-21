//! Cross-platform runtime abstractions.
//!
//! Provides `sleep()` and `spawn()` that work on both native (tokio) and WASM (gloo/wasm-bindgen).
//! Used internally by hedged requests, streaming FC, and retry logic.

use std::future::Future;
use std::time::Duration;

/// Cross-platform async sleep.
pub async fn sleep(duration: Duration) {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(duration).await;

    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(duration.as_millis() as u32).await;
}

/// Cross-platform spawn — fire and forget.
///
/// Native: `tokio::spawn` (requires Send).
/// WASM: `wasm_bindgen_futures::spawn_local` (single-threaded, no Send needed).
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future);
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

/// Cross-platform timeout — wraps a future with a deadline.
///
/// Native: `tokio::time::timeout`.
/// WASM: no timeout (browser/runtime handles it via fetch AbortSignal).
#[cfg(not(target_arch = "wasm32"))]
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T, ()>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future).await.map_err(|_| ())
}

#[cfg(target_arch = "wasm32")]
pub async fn timeout<F, T>(_duration: Duration, future: F) -> Result<T, ()>
where
    F: Future<Output = T>,
{
    Ok(future.await)
}

/// Exponential backoff delay: 0.5s * 2^attempt, capped at 60s.
pub fn backoff_ms(attempt: u32) -> Duration {
    let secs = (0.5 * 2.0_f64.powi(attempt as i32)).min(60.0);
    Duration::from_secs_f64(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_ms_exponential() {
        assert_eq!(backoff_ms(0), Duration::from_millis(500));
        assert_eq!(backoff_ms(1), Duration::from_secs(1));
        assert_eq!(backoff_ms(2), Duration::from_secs(2));
        assert_eq!(backoff_ms(3), Duration::from_secs(4));
    }

    #[test]
    fn backoff_ms_capped_at_60s() {
        assert_eq!(backoff_ms(10), Duration::from_secs(60));
        assert_eq!(backoff_ms(20), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn sleep_completes() {
        let t0 = std::time::Instant::now();
        sleep(Duration::from_millis(50)).await;
        assert!(t0.elapsed() >= Duration::from_millis(40));
    }

    #[tokio::test]
    async fn timeout_success() {
        let result = timeout(Duration::from_secs(1), async { 42 }).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn timeout_expires() {
        let result = timeout(Duration::from_millis(10), async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            42
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn spawn_runs_to_completion() {
        let (tx, rx) = tokio::sync::oneshot::channel();
        spawn(async move {
            tx.send(42).ok();
        });
        assert_eq!(rx.await.unwrap(), 42);
    }
}
