use core::ptr;
use std::ops::Deref;

use ash::vk::{Buffer, BufferCreateInfo, BufferUsageFlags, DeviceSize};
use log::debug;
use thiserror::Error;
use vk_mem::{Allocation, AllocationCreateFlags, AllocationCreateInfo, AllocationInfo, Allocator as VkMemAllocator, AllocatorCreateInfo, Error as VkMemError, MemoryUsage};

use crate::device::Device;
use crate::instance::Instance;

// Wrapper

pub struct Allocator {
  pub wrapped: VkMemAllocator
}

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
    let allocator = VkMemAllocator::new(&create_info)?;
    debug!("Created allocator");
    Ok(Allocator { wrapped: allocator })
  }
}

// Destruction

impl Allocator {
  pub unsafe fn destroy(&mut self) {
    self.wrapped.destroy();
  }
}

// Buffer allocation

pub struct BufferAllocation {
  pub buffer: Buffer,
  pub allocation: Allocation,
  pub info: AllocationInfo,
}

#[derive(Error, Debug)]
#[error("Failed to allocate buffer: {0:?}")]
pub struct BufferAllocationError(#[from] VkMemError);

impl Allocator {
  pub unsafe fn allocate_buffer(
    &self,
    size: usize,
    buffer_usage: BufferUsageFlags,
    memory_usage: MemoryUsage,
    flags: AllocationCreateFlags,
  ) -> Result<BufferAllocation, BufferAllocationError> {
    let (buffer, allocation, info) = self.create_buffer(
      &BufferCreateInfo::builder().size(size as DeviceSize).usage(buffer_usage),
      &AllocationCreateInfo { usage: memory_usage, flags, ..AllocationCreateInfo::default() }
    )?;
    Ok(BufferAllocation { buffer, allocation, info })
  }


  pub unsafe fn allocate_host_staging_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.allocate_buffer(size, BufferUsageFlags::TRANSFER_SRC, MemoryUsage::CpuOnly, AllocationCreateFlags::NONE)
  }

  pub unsafe fn allocate_host_staging_buffer_mapped(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.allocate_buffer(size, BufferUsageFlags::TRANSFER_SRC, MemoryUsage::CpuOnly, AllocationCreateFlags::MAPPED)
  }


  pub unsafe fn allocate_device_static_vertex_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.allocate_buffer(size, BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER, MemoryUsage::GpuOnly, AllocationCreateFlags::NONE)
  }

  pub unsafe fn allocate_device_dynamic_vertex_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.allocate_buffer(size, BufferUsageFlags::VERTEX_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::NONE)
  }

  pub unsafe fn allocate_device_dynamic_vertex_buffer_mapped(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.allocate_buffer(size, BufferUsageFlags::VERTEX_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::MAPPED)
  }


  pub unsafe fn allocate_device_static_index_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.allocate_buffer(size, BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER, MemoryUsage::GpuOnly, AllocationCreateFlags::NONE)
  }

  pub unsafe fn allocate_device_dynamic_index_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.allocate_buffer(size, BufferUsageFlags::INDEX_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::NONE)
  }

  pub unsafe fn allocate_device_dynamic_index_buffer_mapped(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.allocate_buffer(size, BufferUsageFlags::INDEX_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::MAPPED)
  }


  pub unsafe fn allocate_device_static_uniform_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.allocate_buffer(size, BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::UNIFORM_BUFFER, MemoryUsage::GpuOnly, AllocationCreateFlags::NONE)
  }

  pub unsafe fn allocate_device_dynamic_uniform_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.allocate_buffer(size, BufferUsageFlags::UNIFORM_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::NONE)
  }

  pub unsafe fn allocate_device_dynamic_uniform_buffer_mapped(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.allocate_buffer(size, BufferUsageFlags::UNIFORM_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::MAPPED)
  }
}

// Buffer deallocation/destruction

impl BufferAllocation {
  pub unsafe fn destroy(&mut self, allocator: &Allocator) {
    // CORRECTNESS: safe to `ok` - `destroy_buffer` never fails.
    allocator.destroy_buffer(self.buffer, &self.allocation).ok();
  }
}

// Memory mapping

#[derive(Error, Debug)]
#[error("Failed to map memory: {0:?}")]
pub struct MemoryMapError(#[from] VkMemError);

pub struct MappedMemory<'a> {
  ptr: *mut u8,
  unmap: Option<(&'a Allocator, &'a Allocation)>,
}

impl BufferAllocation {
  /// Returns a pointer to the mapped data if memory is persistently mapped, `None` otherwise.
  pub unsafe fn get_mapped_data(&self) -> Option<MappedMemory> {
    let ptr = self.info.get_mapped_data();
    if ptr == ptr::null_mut() {
      None
    } else {
      Some(MappedMemory { ptr, unmap: None })
    }
  }

  pub unsafe fn map<'a>(&'a self, allocator: &'a Allocator) -> Result<MappedMemory<'a>, MemoryMapError> {
    let allocation = &self.allocation;
    let ptr = allocator.map_memory(allocation)?;
    Ok(MappedMemory { ptr, unmap: Some((allocator, allocation)) })
  }
}

impl MappedMemory<'_> {
  pub unsafe fn copy_from_slice<T>(&self, src: &[T]) {
    self.copy_from(src.as_ptr(), src.len());
  }

  pub unsafe fn copy_from<T>(&self, src: *const T, count: usize) {
    let dst = self.ptr as *mut T;
    std::ptr::copy_nonoverlapping(src, dst, count);
  }

  pub unsafe fn unmap(self) { /* Just drops self */ }
}

// Implementations

impl Deref for Allocator {
  type Target = VkMemAllocator;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.wrapped }
}

impl<'a> Drop for MappedMemory<'a> {
  fn drop(&mut self) {
    if let Some((allocator, allocation)) = self.unmap {
      // CORRECTNESS: safe to `ok` - `unmap_memory` never fails.
      allocator.wrapped.unmap_memory(allocation).ok();
    }
  }
}
