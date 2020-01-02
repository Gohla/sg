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

use gfx::{GfxInstance, create_device, create_swapchain, create_swapchain_loader};

fn main() -> Result<()> {
  simple_logger::init()
    .with_context(|| "Failed to initialize logger")?;

  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop)
    .with_context(|| "Failed to create window")?;

  let gfx_instance = GfxInstance::new(cfg!(debug_assertions), window.raw_window_handle())
    .with_context(|| "Failed to create GFX instance")?;

  let device = create_device(&gfx_instance.instance, &gfx_instance.surface)
    .with_context(|| "Failed to create GFX device")?;
  dbg!(&device.features);

  let swapchain_loader = create_swapchain_loader(&device);
  let mut swapchain = create_swapchain(&swapchain_loader, &device, &gfx_instance.surface, window.inner_size())
    .with_context(|| "Failed to create GFX swapchain")?;
  dbg!(&swapchain.features);

  run(event_loop, window, &mut swapchain)?;

  Ok(())
}

fn run(mut event_loop: EventLoop<()>, window: Window, swapchain: &mut Swapchain) -> Result<()> {
  Ok(event_loop.run_return(move |event, _, control_flow| {
    match event {
      Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::Resized(window_size) => {
          let (width, height) = window_size.into();
          swapchain.recreate(Extent2D { width, height })
            .with_context(|| "Failed to recreate GFX swapchain").unwrap();
        }
        _ => *control_flow = ControlFlow::Wait,
      },
      _ => *control_flow = ControlFlow::Wait,
    }
  }))
}
