use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

/// A simple, smooth rate limiter based on tracking the next allowed event time.
/// Distributes bandwidth evenly across concurrent tasks.
pub struct SpeedLimiter {
    limit_bps: AtomicUsize,
    next_allowed: Mutex<Instant>,
}

impl SpeedLimiter {
    pub fn new(initial_limit_bps: usize) -> Self {
        Self {
            limit_bps: AtomicUsize::new(initial_limit_bps),
            next_allowed: Mutex::new(Instant::now()),
        }
    }

    /// Update the speed limit dynamically. 0 means unlimited.
    pub fn set_limit(&self, bps: usize) {
        self.limit_bps.store(bps, Ordering::Relaxed);
        // Reset the allowed time to now to prevent long locks if the limit drops drastically
        if let Ok(mut g) = self.next_allowed.try_lock() {
            *g = Instant::now();
        }
    }

    /// Asynchronously waits until capacity is available for `bytes`.
    pub async fn wait(&self, bytes: usize) {
        let bps = self.limit_bps.load(Ordering::Relaxed);
        if bps == 0 {
            return; // 0 = Unlimited
        }

        // Calculate how much time this chunk of bytes "costs"
        let duration_for_bytes = Duration::from_secs_f64(bytes as f64 / bps as f64);

        let mut next = self.next_allowed.lock().await;
        let now = Instant::now();

        // If the allowed time is in the past, bring it to the present
        // (we haven't utilized our capacity recently)
        if *next < now {
            *next = now;
        }

        // Add the cost of these bytes to the next allowed time
        *next += duration_for_bytes;

        // Compute sleep duration *before* dropping the lock
        let sleep_duration = next.saturating_duration_since(now);

        // Critical: drop the lock so other tasks can calculate their sleep times!
        drop(next);

        // Sleep to respect the limit, only if duration is meaningful (> 1ms)
        if sleep_duration > Duration::from_millis(1) {
            sleep(sleep_duration).await;
        }
    }
}
