use ash::version::DeviceV1_0;
use ash::vk::{Framebuffer, FramebufferCreateInfo, Result as VkError};
use log::debug;
use thiserror::Error;

use crate::device::Device;

// Creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create framebuffer")]
pub struct FramebufferCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_framebuffer(&self, create_info: &FramebufferCreateInfo) -> Result<Framebuffer, FramebufferCreateError> {
    let framebuffer = self.wrapped.create_framebuffer(create_info, None)?;
    debug!("Created framebuffer {:?}", framebuffer);
    Ok(framebuffer)
  }

  pub unsafe fn destroy_framebuffer(&self, framebuffer: Framebuffer) {
    debug!("Destroying framebuffer {:?}", framebuffer);
    self.wrapped.destroy_framebuffer(framebuffer, None)
  }
}
