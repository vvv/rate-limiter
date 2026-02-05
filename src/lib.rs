//! Simplistic rate limiter.
//!
//! # Examples
//!
//! ```
//! # use ratelim::RateLimiter;
//! # use std::{thread, time::Duration};
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

    /// Runs the function if the cooldown period has elapsed,
    /// passing the elapsed time since the last run.
    ///
    /// The first call [starts] the `RateLimiter` without running the function.
    ///
    /// [starts]: Self::start_now
    pub fn run_dt(&mut self, f: impl FnOnce(Duration)) {
        let Some(start) = self.start else {
            self.start_now();
            return;
        };
        let now = Instant::now();
        let elapsed = now - start;
        if elapsed >= self.cooldown {
            f(elapsed);
            self.start.replace(now);
        }
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

/// A timer that calls a function on drop with the elapsed time.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Timer<F: FnMut(Duration)> {
    started: Instant,
    on_drop: F,
}

impl<F: FnMut(Duration)> Timer<F> {
    /// Starts the timer, specifying the function to call on drop.
    ///
    /// Example:
    ///
    /// ```
    /// use ratelim::Timer;
    /// use std::{thread, time::Duration};
    ///
    /// {
    ///     let _t = Timer::start(|elapsed| eprintln!("slept for {elapsed:?}"));
    ///     thread::sleep(Duration::from_millis(10));
    /// }
    /// ```
    pub fn start(on_drop: F) -> Self {
        Self {
            started: Instant::now(),
            on_drop,
        }
    }
}

impl<F: FnMut(Duration)> Drop for Timer<F> {
    fn drop(&mut self) {
        (self.on_drop)(self.started.elapsed());
    }
}
