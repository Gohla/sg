use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

use ash::extensions::ext::DebugReport as VkDebugReport;
use ash::vk::{self, DebugReportCallbackEXT, DebugReportFlagsEXT, DebugReportObjectTypeEXT, Result as VkError};
use byte_strings::c_str;

use crate::instance::InstanceFeatures;

use super::{Instance, InstanceFeaturesQuery};

// Wrapper

pub struct DebugReport {
  loader: VkDebugReport,
  callback: DebugReportCallbackEXT,
}

// Creation

impl DebugReport {
  pub fn new(instance: &Instance) -> Result<Self, VkError> {
    use vk::DebugReportCallbackCreateInfoEXT;

    let info = DebugReportCallbackCreateInfoEXT::builder()
      .flags(DebugReportFlagsEXT::ERROR | DebugReportFlagsEXT::WARNING | DebugReportFlagsEXT::PERFORMANCE_WARNING)
      .pfn_callback(Some(vulkan_debug_callback));
    let loader = VkDebugReport::new(&instance.entry.wrapped, &instance.wrapped);
    let callback = unsafe { loader.create_debug_report_callback(&info, None) }?;
    Ok(Self { loader, callback })
  }
}

// API

impl InstanceFeaturesQuery {
  pub fn want_debug_report_extension(&mut self) {
    self.want_extension(self::DEBUG_REPORT_EXTENSION_NAME);
  }

  pub fn require_debug_report_extension(&mut self) {
    self.require_extension(self::DEBUG_REPORT_EXTENSION_NAME);
  }
}

impl InstanceFeatures {
  pub fn is_debug_report_extension_enabled(&self) -> bool {
    self.is_extension_enabled(self::DEBUG_REPORT_EXTENSION_NAME)
  }
}

// Implementations

impl Drop for DebugReport {
  fn drop(&mut self) {
    unsafe { self.loader.destroy_debug_report_callback(self.callback, None); }
  }
}

// Extension name

pub const DEBUG_REPORT_EXTENSION_NAME: &'static CStr = c_str!("VK_EXT_debug_report");

// Callback

unsafe extern "system" fn vulkan_debug_callback(
  flags: DebugReportFlagsEXT,
  _object_type: DebugReportObjectTypeEXT,
  _object: u64,
  _location: usize,
  _message_code: i32,
  _p_layer_prefix: *const c_char,
  p_message: *const c_char,
  _p_user_data: *mut c_void,
) -> u32 {
  use log::{Level, log as log_macro};

  let level = match flags {
    DebugReportFlagsEXT::ERROR => Level::Error,
    DebugReportFlagsEXT::WARNING => Level::Warn,
    DebugReportFlagsEXT::PERFORMANCE_WARNING => Level::Warn,
    DebugReportFlagsEXT::INFORMATION => Level::Info,
    DebugReportFlagsEXT::DEBUG => Level::Debug,
    _ => Level::Trace,
  };
  let msg = CStr::from_ptr(p_message);
  log_macro!(level, "{:?}", msg);
  vk::FALSE
}
