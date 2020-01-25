use std::num::NonZeroU32;
use std::sync::mpsc::Receiver;
use std::thread;

use anyhow::{Context, Result};
use winit::event::VirtualKeyCode;

use gfx::camera::CameraInput;
use gfx::Gfx;
use gfx::grid_renderer::GridTileRender;
use gfx::texture_def::TextureDefBuilder;
use math::prelude::*;
use os::context::OsContext;
use os::event_sys::{OsEvent, OsEventSys};
use os::input_sys::{OsInputSys, RawInput};
use os::window::Window;
use sim::{WorldTransform, WorldDynamics, InGrid, GridPosition, GridOrientation};
use sim::legion_sim::Sim;
use util::image::{Components, ImageData};
use util::timing::Duration;

use crate::timing::{FrameTime, FrameTimer, TickTimer};

pub mod timing;

fn main() -> Result<()> {
  // Initialize logger.
  simple_logger::init_with_level(log::Level::Debug)
    .with_context(|| "Failed to initialize logger")?;
  // OS context, window, and event handling.
  let mut os_context = OsContext::new();
  let window = {
    let window_min_size = LogicalSize::new(800.0, 600.0);
    Window::new(&os_context, window_min_size, window_min_size, "SG")
      .with_context(|| "Failed to create window")?
  };
  let (mut os_event_sys, os_event_rx, os_input_sys) = {
    let (event_sys, input_event_rx, event_rx) = OsEventSys::new(&window);
    let input_sys = OsInputSys::new(input_event_rx);
    (event_sys, event_rx, input_sys)
  };
  // Initialize simulation.
  let mut sim = Sim::new();
  // Initialize game.
  let texture_def_builder = init(&mut sim)
    .with_context(|| "Failed to initialize game")?;
  // Initialize graphics.
  let gfx = Gfx::new(
    cfg!(debug_assertions),
    NonZeroU32::new(2).unwrap(),
    window.winit_raw_window_handle(),
    window.window_inner_size(),
    texture_def_builder,
  ).with_context(|| "Failed to create GFX instance")?;
  // Spawn game thread and run OS event loop.
  let game_thread = thread::spawn(move || run(window, os_event_rx, os_input_sys, sim, gfx));
  os_event_sys.run_return(&mut os_context);
  game_thread.join()
    .unwrap_or_else(|e| panic!("Game thread paniced: {:?}", e))
    .with_context(|| "Game thread stopped with an error")?;
  Ok(())
}

fn init(sim: &mut Sim) -> Result<TextureDefBuilder> {
  let mut texture_def_builder = TextureDefBuilder::new();
  let tex1 = texture_def_builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/dark.png"), Some(Components::Components4))?);
  let tex2 = texture_def_builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/light.png"), Some(Components::Components4))?);
  let tex3 = texture_def_builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/green.png"), Some(Components::Components4))?);

  let world = &mut sim.world;
  let grid = world.insert((), vec![
    (WorldTransform::new(0.0, 0.0, 0.0), WorldDynamics::new(0.0, 0.0, 0.0)),
  ])[0];

  world.insert((InGrid::new(grid), ), vec![
    (GridPosition::new(0, 0), GridOrientation::default(), GridTileRender(tex1)),
    //(GridPosition::new(-1, 0), GridOrientation::default(), GridTileRender(tex1)),
    //(GridPosition::new(0, -1), GridOrientation::default(), GridTileRender(tex1)),
    //(GridPosition::new(-1, -1), GridOrientation::default(), GridTileRender(tex1)),
    //(GridPosition::new(0, 7), GridOrientation::default(), GridTileRender(tex2)),
    //(GridPosition::new(0, 8), GridOrientation::default(), GridTileRender(tex3)),
  ]);

  Ok(texture_def_builder)
}

fn run(_window: Window, os_event_rx: Receiver<OsEvent>, mut os_input_sys: OsInputSys, mut sim: Sim, mut gfx: Gfx) -> Result<()> {
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
    let camera_input = raw_input_to_camera_input(raw_input);
    // Simulate tick
    if tick_timer.should_tick() {
      while tick_timer.should_tick() { // Run simulation.
        tick_timer.tick_start();
        sim.simulate(tick_timer.time_target());
        tick_timer.tick_end();
      }
    }
    // Render frame
    gfx.render_frame(&mut sim.world, camera_input, tick_timer.extrapolation(), frame_time)?;
  }

  Ok(gfx.wait_idle()?)
}

fn raw_input_to_camera_input(input: RawInput) -> CameraInput {
  CameraInput {
    move_up: input.is_key_down(VirtualKeyCode::W),
    move_right: input.is_key_down(VirtualKeyCode::D),
    move_down: input.is_key_down(VirtualKeyCode::S),
    move_left: input.is_key_down(VirtualKeyCode::A),
    zoom_delta: input.mouse_wheel_delta.y as f32,
    drag: input.mouse_buttons.right,
    drag_pos: input.mouse_pos,
  }
}
