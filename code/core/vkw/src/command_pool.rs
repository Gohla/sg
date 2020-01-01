use ash::version::DeviceV1_0;
use ash::vk::{self, CommandPool, Result as VkError};
use thiserror::Error;

use crate::device::Device;

// Creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create command pool")]
pub struct CommandPoolCreateError(#[from] VkError);

impl Device<'_> {
  pub fn create_command_pool(&self, transient: bool, reset_individual_buffers: bool) -> Result<CommandPool, CommandPoolCreateError> {
    use vk::{CommandPoolCreateFlags, CommandPoolCreateInfo};
    let flags = {
      let mut flags = CommandPoolCreateFlags::empty();
      if transient { flags |= CommandPoolCreateFlags::TRANSIENT; }
      if reset_individual_buffers { flags |= CommandPoolCreateFlags::RESET_COMMAND_BUFFER; }
      flags
    };
    let create_info = CommandPoolCreateInfo::builder()
      .flags(flags)
      // TODO: don't assume that command pools are always created for the graphics queue.
      .queue_family_index(self.graphics_queue_index)
      ;
    Ok(unsafe { self.wrapped.create_command_pool(&create_info, None) }?)
  }

  pub unsafe fn destroy_command_pool(&self, command_pool: CommandPool) {
    self.wrapped.destroy_command_pool(command_pool, None)
  }
}

// Reset

#[derive(Error, Debug)]
#[error("Failed to reset command pool")]
pub struct CommandPoolResetError(#[from] VkError);

impl Device<'_> {
  pub unsafe fn reset_command_pool(&self, command_pool: CommandPool, release_resources: bool) -> Result<(), CommandPoolResetError> {
    use vk::CommandPoolResetFlags;
    let flags = {
      let mut flags = CommandPoolResetFlags::empty();
      if release_resources { flags |= CommandPoolResetFlags::RELEASE_RESOURCES }
      flags
    };
    Ok(self.wrapped.reset_command_pool(command_pool, flags)?)
  }
}
