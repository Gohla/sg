use ash::version::DeviceV1_0;
use ash::vk::{self, CommandBuffer, Fence, PipelineStageFlags, Result as VkError, Semaphore};
use log::trace;
use thiserror::Error;

use crate::device::Device;

// Beginning/ending command buffers

#[derive(Error, Debug)]
#[error("Failed to begin command buffer")]
pub struct CommandBufferBeginError(#[from] VkError);

impl Device {
  pub unsafe fn begin_command_buffer(&self, command_buffer: CommandBuffer, one_time_submit: bool) -> Result<(), CommandBufferBeginError> {
    use vk::CommandBufferUsageFlags;
    let flags = {
      let mut flags = CommandBufferUsageFlags::empty();
      if one_time_submit { flags |= CommandBufferUsageFlags::ONE_TIME_SUBMIT; }
      flags
    };
    let begin_info = vk::CommandBufferBeginInfo::builder()
      .flags(flags)
      ;
    self.wrapped.begin_command_buffer(command_buffer, &begin_info)?;
    trace!("Begun recording for command buffer {:?}", command_buffer);
    Ok(())
  }
}

#[derive(Error, Debug)]
#[error("Failed to end command buffer")]
pub struct CommandBufferEndError(#[from] VkError);

impl Device {
  pub unsafe fn end_command_buffer(&self, command_buffer: CommandBuffer) -> Result<(), CommandBufferEndError> {
    trace!("Ending recording for command buffer {:?}", command_buffer);
    Ok(self.wrapped.end_command_buffer(command_buffer)?)
  }
}

// Submit

#[derive(Error, Debug)]
#[error("Failed to submit command buffer")]
pub struct CommandBufferSubmitError(#[from] VkError);

impl Device {
  pub unsafe fn submit_command_buffers(
    &self,
    command_buffers: &[CommandBuffer],
    wait_semaphores: &[Semaphore],
    wait_dst_stage_mask: &[PipelineStageFlags],
    signal_semaphores: &[Semaphore],
    fence: Fence,
  ) -> Result<(), CommandBufferSubmitError> {
    let submits = vec![vk::SubmitInfo::builder()
      .wait_semaphores(wait_semaphores)
      .wait_dst_stage_mask(wait_dst_stage_mask)
      .command_buffers(command_buffers)
      .signal_semaphores(signal_semaphores)
      .build()
    ];
    // TODO: don't assume that command pools are always submitted to the graphics queue.
    // CORRECTNESS: slices are taken by pointer but are alive until `queue_submit` is called.
    self.wrapped.queue_submit(self.graphics_queue, &submits, fence)?;
    trace!("Submitted command buffers {:?}", command_buffers);
    Ok(())
  }

  pub unsafe fn submit_command_buffer(
    &self,
    command_buffer: CommandBuffer,
    wait_semaphores: &[Semaphore],
    wait_dst_stage_mask: &[PipelineStageFlags],
    signal_semaphores: &[Semaphore],
    fence: Option<Fence>,
  ) -> Result<(), CommandBufferSubmitError> {
    self.submit_command_buffers(&[command_buffer], wait_semaphores, wait_dst_stage_mask, signal_semaphores, fence.unwrap_or_default())
  }
}
