use std::{thread, time::Duration};

use ratelim::{RateLimiter, Timer};

#[test]
fn test_runner() {
    let mut nr_calls = 0;

    let cooldown = Duration::from_millis(50);
    let mut lim = RateLimiter::new(cooldown);
    assert_eq!(lim.cooldown_period(), cooldown);
    // not started, cold
    lim.try_run(|| nr_calls += 1).unwrap(); // hot; 50 ms to cool down
    assert_eq!(nr_calls, 1);
    thread::sleep(Duration::from_millis(10)); // 40 ms to cool down
    let wait = lim.try_run(|| unreachable!()).unwrap_err();
    thread::sleep(wait / 2); // sleep for 20 ms; 20 ms to cool down
    thread::sleep(wait / 4); // sleep for 10 ms; 10 ms to cool down
    let wait = lim.try_run(|| unreachable!()).unwrap_err();
    thread::sleep(wait);
    // cold
    lim.run(|| nr_calls += 1); // hot
    assert_eq!(nr_calls, 2);
    assert!(lim.try_run(|| unreachable!()).is_err());

    let _ = lim.clone();
}

#[test]
fn test_timer() {
    let _t = Timer::start(|elapsed| eprintln!("slept for {elapsed:?}"));
    thread::sleep(Duration::from_millis(10));
}
