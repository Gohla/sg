use ash::version::DeviceV1_0;
use ash::vk::{self, ClearValue, CommandBuffer, Framebuffer, Rect2D, RenderPass, RenderPassCreateInfo, Result as VkError};
use log::debug;
use thiserror::Error;

use crate::device::Device;

// Creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create render pass: {0:?}")]
pub struct RenderPassCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_render_pass(&self, create_info: &RenderPassCreateInfo) -> Result<RenderPass, RenderPassCreateError> {
    let render_pass = self.wrapped.create_render_pass(create_info, None)?;
    debug!("Created render pass {:?}", render_pass);
    Ok(render_pass)
  }

  pub unsafe fn destroy_render_pass(&self, render_pass: RenderPass) {
    debug!("Destroying render pass {:?}", render_pass);
    self.wrapped.destroy_render_pass(render_pass, None)
  }
}

// Beginning and ending a render pass

impl Device {
  pub unsafe fn begin_render_pass(
    &self,
    command_buffer: CommandBuffer,
    render_pass: RenderPass,
    framebuffer: Framebuffer,
    render_area: Rect2D,
    clear_values: &[ClearValue]
  ) {
    let begin_info = vk::RenderPassBeginInfo::builder()
      .render_pass(render_pass)
      .framebuffer(framebuffer)
      .render_area(render_area)
      .clear_values(clear_values)
      ;
    self.wrapped.cmd_begin_render_pass(command_buffer, &begin_info, vk::SubpassContents::INLINE);
  }

  pub unsafe fn end_render_pass(&self, command_buffer: CommandBuffer) {
    self.wrapped.cmd_end_render_pass(command_buffer)
  }
}
