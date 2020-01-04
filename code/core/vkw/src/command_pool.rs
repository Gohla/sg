use ash::version::DeviceV1_0;
use ash::vk::{self, CommandBuffer, CommandPool, Result as VkError};
use log::trace;
use thiserror::Error;

use crate::command_buffer::{CommandBufferBeginError, CommandBufferEndError, CommandBufferSubmitError};
use crate::device::Device;
use crate::sync::{FenceCreateError, FenceWaitError};
use crate::timeout::Timeout;

// Creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create command pool: {0:?}")]
pub struct CommandPoolCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_command_pool(&self, transient: bool, reset_individual_buffers: bool) -> Result<CommandPool, CommandPoolCreateError> {
    use vk::CommandPoolCreateFlags;
    let flags = {
      let mut flags = CommandPoolCreateFlags::empty();
      if transient { flags |= CommandPoolCreateFlags::TRANSIENT; }
      if reset_individual_buffers { flags |= CommandPoolCreateFlags::RESET_COMMAND_BUFFER; }
      flags
    };
    let create_info = vk::CommandPoolCreateInfo::builder()
      .flags(flags)
      // TODO: don't assume that command pools are always created for the graphics queue.
      .queue_family_index(self.graphics_queue_index)
      ;
    let command_pool = self.wrapped.create_command_pool(&create_info, None)?;
    trace!("Created command pool {:?}", command_pool);
    Ok(command_pool)
  }

  pub unsafe fn destroy_command_pool(&self, command_pool: CommandPool) {
    trace!("Destroying command pool {:?}", command_pool);
    self.wrapped.destroy_command_pool(command_pool, None)
  }
}

// Reset

#[derive(Error, Debug)]
#[error("Failed to reset command pool: {0:?}")]
pub struct CommandPoolResetError(#[from] VkError);

impl Device {
  pub unsafe fn reset_command_pool(&self, command_pool: CommandPool, release_resources: bool) -> Result<(), CommandPoolResetError> {
    use vk::CommandPoolResetFlags;
    let flags = {
      let mut flags = CommandPoolResetFlags::empty();
      if release_resources { flags |= CommandPoolResetFlags::RELEASE_RESOURCES }
      flags
    };
    self.wrapped.reset_command_pool(command_pool, flags)?;
    trace!("Reset command pool {:?}", command_pool);
    Ok(())
  }
}

// Allocating/freeing command buffers

#[derive(Error, Debug)]
#[error("Failed to allocate command buffers from pool: {0:?}")]
pub struct AllocateCommandBuffersError(#[from] VkError);

impl Device {
  pub unsafe fn allocate_command_buffers(&self, command_pool: CommandPool, secondary: bool, count: u32) -> Result<Vec<CommandBuffer>, AllocateCommandBuffersError> {
    use vk::CommandBufferLevel;
    let level = if secondary { CommandBufferLevel::SECONDARY } else { CommandBufferLevel::PRIMARY };
    let create_info = vk::CommandBufferAllocateInfo::builder()
      .command_pool(command_pool)
      .level(level)
      .command_buffer_count(count)
      ;
    let command_buffers = self.wrapped.allocate_command_buffers(&create_info)?;
    trace!("Allocated command buffers from {:?}", command_buffers);
    Ok(command_buffers)
  }

  pub unsafe fn allocate_command_buffer(&self, command_pool: CommandPool, secondary: bool) -> Result<CommandBuffer, AllocateCommandBuffersError> {
    Ok(self.allocate_command_buffers(command_pool, secondary, 1)?[0])
  }

  pub unsafe fn free_command_buffers(&self, command_pool: CommandPool, command_buffers: &[CommandBuffer]) {
    trace!("Freeing command buffers {:?}", command_buffers);
    self.wrapped.free_command_buffers(command_pool, command_buffers);
  }

  pub unsafe fn free_command_buffer(&self, command_pool: CommandPool, command_buffer: CommandBuffer) {
    self.free_command_buffers(command_pool, &[command_buffer]);
  }
}

// Allocate + begin + end + submit + free

#[derive(Error, Debug)]
pub enum AllocateRecordSubmitWaitError<E: std::error::Error + 'static> {
  #[error(transparent)]
  AllocateFail(#[from] AllocateCommandBuffersError),
  #[error(transparent)]
  BeginFail(#[from] CommandBufferBeginError),
  #[error("Failed to record command buffer")]
  RecordFail(#[source] E),
  #[error(transparent)]
  EndFail(#[from] CommandBufferEndError),
  #[error(transparent)]
  FenceCreateFail(#[from] FenceCreateError),
  #[error(transparent)]
  SubmitFail(#[from] CommandBufferSubmitError),
  #[error(transparent)]
  FenceWaitFail(#[from] FenceWaitError)
}

impl Device {
  pub unsafe fn allocate_record_submit_wait<T, E: std::error::Error, F: FnOnce(CommandBuffer) -> Result<T, E>>(
    &self,
    command_pool: CommandPool,
    recorder: F
  ) -> Result<T, AllocateRecordSubmitWaitError<E>> {
    use AllocateRecordSubmitWaitError::*;
    let command_buffer = self.allocate_command_buffer(command_pool, false)?;
    self.begin_command_buffer(command_buffer, true)?;
    let result = recorder(command_buffer).map_err(|e| RecordFail(e))?;
    self.end_command_buffer(command_buffer)?;
    let fence = self.create_fence(false)?;
    self.submit_command_buffer(command_buffer, &[], &[], &[], Some(fence))?;
    self.wait_for_fence(fence, Timeout::Infinite)?;
    self.destroy_fence(fence);
    self.free_command_buffer(command_pool, command_buffer);
    Ok(result)
  }
}
