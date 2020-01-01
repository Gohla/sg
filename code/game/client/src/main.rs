use anyhow::{Context, Result};
use raw_window_handle::HasRawWindowHandle;
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder
};
use winit::platform::desktop::EventLoopExtDesktop;
use winit::window::Window;

use gfx::{create_debug_report, create_device, create_entry, create_instance, create_surface, create_swapchain, create_swapchain_loader};

fn main() -> Result<()> {
  simple_logger::init()
    .with_context(|| "Failed to initialize logger")?;

  let entry = create_entry()
    .with_context(|| "Failed to create GFX entry")?;
  dbg!(entry.instance_version());
  let instance = create_instance(entry)
    .with_context(|| "Failed to create GFX instance")?;
  dbg!(&instance.features);
  let _debug_report = create_debug_report(&instance)
    .with_context(|| "Failed to create GFX debug report extension")?;

  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop)
    .with_context(|| "Failed to create window")?;

  let surface = create_surface(&instance, window.raw_window_handle())
    .with_context(|| "Failed to create GFX surface")?;

  let device = create_device(&instance, &surface)
    .with_context(|| "Failed to create GFX device")?;
  dbg!(&device.features);

  let swapchain_loader = create_swapchain_loader(&device);
  let swapchain = create_swapchain(&swapchain_loader, &device, &surface, window.inner_size(), None)
    .with_context(|| "Failed to create GFX swapchain")?;
  dbg!(&swapchain.features);

  run(event_loop, window)?;

  Ok(())
}

fn run(mut event_loop: EventLoop<()>, window: Window) -> Result<()> {
  Ok(event_loop.run_return(move |event, _, control_flow| {
    match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        window_id,
      } if window_id == window.id() => *control_flow = ControlFlow::Exit,
      _ => *control_flow = ControlFlow::Wait,
    }
  }))
}
