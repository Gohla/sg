use ash::version::DeviceV1_0;
use ash::vk::{self, CommandBuffer, Format};
use thiserror::Error;

use util::image::{Components, Dimensions, ImageData};

use crate::allocator::{Allocator, BufferAllocationError, ImageAllocationError, MemoryMapError};
use crate::command_pool::RecordedStagingBuffer;
use crate::device::Device;
use crate::image::layout_transition::LayoutTransitionError;
use crate::image::sampler::SamplerCreateError;
use crate::image::texture::Texture;
use crate::image::view::ImageViewCreateError;

#[derive(Debug, Error)]
pub enum AllocateRecordCopyTextureArrayError {
  #[error("No image data was given")]
  NoImageDataGiven,
  #[error("Dimensions of image {0:?} differ from dimensions of first image {0:?}")]
  InconsistentDimensions(Dimensions, Dimensions),
  #[error("Image data has {0} components, but 4 components are required")]
  IncorrectComponentCount(u8),
  #[error("Failed to allocate staging buffer")]
  StagingBufferAllocateFail(#[from] BufferAllocationError),
  #[error("Failed to memory map staging buffer")]
  StagingBufferMemoryMapFail(#[from] MemoryMapError),
  #[error(transparent)]
  ImageAllocateFail(#[from] ImageAllocationError),
  #[error(transparent)]
  ImageLayoutTransitionFail(#[from] LayoutTransitionError),
  #[error(transparent)]
  ImageViewCreateFail(#[from] ImageViewCreateError),
  #[error(transparent)]
  SamplerCreateFail(#[from] SamplerCreateError),
}

impl Device {
  pub unsafe fn allocate_record_copy_texture_array(
    &self,
    images_data: &[ImageData],
    allocator: &Allocator,
    format: Format,
    command_buffer: CommandBuffer,
  ) -> Result<RecordedStagingBuffer<Texture>, AllocateRecordCopyTextureArrayError> {
    use AllocateRecordCopyTextureArrayError::*;
    use vk::{Extent3D, ImageAspectFlags, ImageUsageFlags, ImageLayout};

    if images_data.is_empty() {
      return Err(NoImageDataGiven);
    }

    let dimensions = images_data[0].dimensions;
    for image_data in images_data {
      let dim = image_data.dimensions;
      if dim != dimensions {
        return Err(InconsistentDimensions(dim, dimensions));
      }
      if dim.components != Components::Components4 {
        return Err(IncorrectComponentCount(dim.components.into()));
      }
    }
    let layer_count = images_data.len();
    let size = dimensions.num_bytes();

    let staging_buffer = allocator.create_staging_buffer(size * layer_count)?;
    {
      let map = staging_buffer.map(allocator)?;
      let mut dst_offset = 0;
      for image_data in images_data {
        map.copy_from_bytes_offset_ptr(image_data.data_ptr(), dst_offset, size);
        dst_offset += size as isize;
      }
    }

    let image_info = vk::ImageCreateInfo::builder()
      .image_type(vk::ImageType::TYPE_2D)
      .format(format)
      .extent(Extent3D { width: dimensions.width, height: dimensions.height, depth: 1 })
      .mip_levels(1)
      .array_layers(layer_count as u32)
      .samples(vk::SampleCountFlags::TYPE_1)
      .tiling(vk::ImageTiling::OPTIMAL)
      .usage(ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED)
      .sharing_mode(vk::SharingMode::EXCLUSIVE)
      .initial_layout(vk::ImageLayout::UNDEFINED)
      ;
    let image_allocation = allocator.create_image(&image_info, vk_mem::MemoryUsage::GpuOnly, vk_mem::AllocationCreateFlags::NONE)?;

    self.record_images_layout_transition(
      std::iter::once(image_allocation.image),
      format,
      ImageLayout::UNDEFINED,
      ImageLayout::TRANSFER_DST_OPTIMAL,
      layer_count as u32,
      command_buffer,
    )?;

    let regions: Vec<_> = (0..layer_count).into_iter()
      .map(|i| {
        let buffer_offset = i * size;
        vk::BufferImageCopy::builder()
          .buffer_offset(buffer_offset as u64)
          .buffer_row_length(0)
          .buffer_image_height(0)
          .image_subresource(vk::ImageSubresourceLayers::builder()
            .aspect_mask(ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(i as u32)
            .layer_count(1)
            .build()
          )
          .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
          .image_extent(Extent3D { width: dimensions.width, height: dimensions.height, depth: 1 })
          .build()
      })
      .collect();
    self.cmd_copy_buffer_to_image(
      command_buffer,
      staging_buffer.buffer,
      image_allocation.image,
      ImageLayout::TRANSFER_DST_OPTIMAL,
      &regions,
    );

    self.record_images_layout_transition(
      std::iter::once(image_allocation.image),
      format,
      ImageLayout::TRANSFER_DST_OPTIMAL,
      ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      1,
      command_buffer,
    )?;

    let view = self.create_image_view(image_allocation.image, format, vk::ImageViewType::TYPE_2D, ImageAspectFlags::COLOR, layer_count as u32)?;
    let sampler = self.create_default_sampler()?;
    let texture = Texture { allocation: image_allocation, view, sampler };
    Ok(RecordedStagingBuffer::new(staging_buffer, texture))
  }
}
