/// Exponential restart backoff with deterministic jitter and a cap (TC14).
/// Jitter is deterministic on purpose: tests can assert exact bounds, and two
/// runs with the same attempt count behave the same way.
#[derive(Debug, Clone, Copy)]
pub struct Backoff {
    pub start_seconds: f64,
    pub max_seconds: f64,
}

impl Backoff {
    pub fn new(start_seconds: f64, max_seconds: f64) -> Self {
        Self {
            start_seconds: start_seconds.max(0.0),
            max_seconds: max_seconds.max(start_seconds),
        }
    }

    fn jitter_factor(attempt: u32) -> f64 {
        // Simple hash style spread in the range 0.85 to 1.15.
        let hash = (attempt as u64).wrapping_mul(2_654_435_761) % 100;
        0.85 + (hash as f64 / 100.0) * 0.30
    }

    pub fn base_delay_seconds(&self, attempt: u32) -> f64 {
        let grown = self.start_seconds * 2f64.powi(attempt as i32);
        grown.min(self.max_seconds)
    }

    pub fn delay_seconds(&self, attempt: u32) -> f64 {
        (self.base_delay_seconds(attempt) * Self::jitter_factor(attempt)).min(self.max_seconds)
    }
}

/// Tracks consecutive failures for one agent and resets after a stable run.
#[derive(Debug, Clone)]
pub struct RestartTracker {
    pub backoff: Backoff,
    pub stable_after_seconds: u64,
    failures: u32,
}

impl RestartTracker {
    pub fn new(backoff: Backoff, stable_after_seconds: u64) -> Self {
        Self {
            backoff,
            stable_after_seconds,
            failures: 0,
        }
    }

    pub fn failures(&self) -> u32 {
        self.failures
    }

    /// Record a failure and return the delay before the next restart.
    pub fn record_failure(&mut self) -> f64 {
        let delay = self.backoff.delay_seconds(self.failures);
        self.failures += 1;
        delay
    }

    /// Record how long the agent ran. A run at least as long as the stable
    /// period resets the failure counter (spec section 6.3).
    pub fn record_run_duration(&mut self, ran_for_seconds: u64) {
        if ran_for_seconds >= self.stable_after_seconds {
            self.failures = 0;
        }
    }

    pub fn has_exceeded(&self, max_restarts: u32) -> bool {
        self.failures > max_restarts
    }
}
