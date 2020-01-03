use ash::version::DeviceV1_0;
use ash::vk::{self, Fence, Queue, Result as VkError, Semaphore};
use log::{trace, debug};
use thiserror::Error;

use crate::device::Device;
use crate::timeout::Timeout;

// Fence creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create fence: {0:?}")]
pub struct FenceCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_fence(&self, signaled: bool) -> Result<Fence, FenceCreateError> {
    use vk::{FenceCreateFlags, FenceCreateInfo};
    let flags = if signaled { FenceCreateFlags::SIGNALED } else { FenceCreateFlags::empty() };
    let create_info = FenceCreateInfo::builder()
      .flags(flags)
      .build();
    trace!("Creating fence from {:?}", create_info);
    Ok(self.wrapped.create_fence(&create_info, None)?)
  }

  pub unsafe fn destroy_fence(&self, fence: Fence) {
    trace!("Destroying fence {:?}", fence);
    self.wrapped.destroy_fence(fence, None)
  }
}

// Fence wait

#[derive(Error, Debug)]
#[error("Failed to wait for fences: {0:?}")]
pub struct FenceWaitError(#[from] VkError);

impl Device {
  pub unsafe fn wait_for_fences(&self, fences: &[Fence], wait_all: bool, timeout: Timeout) -> Result<(), FenceWaitError> {
    trace!("Waiting for {} fences {:?}", if wait_all { "all" } else { "one of" }, fences);
    Ok(self.wrapped.wait_for_fences(fences, wait_all, timeout.into())?)
  }

  pub unsafe fn wait_for_fence(&self, fence: Fence, timeout: Timeout) -> Result<(), FenceWaitError> {
    self.wait_for_fences(&[fence], true, timeout)
  }
}

// Fence reset

#[derive(Error, Debug)]
#[error("Failed to reset fences: {0:?}")]
pub struct FenceResetError(#[from] VkError);

impl Device {
  pub unsafe fn reset_fences(&self, fences: &[Fence]) -> Result<(), FenceResetError> {
    trace!("Resetting fences {:?}", fences);
    Ok(self.wrapped.reset_fences(fences)?)
  }

  pub unsafe fn reset_fence(&self, fence: Fence) -> Result<(), FenceResetError> {
    self.reset_fences(&[fence])
  }
}

// Semaphore creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create semaphore: {0:?}")]
pub struct SemaphoreCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_semaphore(&self) -> Result<Semaphore, SemaphoreCreateError> {
    use vk::SemaphoreCreateInfo;
    let create_info = SemaphoreCreateInfo::builder().build();
    trace!("Creating semaphore from {:?}", create_info);
    Ok(self.wrapped.create_semaphore(&create_info, None)?)
  }

  pub unsafe fn destroy_semaphore(&self, semaphore: Semaphore) {
    trace!("Destroying semaphore {:?}", semaphore);
    self.wrapped.destroy_semaphore(semaphore, None)
  }
}

// Wait idle

#[derive(Error, Debug)]
#[error("Failed to wait for queue idle: {0:?}")]
pub struct QueueWaitIdleError(#[from] VkError);

impl Device {
  pub unsafe fn queue_wait_idle(&self, queue: Queue) -> Result<(), QueueWaitIdleError> {
    debug!("Waiting for queue {:?} idle", queue);
    Ok(self.wrapped.queue_wait_idle(queue)?)
  }
}

#[derive(Error, Debug)]
#[error("Failed to wait for device idle: {0:?}")]
pub struct DeviceWaitIdleError(#[from] VkError);

impl Device {
  pub unsafe fn device_wait_idle(&self) -> Result<(), DeviceWaitIdleError> {
    debug!("Waiting for device {:?} idle", self.wrapped.handle());
    Ok(self.wrapped.device_wait_idle()?)
  }
}
