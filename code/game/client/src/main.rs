use anyhow::{Context, Result};
use raw_window_handle::HasRawWindowHandle;
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder
};

use gfx::{create_debug_report, create_device, create_entry, create_instance, create_surface};

fn main() -> Result<()> {
  simple_logger::init().with_context(|| "Failed to initialize logger")?;

  let entry = create_entry().with_context(|| "Failed to create GFX entry")?;
  dbg!(entry.instance_version());
  let instance = create_instance(&entry).with_context(|| "Failed to create GFX instance")?;
  dbg!(&instance.features);
  let _debug_report = create_debug_report(&entry, &instance).with_context(|| "Failed to create GFX debug report extension")?;

  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop).with_context(|| "Failed to create window")?;

  let surface = create_surface(&entry, &instance, window.raw_window_handle()).with_context(|| "Failed to create GFX surface")?;

  let device = create_device(&instance, &surface).with_context(|| "Failed to create GFX device")?;
  dbg!(&device.features);

  event_loop.run(move |event, _, control_flow| {
    match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        window_id,
      } if window_id == window.id() => *control_flow = ControlFlow::Exit,
      _ => *control_flow = ControlFlow::Wait,
    }
  });
}
