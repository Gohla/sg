use std::ffi::CStr;

use byte_strings::c_str;

use crate::instance::{InstanceFeatures, InstanceFeaturesQuery};

// Layer name

pub const VALIDATION_LAYER_NAME: &'static CStr = c_str!("VK_LAYER_LUNARG_standard_validation");

// Implementations

impl InstanceFeatures {
  pub fn is_validation_layer_enabled(&self) -> bool {
    self.is_layer_enabled(self::VALIDATION_LAYER_NAME)
  }
}

impl InstanceFeaturesQuery {
  pub fn want_validation_layer(&mut self) {
    self.want_layer(self::VALIDATION_LAYER_NAME);
    self.want_debug_report_extension(); // Debug report extension is needed for reporting validation errors.
  }

  pub fn require_validation_layer(&mut self) {
    self.require_layer(self::VALIDATION_LAYER_NAME);
    self.require_debug_report_extension(); // Debug report extension is needed for reporting validation errors.
  }
}
