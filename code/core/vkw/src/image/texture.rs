use ash::version::DeviceV1_0;
use ash::vk::{self, CommandBuffer, Format, ImageView, Sampler};
use thiserror::Error;

use util::image::{Components, Dimensions, ImageData};

use crate::allocator::{Allocator, ImageAllocation, ImageAllocationError, StagingBufferAllocationError};
use crate::command_pool::RecordedStagingBuffer;
use crate::device::Device;
use crate::image::layout_transition::LayoutTransitionError;
use crate::image::sampler::SamplerCreateError;
use crate::image::view::ImageViewCreateError;

pub struct Texture {
  pub allocation: ImageAllocation,
  pub view: ImageView,
  pub sampler: Sampler,
}

impl Texture {
  pub unsafe fn destroy(&self, device: &Device, allocator: &Allocator) {
    device.destroy_sampler(self.sampler);
    device.destroy_image_view(self.view);
    self.allocation.destroy(allocator);
  }
}

#[derive(Debug, Error)]
pub enum AllocateRecordCopyTexturesError {
  #[error("Image data has {0} components, but 4 components are required")]
  IncorrectComponentCount(u8),
  #[error(transparent)]
  StagingBufferAllocateFail(#[from] StagingBufferAllocationError),
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
  pub unsafe fn allocate_record_copy_textures<I: IntoIterator<Item=ImageData>>(
    &self,
    images_data: I,
    allocator: &Allocator,
    format: Format,
    command_buffer: CommandBuffer,
  ) -> Result<Vec<RecordedStagingBuffer<Texture>>, AllocateRecordCopyTexturesError> {
    use AllocateRecordCopyTexturesError::*;
    use crate::allocator::{BufferAllocation};
    use vk::{Extent3D, ImageAspectFlags, ImageUsageFlags, ImageLayout};

    struct Transfer {
      dimensions: Dimensions,
      staging_buffer: BufferAllocation,
      image_allocation: ImageAllocation,
    }
    let transfers: Result<Vec<Transfer>, _> = images_data.into_iter().map(|image_data: ImageData| {
      let dimensions = image_data.dimensions;
      if dimensions.components != Components::Components4 {
        return Err(IncorrectComponentCount(dimensions.components.into()))
      }
      let staging_buffer = allocator.create_staging_from_slice(image_data.data_slice())?;
      let image_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(Extent3D { width: dimensions.width, height: dimensions.height, depth: 1 })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        ;
      let image_allocation = allocator.create_image(&image_info, vk_mem::MemoryUsage::GpuOnly, vk_mem::AllocationCreateFlags::NONE)?;
      Ok(Transfer { dimensions, staging_buffer, image_allocation })
    }).collect();
    let transfers = transfers?;

    self.record_images_layout_transition(
      transfers.iter().map(|t| t.image_allocation.image),
      format,
      ImageLayout::UNDEFINED,
      ImageLayout::TRANSFER_DST_OPTIMAL,
      1,
      command_buffer
    )?;
    for transfer in &transfers {
      self.cmd_copy_buffer_to_image(
        command_buffer,
        transfer.staging_buffer.buffer,
        transfer.image_allocation.image,
        ImageLayout::TRANSFER_DST_OPTIMAL,
        &[vk::BufferImageCopy::builder()
          .buffer_offset(0)
          .buffer_row_length(0)
          .buffer_image_height(0)
          .image_subresource(vk::ImageSubresourceLayers::builder()
            .aspect_mask(ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .build()
          )
          .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
          .image_extent(Extent3D { width: transfer.dimensions.width, height: transfer.dimensions.height, depth: 1 })
          .build()
        ]
      );
    }
    self.record_images_layout_transition(
      transfers.iter().map(|t| t.image_allocation.image),
      format,
      ImageLayout::TRANSFER_DST_OPTIMAL,
      ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      1,
      command_buffer
    )?;

    transfers.into_iter().map(|t| {
      let view = self.create_image_view(t.image_allocation.image, format, vk::ImageViewType::TYPE_2D, ImageAspectFlags::COLOR, 1)?;
      let sampler = self.create_default_sampler()?;
      let texture = Texture { allocation: t.image_allocation, view, sampler };
      Ok(RecordedStagingBuffer::new(t.staging_buffer, texture))
    }).collect()
  }
}
