use std::ffi::CStr;

use ash::vk::PhysicalDeviceDescriptorIndexingFeaturesEXT;
use byte_strings::c_str;

use crate::device::{DeviceFeatures, DeviceFeaturesQuery};

// API

impl DeviceFeatures {
  pub fn is_descriptor_indexing_extension_enabled(&self) -> bool {
    self.is_extension_enabled(self::DESCRIPTOR_INDEXING_EXTENSION_NAME)
  }
}

impl DeviceFeaturesQuery {
  pub fn want_descriptor_indexing_extension(&mut self) {
    self.want_extension(self::DESCRIPTOR_INDEXING_EXTENSION_NAME);
  }

  pub fn require_descriptor_indexing_extension(&mut self) {
    self.require_extension(self::DESCRIPTOR_INDEXING_EXTENSION_NAME);
  }

  pub fn require_descriptor_indexing_features(&mut self, required_features: PhysicalDeviceDescriptorIndexingFeaturesEXT) {
    self.descriptor_indexing_features = required_features;
  }
}

// Extension name

pub const DESCRIPTOR_INDEXING_EXTENSION_NAME: &'static CStr = c_str!("VK_EXT_descriptor_indexing");
