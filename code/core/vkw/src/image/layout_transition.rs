use ash::version::DeviceV1_0;
use ash::vk::{self, AccessFlags, CommandBuffer, DependencyFlags, Format, Image, ImageAspectFlags, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, PipelineStageFlags};
use thiserror::Error;

use crate::device::Device;

#[derive(Error, Debug)]
#[error("Failed to record image layout transition")]
pub struct LayoutTransitionError;

impl Device {
  pub fn record_images_layout_transition<I: IntoIterator<Item=Image>>(
    &self,
    images: I,
    format: Format,
    old_layout: ImageLayout,
    new_layout: ImageLayout,
    layer_count: u32,
    command_buffer: CommandBuffer,
  ) -> Result<(), LayoutTransitionError> {
    // Determine access masks and pipeline stages.
    let (src_access_mask, dst_access_mask, src_stage, dst_stage) = match (old_layout, new_layout) {
      (ImageLayout::UNDEFINED, ImageLayout::TRANSFER_DST_OPTIMAL) => (
        AccessFlags::empty(), AccessFlags::TRANSFER_WRITE, PipelineStageFlags::TOP_OF_PIPE, PipelineStageFlags::TRANSFER
      ),
      (ImageLayout::TRANSFER_DST_OPTIMAL, ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
        AccessFlags::TRANSFER_WRITE, AccessFlags::SHADER_READ, PipelineStageFlags::TRANSFER, PipelineStageFlags::FRAGMENT_SHADER
      ),
      (ImageLayout::UNDEFINED, ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
        AccessFlags::empty(), AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE, PipelineStageFlags::TOP_OF_PIPE, PipelineStageFlags::EARLY_FRAGMENT_TESTS
      ),
      _ => return Err(LayoutTransitionError),
    };
    // Determine aspect mask/
    let mut aspect_mask = ImageAspectFlags::empty();
    if new_layout == ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
      aspect_mask |= ImageAspectFlags::DEPTH;
      if Self::has_stencil_component(format) {
        aspect_mask |= ImageAspectFlags::STENCIL;
      }
    } else {
      aspect_mask |= ImageAspectFlags::COLOR;
    }
    // Create image barrier.
    let image_memory_barriers: Vec<_> = images.into_iter().map(|image| ImageMemoryBarrier::builder()
      .src_access_mask(src_access_mask)
      .dst_access_mask(dst_access_mask)
      .old_layout(old_layout)
      .new_layout(new_layout)
      .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
      .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
      .image(image)
      .subresource_range(ImageSubresourceRange::builder()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(layer_count)
        .build()
      )
      .build()
    ).collect();
    // Record layout transition.
    unsafe {
      self.cmd_pipeline_barrier(
        command_buffer,
        src_stage,
        dst_stage,
        DependencyFlags::empty(),
        &[],
        &[],
        &image_memory_barriers
      )
    };
    Ok(())
  }


  fn has_stencil_component(format: Format) -> bool {
    match format {
      Format::D32_SFLOAT_S8_UINT => true,
      Format::D24_UNORM_S8_UINT => true,
      _ => false,
    }
  }
}
