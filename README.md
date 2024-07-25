# Simplistic rate limiter

Useful in cases when you don't want to repeat an operation too often.
For example, to prevent flooding the logs with the same message.

Example:

```rust
use std::{thread, time::Duration};

use ratelim::RateLimiter;
use tracing::warn;

fn main() {
    let mut lim_warn_oddities = RateLimiter::new(Duration::from_millis(10));

    let mut n = 0;
    for i in 0..1000 {
        lim_warn_oddities.run(|| {
            if i % 2 != 0 {
                warn!("{} is odd. Oh my!", i);
                n += 1;
            }
        });
        thread::sleep(Duration::from_micros(100));
    }
    assert!(0 < n && n < 10);
}
```
