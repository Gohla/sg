use std::borrow::Borrow;
use std::collections::HashSet;
use std::ffi::{CStr, CString};

use ash::Instance;
use ash::version::InstanceV1_0;
use byte_strings::c_str;

use crate::entry::{DebugReporter, VkEntry};

pub const VALIDATION_LAYER_NAME: &'static CStr = c_str!("VK_LAYER_LUNARG_standard_validation");


pub const SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_surface");

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_xlib_surface");
#[cfg(target_os = "macos")]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_MVK_macos_surface");
#[cfg(all(windows))]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_win32_surface");

pub const DEBUG_REPORT_EXTENSION_NAME: &'static CStr = c_str!("VK_EXT_debug_report");


#[derive(Debug)]
pub struct InstanceFeatures {
  enabled_layers: HashSet<CString>,
  enabled_extensions: HashSet<CString>,
}

impl InstanceFeatures {
  pub(crate) fn new(enabled_layers: HashSet<CString>, enabled_extensions: HashSet<CString>) -> Self {
    Self { enabled_layers, enabled_extensions }
  }


  pub fn is_layer_enabled<B: Borrow<CStr> + ?Sized>(&self, layer_name: &B) -> bool {
    self.enabled_layers.contains(layer_name.borrow())
  }

  pub fn is_validation_layer_enabled(&self) -> bool {
    self.is_layer_enabled(self::VALIDATION_LAYER_NAME)
  }

  pub fn is_extension_enabled<B: Borrow<CStr> + ?Sized>(&self, extension_name: &B) -> bool {
    self.enabled_extensions.contains(extension_name.borrow())
  }

  pub fn is_debug_report_extension_enabled(&self) -> bool {
    self.is_extension_enabled(self::DEBUG_REPORT_EXTENSION_NAME)
  }
}


pub struct VkInstance<'e> {
  pub entry: &'e VkEntry,
  pub instance: Instance,
  pub instance_features: InstanceFeatures,
  debug_reporter: Option<DebugReporter>,
}

impl<'e> VkInstance<'e> {
  pub(crate) fn new(
    entry: &'e VkEntry,
    instance: Instance,
    instance_features: InstanceFeatures,
    debug_reporter: Option<DebugReporter>,
  ) -> Self {
    Self {
      entry,
      instance,
      instance_features,
      debug_reporter,
    }
  }
}

impl<'e> Drop for VkInstance<'e> {
  fn drop(&mut self) {
    if let Some(debug_reporter) = &self.debug_reporter {
      debug_reporter.destroy();
    }
    unsafe { self.instance.destroy_instance(None) };
  }
}
