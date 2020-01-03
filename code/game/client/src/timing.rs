use util::{
  sampler::{
    EventSampler,
    ValueSampler,
  },
  timing::{
    Duration,
    Instant,
    Time,
    timed_ref,
    Timer,
  },
};

pub struct FrameTimer {
  timer: Timer,
  frame: u64,
}

#[derive(Copy, Clone, Debug)]
pub struct FrameTime {
  pub elapsed: Duration,
  pub frame_time: Duration,
  pub frame: u64,
}

impl FrameTimer {
  pub fn new() -> FrameTimer { FrameTimer { timer: Timer::new(), frame: 0 } }

  pub fn frame(&mut self) -> FrameTime {
    let Time { elapsed, delta: frame_time } = self.timer.update();
    let frame_time = FrameTime { elapsed, frame_time, frame: self.frame };
    self.frame += 1;
    frame_time
  }
}


pub struct TickTimer {
  tick: u64,
  start: Instant,
  time_target: Duration,
  accumulated_lag: Duration,
}

impl TickTimer {
  pub fn new(tick_time_target: Duration) -> TickTimer {
    TickTimer {
      tick: 0,
      start: Instant::now(),
      time_target: tick_time_target,
      accumulated_lag: Duration::zero(),
    }
  }


  pub fn update_lag(&mut self, frame_time: Duration) -> Duration {
    self.accumulated_lag += frame_time;
    self.accumulated_lag
  }

  pub fn num_upcoming_ticks(&self) -> u64 {
    (self.accumulated_lag / self.time_target).floor() as u64
  }

  pub fn should_tick(&self) -> bool {
    self.accumulated_lag >= self.time_target
  }

  pub fn tick_start(&mut self) -> u64 {
    self.start = Instant::now();
    self.tick
  }

  pub fn tick_end(&mut self) -> Duration {
    self.tick += 1;
    self.accumulated_lag -= self.time_target;
    self.start.to(Instant::now())
  }


  pub fn time_target(&self) -> Duration {
    self.time_target
  }

  pub fn accumulated_lag(&self) -> Duration {
    self.accumulated_lag
  }

  pub fn extrapolation(&self) -> f64 {
    let lag_ns = self.accumulated_lag.as_ns();
    let target_ns = self.time_target.as_ns();
    lag_ns as f64 / target_ns as f64
  }
}


#[derive(Default)]
pub struct TimingStats {
  // Time
  pub elapsed_time: Duration,
  // Frame
  pub frame: u64,
  pub frame_time: ValueSampler<Duration>,
  // Tick
  pub tick: u64,
  pub tick_time_target: Duration,
  pub tick_time: ValueSampler<Duration>,
  pub tick_rate: EventSampler,
  pub accumulated_lag: Duration,
  pub gfx_extrapolation: f32,
  // Detailed timing
  pub time_os_event_ns: ValueSampler<Duration>,
}

impl TimingStats {
  pub fn new() -> TimingStats { TimingStats::default() }

  pub fn frame(&mut self, elapsed: Duration, frame_time: Duration, frame: u64) {
    self.elapsed_time = elapsed;
    self.frame = frame;
    self.frame_time.add(frame_time);
  }

  pub fn tick_time_target(&mut self, tick_time_target: Duration) {
    self.tick_time_target = tick_time_target;
  }

  pub fn tick_start(&mut self, tick: u64) {
    self.tick = tick;
  }

  pub fn tick_end(&mut self, tick_time: Duration) {
    self.tick_time.add(tick_time);
    self.tick_rate.add(Instant::now())
  }

  pub fn tick_lag(&mut self, accumulated_lag: Duration, gfx_extrapolation: f32) {
    self.accumulated_lag = accumulated_lag;
    self.gfx_extrapolation = gfx_extrapolation;
  }

  #[inline]
  pub fn os_event_time<T, F: FnMut() -> T>(&mut self, func: F) -> T {
    let mut duration = Duration::zero();
    let result = timed_ref(func, &mut duration);
    self.time_os_event_ns.add(duration);
    result
  }
}