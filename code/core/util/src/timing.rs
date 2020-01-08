use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

#[allow(deprecated)]
use time::precise_time_ns;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Debug)]
pub struct Instant(u64);

impl Instant {
  #[allow(deprecated)]
  pub fn now() -> Instant {
    Instant(precise_time_ns())
  }

  #[inline]
  pub fn to(&self, later: Instant) -> Duration {
    Duration::from_ns((later.0 - self.0) as i64)
  }
}


#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Debug)]
pub struct Duration(i64);

impl Duration {
  pub fn zero() -> Duration { Duration(0) }

  pub fn from_ns(ns: i64) -> Duration { Duration(ns) }
  pub fn from_us(us: i64) -> Duration { Duration::from_ns(us * 1_000) }
  pub fn from_ms(ms: i64) -> Duration { Duration::from_ns(ms * 1_000_000) }
  pub fn from_s(s: i64) -> Duration { Duration::from_ns(s * 1_000_000_000) }

  pub fn as_ns(&self) -> i64 { self.0 }
  pub fn as_us(&self) -> f64 { self.0 as f64 / 1_000.0 }
  pub fn as_ms(&self) -> f64 { self.0 as f64 / 1_000_000.0 }
  pub fn as_s(&self) -> f64 { self.0 as f64 / 1_000_000_000.0 }
}

impl Add for Duration {
  type Output = Duration;
  fn add(self, rhs: Duration) -> Self::Output { Duration(self.0 + rhs.0) }
}

impl AddAssign for Duration {
  fn add_assign(&mut self, rhs: Duration) { self.0 += rhs.0 }
}

impl Sub for Duration {
  type Output = Duration;
  fn sub(self, rhs: Duration) -> Self::Output { Duration(self.0 - rhs.0) }
}

impl SubAssign for Duration {
  fn sub_assign(&mut self, rhs: Duration) { self.0 -= rhs.0 }
}

impl Mul<i64> for Duration {
  type Output = Duration;
  fn mul(self, rhs: i64) -> Self::Output { Duration(self.0 * rhs) }
}

impl Div<Duration> for Duration {
  type Output = f64;
  fn div(self, rhs: Duration) -> Self::Output { self.0 as f64 / rhs.0 as f64 }
}

impl Div<u64> for Duration {
  type Output = Duration;
  fn div(self, rhs: u64) -> Self::Output { Duration(self.0 / rhs as i64) }
}

impl Div<usize> for Duration {
  type Output = Duration;
  fn div(self, rhs: usize) -> Self::Output { Duration(self.0 / rhs as i64) }
}

impl Div<i64> for Duration {
  type Output = Duration;
  fn div(self, rhs: i64) -> Self::Output { Duration(self.0 / rhs) }
}

impl Div<isize> for Duration {
  type Output = Duration;
  fn div(self, rhs: isize) -> Self::Output { Duration(self.0 / rhs as i64) }
}


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
    let elapsed = self.start.to(now);
    let delta = self.last.to(now);
    self.last = now;
    Time { elapsed, delta }
  }
}
