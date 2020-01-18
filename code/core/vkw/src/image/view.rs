use ash::version::DeviceV1_0;
use ash::vk::{self, Buffer, BufferView, DeviceSize, Format, Image, ImageAspectFlags, ImageView, ImageViewType, Result as VkError};
use log::trace;
use thiserror::Error;

use crate::device::Device;

// Image view creation/destruction

#[derive(Error, Debug)]
#[error("Failed to create image view: {0:?}")]
pub struct ImageViewCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_image_view(
    &self,
    image: Image,
    format: Format,
    view_type: ImageViewType,
    aspect_mask: ImageAspectFlags,
    layer_count: u32,
  ) -> Result<ImageView, ImageViewCreateError> {
    let create_info = vk::ImageViewCreateInfo::builder()
      .image(image)
      .view_type(view_type)
      .format(format)
      .components(vk::ComponentMapping::builder()
        .r(vk::ComponentSwizzle::IDENTITY)
        .g(vk::ComponentSwizzle::IDENTITY)
        .b(vk::ComponentSwizzle::IDENTITY)
        .a(vk::ComponentSwizzle::IDENTITY)
        .build()
      )
      .subresource_range(vk::ImageSubresourceRange::builder()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(layer_count)
        .build()
      )
      ;
    let image_view = self.wrapped.create_image_view(&create_info, None)?;
    trace!("Created image view {:?}", image_view);
    Ok(image_view)
  }

  pub unsafe fn destroy_image_view(&self, image_view: ImageView) {
    trace!("Destroying image view {:?}", image_view);
    self.wrapped.destroy_image_view(image_view, None);
  }
}

// Buffer view creation/destruction

#[derive(Error, Debug)]
#[error("Failed to create image view: {0:?}")]
pub struct BufferViewCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_buffer_view(&self, buffer: Buffer, format: Format, offset: DeviceSize, range: DeviceSize) -> Result<BufferView, BufferViewCreateError> {
    let create_info = vk::BufferViewCreateInfo::builder()
      .buffer(buffer)
      .format(format)
      .offset(offset)
      .range(range)
      ;
    let buffer_view = self.wrapped.create_buffer_view(&create_info, None)?;
    trace!("Created buffer view {:?}", buffer_view);
    Ok(buffer_view)
  }

  pub unsafe fn destroy_buffer_view(&self, buffer_view: BufferView) {
    self.wrapped.destroy_buffer_view(buffer_view, None);
  }
}
