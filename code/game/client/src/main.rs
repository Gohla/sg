use std::num::NonZeroU32;
use std::sync::mpsc::Receiver;
use std::thread;

use anyhow::{Context, Result};

use gfx::Gfx;
use math::prelude::*;
use os::context::OsContext;
use os::event_sys::{OsEvent, OsEventSys};
use os::input_sys::OsInputSys;
use os::window::Window;
use sim::legion_sim::Sim;
use util::timing::Duration;

use crate::timing::{FrameTime, FrameTimer, TickTimer};

pub mod timing;

fn main() -> Result<()> {
  simple_logger::init_with_level(log::Level::Debug)
    .with_context(|| "Failed to initialize logger")?;

  let mut os_context = OsContext::new();
  let window = {
    let window_min_size = LogicalSize::new(800.0, 600.0);
    Window::new(&os_context, window_min_size, window_min_size, "SG")
      .with_context(|| "Failed to create window")?
  };
  let (mut os_event_sys, os_input_sys, os_event_rx) = {
    let (event_sys, input_event_rx, event_rx) = OsEventSys::new(&window);
    let input_sys = OsInputSys::new(input_event_rx);
    (event_sys, input_sys, event_rx)
  };

  let gfx = Gfx::new(
    cfg!(debug_assertions),
    NonZeroU32::new(2).unwrap(),
    window.winit_raw_window_handle(),
    window.window_inner_size()
  ).with_context(|| "Failed to create GFX instance")?;

  let run_thread = thread::spawn(move || run(window, os_input_sys, os_event_rx, gfx));
  os_event_sys.run_return(&mut os_context);
  run_thread.join()
    .unwrap_or_else(|e| panic!("Run thread paniced: {:?}", e))
    .with_context(|| "Run thread stopped with an error")?;

  Ok(())
}

fn run(_window: Window, _os_input_sys: OsInputSys, os_event_rx: Receiver<OsEvent>, mut gfx: Gfx) -> Result<()> {
  let mut sim = Sim::new();

  let mut frame_timer = FrameTimer::new();
  let mut tick_timer = TickTimer::new(Duration::from_ns(16_666_667));

  'main: loop {
    // Timing
    let FrameTime { frame_time, .. } = frame_timer.frame();
    tick_timer.update_lag(frame_time);
    // Handle OS events
    for os_event in os_event_rx.try_iter() {
      match os_event {
        OsEvent::TerminateRequested => break 'main,
        OsEvent::WindowResized(screen_size) => {
          gfx.screen_size_changed(screen_size);
        },
      }
    }
    // Simulate tick
    if tick_timer.should_tick() {
      while tick_timer.should_tick() { // Run simulation.
        tick_timer.tick_start();
        sim.simulate(tick_timer.time_target());
        tick_timer.tick_end();
      }
    }
    // Render frame
    gfx.render_frame(tick_timer.extrapolation())?;
  }

  Ok(gfx.wait_idle()?)
}
