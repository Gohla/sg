use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use thiserror::Error;
use winit::error::OsError;
use winit::window::{Window as WinitWindow, WindowBuilder, WindowId};

use math::screen::{LogicalSize, PhysicalSize, Scale, ScreenSize};

use crate::context::OsContext;
use crate::screen_ext::*;

pub struct Window {
  window: WinitWindow,
}

#[derive(Debug, Error)]
#[error("Could not create Window")]
pub struct WindowCreateError(#[from] OsError);

impl Window {
  pub fn new<S: Into<String>>(
    os_context: &OsContext,
    inner_size: LogicalSize,
    min_inner_size: LogicalSize,
    title: S,
  ) -> Result<Self, WindowCreateError> {
    let window = WindowBuilder::new()
      .with_inner_size(inner_size.into_winit())
      .with_min_inner_size(min_inner_size.into_winit())
      .with_title(title)
      .build(&os_context.event_loop)?;
    Ok(Self { window })
  }


  pub fn window_scale_factor(&self) -> Scale {
    self.window.scale_factor().into()
  }

  pub fn window_inner_physical_size(&self) -> PhysicalSize {
    self.window.inner_size().into_util()
  }

  pub fn window_inner_size(&self) -> ScreenSize {
    let physical_size: (u32, u32) = self.window.inner_size().into();
    let scale = self.window.scale_factor();
    ScreenSize::from_physical_scale(physical_size, scale)
  }


  pub fn winit_window(&self) -> &WinitWindow {
    &self.window
  }

  pub fn winit_window_id(&self) -> WindowId {
    self.window.id()
  }

  pub fn winit_raw_window_handle(&self) -> RawWindowHandle {
    self.window.raw_window_handle()
  }
}
