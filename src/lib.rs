//! Simplistic rate limiter.
//!
//! # Examples
//!
//! ```
//! # use std::{thread, time::Duration};
//! # use ratelim::RateLimiter;
//! #
//! # fn main() {
//! // We don't want to overwater plants. Twice a second should be fine?
//! let mut lim_water_plants = RateLimiter::new(Duration::from_millis(500));
//!
//! let mut n = 0;
//! for _ in 0..5 {
//!     lim_water_plants.run(|| {
//!         println!("Watering plants... 🌱🔫");
//!         n += 1;
//!     });
//!     thread::sleep(Duration::from_millis(200));
//! }
//! assert_eq!(n, 2);
//! # }
//! ```

use std::time::{Duration, Instant};

/// Allows to [run] the operation at most once per the cooldown period.
///
/// [run]: RateLimiter::run
#[derive(Debug, Clone)]
pub struct RateLimiter {
    cooldown: Duration,
    start: Option<Instant>,
}

impl RateLimiter {
    /// Creates a rate limiter with the given cooldown period.
    ///
    /// # Panics
    ///
    /// Panics if `cooldown.is_zero()`.
    pub fn new(cooldown: Duration) -> Self {
        assert!(!cooldown.is_zero());
        Self {
            cooldown,
            start: None,
        }
    }

    /// Returns the cooldown period.
    pub fn cooldown_period(&self) -> Duration {
        self.cooldown
    }

    /// (Re)starts the cooldown period.
    /// Returns the previous start time if any.
    pub fn start_now(&mut self) -> Option<Instant> {
        self.start.replace(Instant::now())
    }

    /// Runs the function if the cooldown period has elapsed.
    ///
    /// The first call succeeds immediately, starting the `RateLimiter`.
    pub fn run(&mut self, f: impl FnOnce()) {
        self.try_run(f).ok();
    }

    /// Runs the function if the cooldown period has elapsed.
    /// Otherwise errs with the time remaining.
    ///
    /// The first call succeeds immediately, starting the `RateLimiter`.
    pub fn try_run(&mut self, f: impl FnOnce()) -> Result<(), Duration> {
        let Some(start) = self.start else {
            f();
            self.start_now();
            return Ok(());
        };

        let t_cold = start + self.cooldown;
        let now = Instant::now();
        if now < t_cold {
            //
            //   |<------ cooldown_period ----->|
            // --+---------------+--------------+---------------> time
            //   |<-- elapsed -->|<--- wait --->|
            //   |               |              |
            //   start           now            t_cold
            //
            Err(t_cold - now)
        } else {
            //
            //   |<----------------- elapsed ------------------->|
            //   |<------ cooldown_period ----->|<-- overshot -->|
            // --+------------------------------+----------------+----> time
            //   |                              |                |
            //   start                          t_cold           now
            //
            f();
            self.start.replace(now);
            Ok(())
        }
    }
}
