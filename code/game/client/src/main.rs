use anyhow::{Context, Result};
use raw_window_handle::HasRawWindowHandle;
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder
};
use winit::platform::desktop::EventLoopExtDesktop;
use winit::window::Window;
use vkw::prelude::*;

use gfx::{GfxDevice};

fn main() -> Result<()> {
  simple_logger::init()
    .with_context(|| "Failed to initialize logger")?;

  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop)
    .with_context(|| "Failed to create window")?;

  let mut gfx_device = GfxDevice::new(cfg!(debug_assertions), window.raw_window_handle(), window.inner_size())
    .with_context(|| "Failed to create GFX instance")?;

  run(event_loop, window, &mut gfx_device)?;

  Ok(())
}

fn run(mut event_loop: EventLoop<()>, window: Window, gfx_device: &mut GfxDevice) -> Result<()> {
  Ok(event_loop.run_return(move |event, _, control_flow| {
    match event {
      Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::Resized(window_size) => {
          let (width, height) = window_size.into();
          unsafe { gfx_device.swapchain.recreate(&gfx_device.device, &gfx_device.surface, Extent2D { width, height }) }
            .with_context(|| "Failed to recreate GFX swapchain").unwrap();
        }
        _ => *control_flow = ControlFlow::Wait,
      },
      _ => *control_flow = ControlFlow::Wait,
    }
  }))
}
