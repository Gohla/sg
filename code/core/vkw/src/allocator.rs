use core::ptr;
use std::mem::size_of;
use std::ops::Deref;

use ash::vk::{self, Buffer, BufferUsageFlags, DeviceSize, Image, ImageCreateInfo};
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

// Buffer creation

pub struct BufferAllocation {
  pub buffer: Buffer,
  pub allocation: Allocation,
  pub info: AllocationInfo,
}

#[derive(Error, Debug)]
#[error("Failed to allocate buffer: {0:?}")]
pub struct BufferAllocationError(#[from] VkMemError);

impl Allocator {
  pub unsafe fn create_buffer(
    &self,
    size: usize,
    buffer_usage: BufferUsageFlags,
    memory_usage: MemoryUsage,
    flags: AllocationCreateFlags,
  ) -> Result<BufferAllocation, BufferAllocationError> {
    let buffer_info = vk::BufferCreateInfo::builder()
      .size(size as DeviceSize)
      .usage(buffer_usage)
      ;
    let allocation_info = AllocationCreateInfo {
      usage: memory_usage,
      flags,
      ..AllocationCreateInfo::default()
    };
    let (buffer, allocation, info) = self.wrapped.create_buffer(&buffer_info, &allocation_info)?;
    Ok(BufferAllocation { buffer, allocation, info })
  }


  pub unsafe fn create_staging_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.create_buffer(size, BufferUsageFlags::TRANSFER_SRC, MemoryUsage::CpuOnly, AllocationCreateFlags::NONE)
  }

  pub unsafe fn create_staging_buffer_mapped(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.create_buffer(size, BufferUsageFlags::TRANSFER_SRC, MemoryUsage::CpuOnly, AllocationCreateFlags::MAPPED)
  }


  pub unsafe fn create_gpu_vertex_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.create_buffer(size, BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER, MemoryUsage::GpuOnly, AllocationCreateFlags::NONE)
  }

  pub unsafe fn create_cpugpu_vertex_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.create_buffer(size, BufferUsageFlags::VERTEX_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::NONE)
  }

  pub unsafe fn create_cpugpu_vertex_buffer_mapped(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.create_buffer(size, BufferUsageFlags::VERTEX_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::MAPPED)
  }


  pub unsafe fn create_gpu_index_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.create_buffer(size, BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER, MemoryUsage::GpuOnly, AllocationCreateFlags::NONE)
  }

  pub unsafe fn create_cpugpu_index_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.create_buffer(size, BufferUsageFlags::INDEX_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::NONE)
  }

  pub unsafe fn create_cpugpu_index_buffer_mapped(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.create_buffer(size, BufferUsageFlags::INDEX_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::MAPPED)
  }


  pub unsafe fn create_gpu_uniform_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.create_buffer(size, BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::UNIFORM_BUFFER, MemoryUsage::GpuOnly, AllocationCreateFlags::NONE)
  }

  pub unsafe fn create_cpugpu_uniform_buffer(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.create_buffer(size, BufferUsageFlags::UNIFORM_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::NONE)
  }

  pub unsafe fn create_cpugpu_uniform_buffer_mapped(&self, size: usize) -> Result<BufferAllocation, BufferAllocationError> {
    self.create_buffer(size, BufferUsageFlags::UNIFORM_BUFFER, MemoryUsage::CpuToGpu, AllocationCreateFlags::MAPPED)
  }
}

// Staging buffer creation

#[derive(Error, Debug)]
pub enum StagingBufferAllocationError {
  #[error(transparent)]
  BufferAllocationFail(#[from] BufferAllocationError),
  #[error(transparent)]
  MemoryMapFail(#[from] MemoryMapError)
}

impl Allocator {
  pub unsafe fn create_staging_buffer_from_slice<T>(&self, slice: &[T]) -> Result<BufferAllocation, StagingBufferAllocationError> {
    let size = size_of::<T>() * slice.len();
    let buffer_allocation = self.create_staging_buffer(size)?;
    {
      let mapped = buffer_allocation.map(self)?;
      mapped.copy_from_slice(slice);
    }
    Ok(buffer_allocation)
  }
}


// Buffer destruction

impl BufferAllocation {
  pub unsafe fn destroy(&self, allocator: &Allocator) {
    // CORRECTNESS: safe to `ok` - `destroy_buffer` never fails.
    allocator.destroy_buffer(self.buffer, &self.allocation).ok();
  }
}

// Image creation

pub struct ImageAllocation {
  pub image: Image,
  pub allocation: Allocation,
  pub info: AllocationInfo,
}

#[derive(Error, Debug)]
#[error("Failed to allocate image: {0:?}")]
pub struct ImageAllocationError(#[from] VkMemError);

impl Allocator {
  pub unsafe fn create_image(
    &self,
    image_info: &ImageCreateInfo,
    memory_usage: MemoryUsage,
    flags: AllocationCreateFlags,
  ) -> Result<ImageAllocation, ImageAllocationError> {
    let allocation_info = AllocationCreateInfo {
      usage: memory_usage,
      flags,
      ..AllocationCreateInfo::default()
    };
    let (image, allocation, info) = self.wrapped.create_image(image_info, &allocation_info)?;
    Ok(ImageAllocation { image, allocation, info })
  }
}

// Image destruction

impl ImageAllocation {
  pub unsafe fn destroy(&self, allocator: &Allocator) {
    // CORRECTNESS: safe to `ok` - `destroy_buffer` never fails.
    allocator.destroy_image(self.image, &self.allocation).ok();
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

impl MappedMemory<'_> {
  #[inline]
  pub unsafe fn copy_from<T>(&self, src: &T) {
    let src = src as *const T;
    self.copy_from_ptr(src, 1);
  }

  #[inline]
  pub unsafe fn copy_from_slice<T>(&self, src: &[T]) {
    self.copy_from_ptr(src.as_ptr(), src.len());
  }

  #[inline]
  pub unsafe fn copy_from_ptr<T>(&self, src: *const T, count: usize) {
    let dst = self.ptr as *mut T;
    std::ptr::copy_nonoverlapping(src, dst, count);
  }

  #[inline]
  pub unsafe fn copy_from_bytes_slice(&self, src: &[u8]) {
    self.copy_from_ptr(src.as_ptr(), src.len());
  }

  #[inline]
  pub unsafe fn copy_from_bytes_ptr(&self, src: *const u8, count: usize) {
    std::ptr::copy_nonoverlapping(src, self.ptr, count);
  }

  #[inline]
  pub unsafe fn copy_from_bytes_offset_ptr(&self, src: *const u8, dst_offset: isize, count: usize) {
    std::ptr::copy_nonoverlapping(src, self.ptr.offset(dst_offset), count);
  }

  #[inline]
  pub unsafe fn unmap(self) { /* Just drops self */ }
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
