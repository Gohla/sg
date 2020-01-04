use log::debug;
use thiserror::Error;
use vk_mem::{Allocator, AllocatorCreateInfo, Error as VkMemError};

use crate::device::Device;
use crate::instance::Instance;

// Creation

#[derive(Error, Debug)]
#[error("Failed to create allocator: {0:?}")]
pub struct AllocatorCreateError(#[from] VkMemError);

impl Device {
  pub unsafe fn create_allocator(&self, instance: &Instance) -> Result<Allocator, AllocatorCreateError> {
    let create_info = AllocatorCreateInfo {
      physical_device: self.physical_device,
      device: self.wrapped.clone(),
      instance: instance.wrapped.clone(),
      ..AllocatorCreateInfo::default()
    };
    let allocator = Allocator::new(&create_info)?;
    debug!("Created allocator");
    Ok(allocator)
  }
}
