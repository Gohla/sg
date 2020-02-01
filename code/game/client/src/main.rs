use std::num::NonZeroU32;
use std::sync::mpsc::Receiver;
use std::thread;

use anyhow::{Context, Result};
use log::debug;

use gfx::Gfx;
use math::prelude::*;
use os::context::OsContext;
use os::event_sys::{OsEvent, OsEventSys};
use os::input_sys::{OsInputSys};
use os::window::Window;
use sim::prelude::*;
use util::timing::Duration;

use crate::game::{Game, GameDef};
use crate::input::Input;
use crate::timing::{FrameTime, FrameTimer, TickTimer};

pub mod timing;
pub mod input;
pub mod game;

fn main() -> Result<()> {
  // Initialize logger.
  simple_logger::init_with_level(log::Level::Debug)
    .with_context(|| "Failed to initialize logger")?;
  // OS context, window, and event handling.
  let mut os_context = OsContext::new();
  let window = {
    let window_min_size = LogicalSize::new(1920.0, 1080.0);
    Window::new(&os_context, window_min_size, window_min_size, "SG")
      .with_context(|| "Failed to create window")?
  };
  let (mut os_event_sys, os_event_rx, os_input_sys) = {
    let (event_sys, input_event_rx, event_rx) = OsEventSys::new(&window);
    let input_sys = OsInputSys::new(input_event_rx);
    (event_sys, event_rx, input_sys)
  };

  // Initialize game definition.
  let (game_def, texture_def_builder) = GameDef::new()
    .with_context(|| "Failed to initialize game definition")?;

  // Initialize simulation.
  let mut sim = Sim::new();
  // Initialize graphics.
  let mut gfx = Gfx::new(
    cfg!(debug_assertions),
    NonZeroU32::new(2).unwrap(),
    window.winit_raw_window_handle(),
    window.window_inner_size(),
    texture_def_builder,
  ).with_context(|| "Failed to create GFX instance")?;

  // Initialize game.
  let game = Game::new(game_def, &mut sim, &mut gfx);

  // Spawn game thread and run OS event loop.
  let game_thread = thread::Builder::new()
    .name("Game".to_string())
    .spawn(move || {
      debug!("Game thread started");
      run(window, os_event_rx, os_input_sys, sim, gfx, game)
        .with_context(|| "Game thread stopped with an error").unwrap();
      debug!("Game thread stopped");
    })
    .with_context(|| "Failed to create game thread")?;
  debug!("Main thread OS-event loop started");
  os_event_sys.run_return(&mut os_context);
  debug!("Main thread OS-event loop stopped");
  game_thread.join()
    .unwrap_or_else(|e| panic!("Game thread paniced: {:?}", e));
  Ok(())
}

fn run(_window: Window, os_event_rx: Receiver<OsEvent>, mut os_input_sys: OsInputSys, mut sim: Sim, mut gfx: Gfx, mut game: Game) -> Result<()> {
  let mut frame_timer = FrameTimer::new();
  let mut tick_timer = TickTimer::new(Duration::from_ns(16_666_667));
  'main: loop {
    // Timing
    let FrameTime { frame_time, .. } = frame_timer.frame();
    tick_timer.update_lag(frame_time);
    // Process OS events
    for os_event in os_event_rx.try_iter() {
      match os_event {
        OsEvent::TerminateRequested => break 'main,
        OsEvent::WindowResized(screen_size) => {
          gfx.screen_size_changed(screen_size);
        },
      }
    }
    // Process input
    let raw_input = os_input_sys.update();
    let Input { game: game_input, camera: camera_input } = Input::from_raw(raw_input);
    // Simulate tick
    if tick_timer.should_tick() {
      while tick_timer.should_tick() { // Run simulation.
        tick_timer.tick_start();
        game.simulate_tick(game_input, &mut sim, &mut gfx);
        sim.simulate_tick(tick_timer.time_target());
        tick_timer.tick_end();
      }
    }
    // Render frame
    gfx.render_frame(&mut sim.world, camera_input, tick_timer.extrapolation(), frame_time)?;
  }

  Ok(gfx.wait_idle()?)
}
