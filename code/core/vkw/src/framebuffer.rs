use ash::version::DeviceV1_0;
use ash::vk::{Framebuffer, FramebufferCreateInfo, Result as VkError};
use thiserror::Error;

use crate::device::Device;

// Creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create framebuffer")]
pub struct FramebufferCreateError(#[from] VkError);

impl Device {
  pub fn create_framebuffer(&self, create_info: &FramebufferCreateInfo) -> Result<Framebuffer, FramebufferCreateError> {
    Ok(unsafe { self.wrapped.create_framebuffer(create_info, None) }?)
  }

  pub unsafe fn destroy_framebuffer(&self, framebuffer: Framebuffer) {
    self.wrapped.destroy_framebuffer(framebuffer, None)
  }
}
