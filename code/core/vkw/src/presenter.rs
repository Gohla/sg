use ash::version::DeviceV1_0;
use ash::vk::{self, CommandBuffer, Extent2D, Framebuffer, Offset2D, Rect2D, Semaphore, Viewport};
use log::trace;

use crate::device::Device;
use crate::device::swapchain_extension::{AcquireNextImageError, QueuePresentError, Swapchain};
use crate::framebuffer::FramebufferCreateError;
use crate::surface_change_handler::SurfaceChangeHandler;
use crate::timeout::Timeout;

// Presenter

pub struct Presenter {
  swapchain_image_states: Box<[SwapchainImageState]>,
}

pub struct SwapchainImageState {
  pub index: u32,
  pub framebuffer: Framebuffer,
}

// Creation and destruction

impl Presenter {
  pub fn new<I: IntoIterator<Item=Framebuffer>>(framebuffers: I) -> Result<Self, FramebufferCreateError> {
    let swapchain_image_states = Self::create_swapchain_image_states(framebuffers);
    Ok(Self { swapchain_image_states })
  }

  pub unsafe fn destroy(&mut self, device: &Device) {
    trace!("Destroying presenter");
    for image_state in self.swapchain_image_states.iter() {
      device.destroy_framebuffer(image_state.framebuffer);
    }
  }

  fn create_swapchain_image_states<I: IntoIterator<Item=Framebuffer>>(framebuffers: I) -> Box<[SwapchainImageState]> {
    framebuffers.into_iter().enumerate()
      .map(|(index, framebuffer)| SwapchainImageState { index: index as u32, framebuffer })
      .collect()
  }
}

// API

impl Presenter {
  pub fn recreate<I: IntoIterator<Item=Framebuffer>>(
    &mut self,
    device: &Device,
    framebuffers: I,
  ) -> Result<(), FramebufferCreateError> {
    trace!("Recreating presenter");
    for image_state in self.swapchain_image_states.iter() {
      unsafe { device.destroy_framebuffer(image_state.framebuffer) };
    }
    self.swapchain_image_states = Self::create_swapchain_image_states(framebuffers);
    Ok(())
  }


  pub fn full_render_area(&self, extent: Extent2D) -> Rect2D {
    return Rect2D { offset: Offset2D::default(), extent };
  }

  pub unsafe fn set_dynamic_state(&self, device: &Device, command_buffer: CommandBuffer, extent: Extent2D) {
    device.cmd_set_viewport(command_buffer, 0, &[Viewport {
      x: 0.0,
      y: 0.0,
      width: extent.width as f32,
      height: extent.height as f32,
      min_depth: 0.0,
      max_depth: 1.0,
    }]);
    device.cmd_set_scissor(command_buffer, 0, &[self.full_render_area(extent)]);
  }

  pub fn acquire_image_state(
    &self,
    swapchain: &Swapchain,
    image_acquired_semaphore: Option<Semaphore>,
    surface_change_handler: &SurfaceChangeHandler,
  ) -> Result<&SwapchainImageState, AcquireNextImageError> {
    let (swapchain_image_index, suboptimal_swapchain) = unsafe { swapchain.acquire_next_image(Timeout::Infinite, image_acquired_semaphore, None)? };
    if suboptimal_swapchain {
      surface_change_handler.signal_suboptimal_swapchain();
    }
    Ok(&self.swapchain_image_states[swapchain_image_index as usize])
  }

  pub fn present(
    &self,
    device: &Device,
    swapchain: &Swapchain,
    swapchain_image_state: &SwapchainImageState,
    wait_semaphores: &[Semaphore],
    surface_change_handler: &SurfaceChangeHandler,
  ) -> Result<(), QueuePresentError> {
    let swapchains = &[swapchain.wrapped];
    let image_indices = &[swapchain_image_state.index];
    let present_info = vk::PresentInfoKHR::builder()
      .wait_semaphores(wait_semaphores)
      .swapchains(swapchains)
      .image_indices(image_indices);
    let suboptimal_swapchain = unsafe { swapchain.queue_present(device.present_queue, &present_info)? };
    if suboptimal_swapchain {
      surface_change_handler.signal_suboptimal_swapchain();
    }
    return Ok(());
  }
}
