//! # Safety
//!
//! Safe usage prohibits:
//!
//! * Calling methods or getting fields of [`Instance`] after it has been [destroyed](Instance::destroy).
//!
//! # Destruction
//!
//! An [`Instance`] must be manually destroyed with [`Instance::destroy`].

use std::borrow::Borrow;
use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::ops::Deref;

use ash::{Instance as VkInstance, InstanceError};
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk::{self, Result as VkError};
use log::trace;
use thiserror::Error;

use crate::entry::Entry;
use crate::version::VkVersion;

pub mod validation_layer;
pub mod debug_report_extension;
pub mod surface_extension;

// Wrapper

pub struct Instance {
  pub entry: Entry,
  pub wrapped: VkInstance,
  pub features: InstanceFeatures,
}

#[derive(Debug)]
pub struct InstanceFeatures {
  pub enabled_layers: HashSet<CString>,
  pub enabled_extensions: HashSet<CString>,
}

impl InstanceFeatures {
  fn new(enabled_layers: HashSet<CString>, enabled_extensions: HashSet<CString>) -> Self {
    Self { enabled_layers, enabled_extensions }
  }

  pub fn is_layer_enabled<B: Borrow<CStr> + ?Sized>(&self, layer_name: &B) -> bool {
    self.enabled_layers.contains(layer_name.borrow())
  }

  pub fn is_extension_enabled<B: Borrow<CStr> + ?Sized>(&self, extension_name: &B) -> bool {
    self.enabled_extensions.contains(extension_name.borrow())
  }
}

// Creation and destruction

#[derive(Default, Debug)]
pub struct InstanceFeaturesQuery {
  wanted_layers: HashSet<CString>,
  required_layers: HashSet<CString>,
  wanted_extensions: HashSet<CString>,
  required_extensions: HashSet<CString>,
}

impl InstanceFeaturesQuery {
  pub fn new() -> Self { Self::default() }

  pub fn want_layer<S: Into<CString>>(&mut self, name: S) {
    self.wanted_layers.insert(name.into());
  }

  pub fn require_layer<S: Into<CString>>(&mut self, name: S) {
    self.required_layers.insert(name.into());
  }

  pub fn want_extension<S: Into<CString>>(&mut self, name: S) {
    self.wanted_extensions.insert(name.into());
  }

  pub fn require_extension<S: Into<CString>>(&mut self, name: S) {
    self.required_extensions.insert(name.into());
  }
}

#[derive(Error, Debug)]
pub enum InstanceCreateError {
  #[error("Failed to enumerate instance layer properties")]
  EnumerateLayerFail(#[source] VkError),
  #[error("One or more required instance layers are missing: {0:?}")]
  RequiredLayersMissing(Vec<CString>),
  #[error("Failed to enumerate instance extension properties")]
  EnumerateExtensionFail(#[source] VkError),
  #[error("One or more required instance extensions are missing: {0:?}")]
  RequiredExtensionsMissing(Vec<CString>),
  #[error("Failed to create Vulkan instance")]
  InstanceCreateFail(#[from] InstanceError),
  #[error("Failed to create Vulkan debug report callback")]
  DebugReportCallbackCreateFail(#[source] VkError),
}

impl Instance {
  pub fn new(
    entry: Entry,
    application_name: Option<&CStr>,
    application_version: Option<VkVersion>,
    engine_name: Option<&CStr>,
    engine_version: Option<VkVersion>,
    max_vulkan_api_version: Option<VkVersion>,
    features_query: InstanceFeaturesQuery,
  ) -> Result<Self, InstanceCreateError> {
    use InstanceCreateError::*;
    use crate::util::get_enabled_or_missing;
    use std::ptr;
    use vk::{ApplicationInfo, InstanceCreateInfo};

    let application_info = ApplicationInfo {
      p_application_name: application_name.map(|n| n.as_ptr()).unwrap_or(ptr::null()),
      application_version: application_version.unwrap_or_default().into(),
      p_engine_name: engine_name.map(|n| n.as_ptr()).unwrap_or(ptr::null()),
      engine_version: engine_version.unwrap_or_default().into(),
      api_version: max_vulkan_api_version.unwrap_or_default().into(),
      ..ApplicationInfo::default()
    };

    let InstanceFeaturesQuery {
      wanted_layers,
      required_layers,
      wanted_extensions,
      required_extensions
    } = features_query;
    let (enabled_layers, enabled_layers_raw) = {
      let available = entry.enumerate_instance_layer_properties()
        .map_err(|e| EnumerateLayerFail(e))?
        .into_iter()
        .map(|p| unsafe { CStr::from_ptr(p.layer_name.as_ptr()) }.to_owned());
      get_enabled_or_missing(available, &wanted_layers, &required_layers)
        .map_err(|e| RequiredLayersMissing(e.0))?
    };
    let (enabled_extensions, enabled_extensions_raw) = {
      let available = entry.enumerate_instance_extension_properties()
        .map_err(|e| EnumerateExtensionFail(e))?
        .into_iter()
        .map(|p| unsafe { CStr::from_ptr(p.extension_name.as_ptr()) }.to_owned());
      get_enabled_or_missing(available, &wanted_extensions, &required_extensions)
        .map_err(|e| RequiredExtensionsMissing(e.0))?
    };

    let create_info = InstanceCreateInfo::builder()
      .application_info(&application_info)
      .enabled_layer_names(&enabled_layers_raw)
      .enabled_extension_names(&enabled_extensions_raw);

    let instance = unsafe { entry.create_instance(&create_info, None) }
      .map_err(|e| InstanceCreateFail(e))?;
    let features = InstanceFeatures::new(enabled_layers, enabled_extensions);

    Ok(Self { entry, wrapped: instance, features })
  }

  pub unsafe fn destroy(&mut self) {
    trace!("Destroying instance {:?}", self.wrapped.handle());
    self.wrapped.destroy_instance(None);
  }
}

// Implementations

impl Deref for Instance {
  type Target = VkInstance;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.wrapped }
}
