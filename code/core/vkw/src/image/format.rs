use ash::version::InstanceV1_0;
use ash::vk::{Format, FormatFeatureFlags, FormatProperties, ImageTiling, PhysicalDevice};
use thiserror::Error;

use crate::device::Device;
use crate::instance::Instance;

impl Instance {
  pub unsafe fn get_format_properties(&self, physical_device: PhysicalDevice, format: Format) -> FormatProperties {
    self.wrapped.get_physical_device_format_properties(physical_device, format)
  }
}


#[derive(Error, Debug)]
#[error("Failed to find suitable format")]
pub struct FormatFindError;

impl Device {
  pub unsafe fn get_format_properties(&self, format: Format) -> FormatProperties {
    self.instance.get_physical_device_format_properties(self.physical_device, format)
  }

  pub unsafe fn find_suitable_format(&self, formats: &[Format], tiling: ImageTiling, features: FormatFeatureFlags) -> Result<Format, FormatFindError> {
    for format in formats {
      let properties = self.get_format_properties(*format);
      match tiling {
        ImageTiling::OPTIMAL if properties.linear_tiling_features.contains(features) => return Ok(*format),
        ImageTiling::LINEAR if properties.optimal_tiling_features.contains(features) => return Ok(*format),
        _ => {}
      }
    };
    Err(FormatFindError)
  }
}
