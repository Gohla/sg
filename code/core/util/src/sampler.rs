use std::collections::VecDeque;
use std::ops::Add;
use std::ops::Div;

use crate::timing::{
  Duration,
  Instant,
};

/// Sampler for figuring out the minimum, maximum, and average value.
pub struct ValueSampler<T> {
  samples: VecDeque<(Instant, T)>,
  sample_window: Duration,
  max_samples: usize,
}

impl<A: Default, T: Copy + Ord + Add<Output=T> + Div<usize, Output=A> + Default> ValueSampler<T> {
  pub fn new(sample_window: Duration, max_samples: usize) -> ValueSampler<T> {
    ValueSampler {
      samples: VecDeque::with_capacity(max_samples),
      sample_window,
      max_samples,
    }
  }


  pub fn min(&self) -> T {
    self.samples.iter().map(|&(_, s)| s).min().unwrap_or(T::default())
  }

  pub fn max(&self) -> T {
    self.samples.iter().map(|&(_, s)| s).max().unwrap_or(T::default())
  }

  pub fn avg(&self) -> A {
    let len = self.samples.len();
    if len == 0 { return A::default(); }
    let sum = self.samples.iter().map(|&(_, s)| s).fold(T::default(), |sum, sample| sum + sample);
    sum / len
  }


  pub fn add(&mut self, sample: T) {
    let now = Instant::now();
    // Remove oldest samples that are outside of the sampling window.
    loop {
      let oldest = {
        let oldest_sample = self.samples.front();
        if oldest_sample.is_none() { break; }
        let &(instant, _) = oldest_sample.unwrap();
        instant
      };
      let age = oldest.to(now);
      if age > self.sample_window {
        self.samples.pop_front();
      } else {
        break;
      }
    }
    // Remove oldest samples down to max_samples - 1, to make space for new sample.
    while self.samples.len() > self.max_samples {
      self.samples.pop_front();
    }
    self.samples.push_back((now, sample));
  }
}

impl<A: Default, T: Copy + Ord + Add<Output=T> + Div<usize, Output=A> + Default> Default for ValueSampler<T> {
  fn default() -> Self { ValueSampler::new(Duration::from_s(1), 8192) }
}


/// Sampler for figuring out how many times an event occurs.
pub struct EventSampler {
  samples: VecDeque<Instant>,
  sample_window: Duration,
  max_samples: usize,
}

impl EventSampler {
  pub fn new(sample_window: Duration, max_samples: usize) -> EventSampler {
    EventSampler {
      samples: VecDeque::with_capacity(max_samples),
      sample_window,
      max_samples,
    }
  }


  pub fn add(&mut self, instant: Instant) {
    let now = Instant::now();
    // Remove oldest samples that are outside of the sampling window.
    loop {
      let oldest: Instant = {
        let oldest = self.samples.front();
        if oldest.is_none() { break; }
        *oldest.unwrap()
      };
      let age = oldest.to(now);
      if age > self.sample_window {
        self.samples.pop_front();
      } else {
        break;
      }
    }
    // Remove oldest samples down to max_samples - 1, to make space for new sample.
    while self.samples.len() > self.max_samples {
      self.samples.pop_front();
    }
    self.samples.push_back(instant);
  }


  pub fn duration(&self) -> Option<Duration> {
    let oldest: Instant = {
      let oldest = self.samples.front();
      if oldest.is_none() { return None; };
      *oldest.unwrap()
    };
    let newest: Instant = {
      let newest = self.samples.back();
      if newest.is_none() { return None; };
      *newest.unwrap()
    };
    let duration = oldest.to(newest);
    Some(duration)
  }

  pub fn num_samples(&self) -> usize { self.samples.len() }
}

impl Default for EventSampler {
  fn default() -> Self { EventSampler::new(Duration::from_s(1), 8192) }
}
