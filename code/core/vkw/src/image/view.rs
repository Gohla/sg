use ash::version::DeviceV1_0;
use ash::vk::{self, Format, Image, ImageAspectFlags, ImageView, ImageViewType, Result as VkError};
use thiserror::Error;

use crate::prelude::Device;

#[derive(Error, Debug)]
pub enum ImageViewCreateError {
  #[error("Failed to create image view")]
  ImageViewCreateFail(#[source] VkError),
}

impl Device<'_, '_> {
  pub fn create_image_view(
    &self,
    image: Image,
    format: Format,
    view_type: ImageViewType,
    aspect_mask: ImageAspectFlags,
    layer_count: u32,
  ) -> Result<ImageView, ImageViewCreateError> {
    use ImageViewCreateError::*;

    let create_info = vk::ImageViewCreateInfo::builder()
      .image(image)
      .view_type(view_type)
      .format(format)
      .subresource_range(vk::ImageSubresourceRange {
        aspect_mask,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count,
      });
    Ok(unsafe { self.wrapped.create_image_view(&create_info, None) }.map_err(|e| ImageViewCreateFail(e))?)
  }

  pub fn destroy_image_view(&self, image_view: ImageView) {
    unsafe { self.wrapped.destroy_image_view(image_view, None); }
  }
}
