use std::num::NonZeroU32;

use anyhow::{Context, Result};
use raw_window_handle::HasRawWindowHandle;
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder
};
use winit::platform::desktop::EventLoopExtDesktop;
use winit::window::Window;

use gfx::Gfx;

fn main() -> Result<()> {
  simple_logger::init()
    .with_context(|| "Failed to initialize logger")?;

  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop)
    .with_context(|| "Failed to create window")?;

  let mut gfx = Gfx::new(cfg!(debug_assertions), unsafe { NonZeroU32::new_unchecked(2) }, window.raw_window_handle(), window.inner_size())
    .with_context(|| "Failed to create GFX instance")?;

  run(event_loop, window, &mut gfx)?;

  Ok(())
}

fn run(mut event_loop: EventLoop<()>, window: Window, gfx: &mut Gfx) -> Result<()> {
  Ok(event_loop.run_return(move |event, _, control_flow| {
    match event {
      Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::Resized(window_size) => gfx.surface_size_changed(window_size),
        WindowEvent::RedrawRequested => gfx.render_frame().unwrap(),
        _ => *control_flow = ControlFlow::Wait,
      },
      _ => *control_flow = ControlFlow::Wait,
    }
  }))
}
