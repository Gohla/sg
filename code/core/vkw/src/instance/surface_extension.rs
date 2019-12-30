use std::ffi::CStr;

use byte_strings::c_str;

use crate::instance::{InstanceFeatures, InstanceFeaturesQuery};

// Extension names

pub const SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_surface");

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_xlib_surface");
#[cfg(target_os = "macos")]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_MVK_macos_surface");
#[cfg(all(windows))]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_win32_surface");

// Implementations

impl InstanceFeatures {
  pub fn is_surface_extension_enabled(&self) -> bool {
    self.is_extension_enabled(self::SURFACE_EXTENSION_NAME) && self.is_extension_enabled(self::PLATFORM_SURFACE_EXTENSION_NAME)
  }
}

impl InstanceFeaturesQuery {
  pub fn want_surface(&mut self) {
    self.want_extension(self::SURFACE_EXTENSION_NAME);
    self.want_extension(self::PLATFORM_SURFACE_EXTENSION_NAME);
  }

  pub fn require_surface(&mut self) {
    self.require_extension(self::SURFACE_EXTENSION_NAME);
    self.require_extension(self::PLATFORM_SURFACE_EXTENSION_NAME);
  }
}
