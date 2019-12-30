use std::ffi::CStr;

use byte_strings::c_str;

use crate::device::{DeviceFeatures, DeviceFeaturesQuery};

// Extension name

pub const SWAPCHAIN_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_swapchain");

// Implementations

impl DeviceFeatures {
  pub fn is_swapchain_extension_enabled(&self) -> bool {
    self.is_extension_enabled(self::SWAPCHAIN_EXTENSION_NAME)
  }
}

impl DeviceFeaturesQuery {
  pub fn want_swapchain_extension(&mut self) {
    self.want_extension(self::SWAPCHAIN_EXTENSION_NAME);
  }

  pub fn require_swapchain_extension(&mut self) {
    self.require_extension(self::SWAPCHAIN_EXTENSION_NAME);
  }
}
