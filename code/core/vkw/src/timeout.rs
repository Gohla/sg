use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Timeout {
  None,
  Some(Duration),
  Infinite,
}

impl Into<u64> for Timeout {
  fn into(self) -> u64 {
    match self {
      Timeout::None => 0,
      Timeout::Some(ref d) => 1_000_000_000u64 * d.as_secs() + u64::from(d.subsec_nanos()),
      Timeout::Infinite => u64::max_value(),
    }
  }
}
