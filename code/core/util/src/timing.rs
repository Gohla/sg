use std::time::{Duration, Instant};

pub struct Timer {
  start: Instant,
  last: Instant,
}

#[derive(Copy, Clone, Debug)]
pub struct Time {
  pub elapsed: Duration,
  pub delta: Duration,
}

impl Timer {
  pub fn new() -> Timer {
    let now = Instant::now();
    return Timer { start: now, last: now };
  }

  pub fn update(&mut self) -> Time {
    let now = Instant::now();
    let elapsed = now - self.start;
    let delta = now - self.last;
    self.last = now;
    Time { elapsed, delta }
  }
}
