use std::num::NonZeroU32;
use std::sync::mpsc;
use std::thread;

use anyhow::{Context, Result};
use raw_window_handle::HasRawWindowHandle;
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder
};
use winit::platform::desktop::EventLoopExtDesktop;

use gfx::Gfx;
use sim::legion_sim::Sim;
use util::timing::Duration;

use crate::timing::{FrameTime, FrameTimer, TickTimer};

pub mod timing;

fn main() -> Result<()> {
  simple_logger::init_with_level(log::Level::Debug)
    .with_context(|| "Failed to initialize logger")?;

  let mut sim = Sim::new();

  let mut event_loop = EventLoop::new();
  let window_min_size = (800, 600);
  let window = WindowBuilder::new()
    .with_inner_size(window_min_size.into())
    .with_min_inner_size(window_min_size.into())
    .with_title("SG")
    .build(&event_loop)
    .with_context(|| "Failed to create window")?;

  let mut gfx = Gfx::new(
    cfg!(debug_assertions),
    unsafe { NonZeroU32::new_unchecked(2) },
    window.raw_window_handle(),
    window.inner_size()
  ).with_context(|| "Failed to create GFX instance")?;

  let (tx, rx) = mpsc::channel();
  let thread = thread::spawn(move || {
    let mut frame_timer = FrameTimer::new();
    let mut tick_timer = TickTimer::new(Duration::from_ns(16_666_667));

    'main: loop {
      let FrameTime { frame_time, .. } = frame_timer.frame();
      tick_timer.update_lag(frame_time);

      // Handle OS events.
      for window_event in rx.try_iter() {
        match window_event {
          WindowEvent::CloseRequested => break 'main,
          WindowEvent::Resized(window_size) => gfx.surface_size_changed(window_size),
          _ => {}
        }
      }

      if tick_timer.should_tick() {
        while tick_timer.should_tick() { // Run simulation.
          tick_timer.tick_start();
          sim.simulate(tick_timer.time_target());
          tick_timer.tick_end();
        }
      }

      gfx.render_frame(tick_timer.extrapolation()).unwrap();
    }
    gfx.wait_idle().unwrap();
  });

  event_loop.run_return(move |event, _, control_flow| {
    match event {
      Event::WindowEvent { event, window_id } if window_id == window.id() => {
        match event {
          WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
          _ => *control_flow = ControlFlow::Wait,
        }
        tx.send(event).ok(); // Ignore failure: receiver was destroyed already, but that means we are closing.
      }
      _ => *control_flow = ControlFlow::Wait,
    }
  });

  thread.join().unwrap();

  Ok(())
}
