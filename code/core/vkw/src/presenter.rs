use std::cell::Cell;

use ash::version::DeviceV1_0;
use ash::vk::{self, CommandBuffer, Extent2D, Framebuffer, Offset2D, Rect2D, RenderPass, Semaphore, Viewport};

use crate::device::Device;
use crate::device::swapchain_extension::{AcquireNextImageError, QueuePresentError, Swapchain};
use crate::instance::surface_extension::Surface;
use crate::timeout::Timeout;

// Presenter

pub struct Presenter {
  pub swapchain: Swapchain,
  pub swapchain_image_states: Box<[SwapchainImageState]>,
  pub create_framebuffers_fn: CreateFramebuffersFn,
  pub signal_surface_resize: Cell<Option<Extent2D>>,
  pub signal_suboptimal_swapchain: Cell<bool>,
}

pub struct SwapchainImageState {
  pub index: u32,
  pub framebuffer: Framebuffer,
}

pub type CreateFramebuffersFn = Box<dyn Fn(&Swapchain, &RenderPass) -> Result<Vec<Framebuffer>, Box<dyn std::error::Error>>>;

// Creation

impl Presenter {
  pub fn new(
    swapchain: Swapchain,
    render_pass: &RenderPass,
    create_framebuffers_fn: CreateFramebuffersFn,
  ) -> Result<Self, Box<dyn std::error::Error>> {
    let framebuffers = create_framebuffers_fn(&swapchain, render_pass)?;
    let swapchain_image_states = Self::create_swapchain_image_states(framebuffers);
    Ok(Self {
      swapchain,
      swapchain_image_states,
      create_framebuffers_fn,
      signal_surface_resize: Cell::new(None),
      signal_suboptimal_swapchain: Cell::new(false)
    })
  }

  fn create_swapchain_image_states(framebuffers: Vec<Framebuffer>) -> Box<[SwapchainImageState]> {
    let mut vec = Vec::with_capacity(framebuffers.len());
    let mut index = 0;
    for framebuffer in framebuffers {
      vec.push(SwapchainImageState { index, framebuffer });
      index += 1;
    }
    vec.into_boxed_slice()
  }
}

// API

impl Presenter {
  pub fn should_recreate(&self) -> bool {
    return !self.signal_surface_resize.get().is_none() || self.signal_suboptimal_swapchain.get();
  }

  pub fn signal_surface_resize(&self, new_extent: Extent2D) {
    self.signal_surface_resize.set(Some(new_extent));
  }

  pub fn signal_suboptimal_swapchain(&self) {
    self.signal_suboptimal_swapchain.set(true);
  }

  pub fn recreate(
    &mut self,
    device: &Device,
    surface: &Surface,
    render_pass: &RenderPass,
  ) -> Result<(), Box<dyn std::error::Error>> {
    if !self.should_recreate() {
      return Ok(());
    }
    let new_extent = self.signal_surface_resize.get().unwrap_or(self.swapchain.extent);
    unsafe { self.swapchain.recreate(device, surface, new_extent) }?;
    let framebuffers = (self.create_framebuffers_fn)(&self.swapchain, render_pass)?;
    self.swapchain_image_states = Self::create_swapchain_image_states(framebuffers);
    self.signal_surface_resize.set(None);
    self.signal_suboptimal_swapchain.set(false);
    Ok(())
  }


  pub fn full_render_area(&self) -> Rect2D {
    return Rect2D { offset: Offset2D::default(), extent: self.swapchain.extent };
  }

  pub unsafe fn set_dynamic_state(&self, device: &Device, command_buffer: CommandBuffer) {
    device.cmd_set_viewport(command_buffer, 0, &[Viewport {
      x: 0.0,
      y: 0.0,
      width: self.swapchain.extent.width as f32,
      height: self.swapchain.extent.height as f32,
      min_depth: 0.0,
      max_depth: 1.0,
    }]);
    device.cmd_set_scissor(command_buffer, 0, &[self.full_render_area()]);
  }

  pub fn acquire_image_state(&self, image_acquired_semaphore: Option<Semaphore>) -> Result<&SwapchainImageState, AcquireNextImageError> {
    let (swapchain_image_index, suboptimal_swapchain) = unsafe { self.swapchain.acquire_next_image(Timeout::Infinite, image_acquired_semaphore, None)? };
    if suboptimal_swapchain {
      self.signal_suboptimal_swapchain();
    }
    Ok(&self.swapchain_image_states[swapchain_image_index as usize])
  }

  pub fn present(&self, device: &Device, swapchain_image_state: &SwapchainImageState, wait_semaphores: &[Semaphore]) -> Result<(), QueuePresentError> {
    let swapchains = &[self.swapchain.wrapped];
    let image_indices = &[swapchain_image_state.index];
    let present_info = vk::PresentInfoKHR::builder()
      .wait_semaphores(wait_semaphores)
      .swapchains(swapchains)
      .image_indices(image_indices);
    let suboptimal_swapchain = unsafe { self.swapchain.queue_present(device.present_queue, &present_info)? };
    if suboptimal_swapchain {
      self.signal_suboptimal_swapchain();
    }
    return Ok(());
  }
}
