use std::borrow::Borrow;
use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::ops::Deref;

use ash::{Instance as VkInstance, InstanceError};
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk::{self, Result as VkError};
use thiserror::Error;

use crate::entry::Entry;
use crate::version::VkVersion;

pub mod validation_layer;
pub mod debug_report_extension;
pub mod surface_extension;

// Wrapper

pub struct Instance<'e> {
  pub entry: &'e Entry,
  pub wrapped: VkInstance,
  pub features: InstanceFeatures,
}

#[derive(Debug)]
pub struct InstanceFeatures {
  enabled_layers: HashSet<CString>,
  enabled_extensions: HashSet<CString>,
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

// Creation

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

impl<'e> Instance<'e> {
  pub fn new(
    entry: &'e Entry,
    application_name: Option<&CStr>,
    application_version: Option<VkVersion>,
    engine_name: Option<&CStr>,
    engine_version: Option<VkVersion>,
    max_vulkan_api_version: Option<VkVersion>,
    features_query: InstanceFeaturesQuery,
  ) -> Result<Self, InstanceCreateError> {
    use InstanceCreateError::*;
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

    let enabled_layers = {
      let available: HashSet<_> = entry.enumerate_instance_layer_properties()
        .map_err(|e| EnumerateLayerFail(e))?
        .into_iter()
        .map(|p| unsafe { CStr::from_ptr(p.layer_name.as_ptr()) }.to_owned())
        .collect();
      let missing: Vec<_> = required_layers.difference(&available).cloned().collect();
      if !missing.is_empty() {
        return Err(RequiredLayersMissing(missing));
      }
      let enabled: HashSet<_> = available.intersection(&wanted_layers.union(&required_layers).cloned().collect()).cloned().collect();
      enabled
    };
    let enabled_layers_raw: Vec<_> = enabled_layers.iter().map(|n| n.as_ptr()).collect();

    let enabled_extensions = {
      let available: HashSet<_> = entry.enumerate_instance_extension_properties()
        .map_err(|e| EnumerateLayerFail(e))?
        .into_iter()
        .map(|p| unsafe { CStr::from_ptr(p.extension_name.as_ptr()) }.to_owned())
        .collect();
      let missing: Vec<_> = required_extensions.difference(&available).cloned().collect();
      if !missing.is_empty() {
        return Err(RequiredExtensionsMissing(missing));
      }
      let enabled: HashSet<_> = available.intersection(&wanted_extensions.union(&required_extensions).cloned().collect()).cloned().collect();
      enabled
    };
    let enabled_extensions_raw: Vec<_> = enabled_extensions.iter().map(|n| n.as_ptr()).collect();

    let create_info = InstanceCreateInfo::builder()
      .application_info(&application_info)
      .enabled_layer_names(&enabled_layers_raw)
      .enabled_extension_names(&enabled_extensions_raw);

    let instance = unsafe { entry.create_instance(&create_info, None) }
      .map_err(|e| InstanceCreateFail(e))?;
    let instance_features = InstanceFeatures::new(enabled_layers, enabled_extensions);

    Ok(Self { entry, wrapped: instance, features: instance_features })
  }
}

// Implementations

impl<'e> Deref for Instance<'e> {
  type Target = VkInstance;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.wrapped }
}

impl<'e> Drop for Instance<'e> {
  fn drop(&mut self) {
    unsafe { self.wrapped.destroy_instance(None); }
  }
}
