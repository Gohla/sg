use ash::vk::Extent2D;
use log::debug;

#[derive(Default)]
pub struct SurfaceChangeHandler {
  pub signal_screen_resize: Option<Extent2D>,
  pub signal_suboptimal_swapchain: bool,
}

impl SurfaceChangeHandler {
  pub fn new() -> Self { Self::default() }

  pub fn signal_screen_resize(&mut self, new_extent: Extent2D) {
    debug!("Signalled surface resize to {:?}", new_extent);
    self.signal_screen_resize = Some(new_extent);
  }

  pub fn signal_suboptimal_swapchain(&mut self) {
    debug!("Signalled suboptimal swapchain");
    self.signal_suboptimal_swapchain = true;
  }

  pub fn query_surface_change(&mut self, swapchain_extent: Extent2D) -> Option<Extent2D> {
    let new_extent = self.signal_screen_resize;
    self.signal_screen_resize = None;
    let suboptimal_swapchain = self.signal_suboptimal_swapchain;
    self.signal_suboptimal_swapchain = false;
    if new_extent.is_some() || suboptimal_swapchain {
      Some(new_extent.unwrap_or(swapchain_extent))
    } else {
      None
    }
  }
}
