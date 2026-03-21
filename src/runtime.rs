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
