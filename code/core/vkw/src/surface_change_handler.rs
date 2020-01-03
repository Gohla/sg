use std::cell::Cell;

use ash::vk::Extent2D;

#[derive(Default)]
pub struct SurfaceChangeHandler {
  pub signal_surface_resize: Cell<Option<Extent2D>>,
  pub signal_suboptimal_swapchain: Cell<bool>,
}

impl SurfaceChangeHandler {
  pub fn new() -> Self { Self::default() }

  pub fn signal_surface_resize(&self, new_extent: Extent2D) {
    self.signal_surface_resize.set(Some(new_extent));
  }

  pub fn signal_suboptimal_swapchain(&self) {
    self.signal_suboptimal_swapchain.set(true);
  }

  pub fn query_surface_change(&self, swapchain_extent: Extent2D) -> Option<Extent2D> {
    let new_extent = self.signal_surface_resize.get();
    self.signal_surface_resize.set(None);
    let suboptimal_swapchain = self.signal_suboptimal_swapchain.get();
    self.signal_suboptimal_swapchain.set(false);
    if new_extent.is_some() || suboptimal_swapchain {
      Some(new_extent.unwrap_or(swapchain_extent))
    } else {
      None
    }
  }
}
