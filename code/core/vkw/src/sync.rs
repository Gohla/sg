use ash::version::DeviceV1_0;
use ash::vk::{self, Fence, Result as VkError, Semaphore};
use thiserror::Error;

use crate::device::Device;
use crate::timeout::Timeout;

// Fence creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create fence")]
pub struct FenceCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_fence(&self, signaled: bool) -> Result<Fence, FenceCreateError> {
    use vk::{FenceCreateFlags, FenceCreateInfo};
    let flags = if signaled { FenceCreateFlags::SIGNALED } else { FenceCreateFlags::empty() };
    let create_info = FenceCreateInfo::builder()
      .flags(flags)
      ;
    Ok(self.wrapped.create_fence(&create_info, None)?)
  }

  pub unsafe fn destroy_fence(&self, fence: Fence) {
    self.wrapped.destroy_fence(fence, None)
  }
}

// Fence wait

#[derive(Error, Debug)]
#[error("Failed to wait for fences")]
pub struct FenceWaitError(#[from] VkError);

impl Device {
  pub unsafe fn wait_for_fence(&self, fence: Fence, timeout: Timeout) -> Result<(), FenceWaitError> {
    Ok(self.wrapped.wait_for_fences(&[fence], true, timeout.into())?)
  }

  pub unsafe fn wait_for_fences(&self, fences: &[Fence], wait_all: bool, timeout: Timeout) -> Result<(), FenceWaitError> {
    Ok(self.wrapped.wait_for_fences(fences, wait_all, timeout.into())?)
  }
}

// Fence reset

#[derive(Error, Debug)]
#[error("Failed to reset fences")]
pub struct FenceResetError(#[from] VkError);

impl Device {
  pub unsafe fn reset_fence(&self, fence: Fence) -> Result<(), FenceResetError> {
    Ok(self.wrapped.reset_fences(&[fence])?)
  }

  pub unsafe fn reset_fences(&self, fences: &[Fence]) -> Result<(), FenceResetError> {
    Ok(self.wrapped.reset_fences(fences)?)
  }
}

// Semaphore creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create semaphore")]
pub struct SemaphoreCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_semaphore(&self) -> Result<Semaphore, SemaphoreCreateError> {
    use vk::SemaphoreCreateInfo;
    let create_info = SemaphoreCreateInfo::builder();
    Ok(self.wrapped.create_semaphore(&create_info, None)?)
  }

  pub unsafe fn destroy_semaphore(&self, semaphore: Semaphore) {
    self.wrapped.destroy_semaphore(semaphore, None)
  }
}

// Device wait idle

#[derive(Error, Debug)]
#[error("Failed to wait for device idle")]
pub struct WaitIdleError(#[from] VkError);

impl Device {
  pub unsafe fn wait_idle(&self) -> Result<(), WaitIdleError> {
    Ok(self.wrapped.device_wait_idle()?)
  }
}
