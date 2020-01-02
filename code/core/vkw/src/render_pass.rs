use ash::version::DeviceV1_0;
use ash::vk::{RenderPass, RenderPassCreateInfo, Result as VkError};
use thiserror::Error;

use crate::device::Device;

// Creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create render pass")]
pub struct RenderPassCreateError(#[from] VkError);

impl Device {
  pub fn create_render_pass(&self, create_info: &RenderPassCreateInfo) -> Result<RenderPass, RenderPassCreateError> {
    Ok(unsafe { self.wrapped.create_render_pass(create_info, None) }?)
  }

  pub unsafe fn destroy_render_pass(&self, render_pass: RenderPass) {
    self.wrapped.destroy_render_pass(render_pass, None)
  }
}
