use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;

use ash::{Entry, InstanceError, LoadingError, vk};
use ash::extensions::ext::DebugReport;
use ash::version::EntryV1_0;
use ash::vk::{ApplicationInfo, DebugReportFlagsEXT, InstanceCreateInfo, Result as VulkanError};
use thiserror::Error;

use crate::instance::{InstanceFeatures, VkInstance};
use crate::instance;
use crate::version::VkVersion;

pub struct VkEntry {
  pub entry: Entry,
}

#[derive(Error, Debug)]
pub enum EntryCreateError {
  #[error("Failed to load Vulkan library")]
  LoadError(#[from] LoadingError),
}

impl VkEntry {
  pub fn new() -> Result<Self, EntryCreateError> {
    let entry = Entry::new()?;
    Ok(Self { entry })
  }

  pub fn instance_version(&self) -> Option<VkVersion> {
    match self.entry.try_enumerate_instance_version() {
      Ok(Some(version)) => Some(version.into()),
      Ok(None) => Some(VkVersion::default()),
      Err(_) => None,
    }
  }
}


#[derive(Debug)]
pub struct InstanceFeaturesQuery {
  wanted_layers: HashSet<CString>,
  required_layers: HashSet<CString>,
  wanted_extensions: HashSet<CString>,
  required_extensions: HashSet<CString>,
}

impl InstanceFeaturesQuery {
  pub fn new() -> Self {
    Self {
      wanted_layers: HashSet::new(),
      required_layers: HashSet::new(),
      wanted_extensions: HashSet::new(),
      required_extensions: HashSet::new()
    }
  }


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


  pub fn want_validation(&mut self) {
    self.want_layer(instance::VALIDATION_LAYER_NAME);
    self.want_extension(instance::DEBUG_REPORT_EXTENSION_NAME);
  }

  pub fn require_validation(&mut self) {
    self.require_layer(instance::VALIDATION_LAYER_NAME);
    self.require_extension(instance::DEBUG_REPORT_EXTENSION_NAME);
  }

  pub fn want_surface(&mut self) {
    self.want_extension(instance::SURFACE_EXTENSION_NAME);
    self.want_extension(instance::PLATFORM_SURFACE_EXTENSION_NAME);
  }

  pub fn require_surface(&mut self) {
    self.require_extension(instance::SURFACE_EXTENSION_NAME);
    self.require_extension(instance::PLATFORM_SURFACE_EXTENSION_NAME);
  }
}

impl Default for InstanceFeaturesQuery {
  fn default() -> Self {
    let mut query = Self::new();
    if cfg!(debug_assertions) {
      query.require_validation();
    }
    query.require_surface();
    query
  }
}


#[derive(Error, Debug)]
pub enum InstanceCreateError {
  #[error("Failed to enumerate instance layer properties")]
  EnumerateLayerFail(#[source] VulkanError),
  #[error("One or more required instance layers are missing: {0:?}")]
  RequiredLayersMissing(Vec<CString>),
  #[error("Failed to enumerate instance extension properties")]
  EnumerateExtensionFail(#[source] VulkanError),
  #[error("One or more required instance extensions are missing: {0:?}")]
  RequiredExtensionsMissing(Vec<CString>),
  #[error("Failed to create Vulkan instance")]
  InstanceCreateFail(#[from] InstanceError),
  #[error("Failed to create Vulkan debug report callback")]
  DebugReportCallbackCreateFail(#[source] VulkanError),
}

impl VkEntry {
  pub fn create_instance(
    &self,
    application_name: Option<&CStr>,
    application_version: Option<VkVersion>,
    engine_name: Option<&CStr>,
    engine_version: Option<VkVersion>,
    max_vulkan_api_version: Option<VkVersion>,
    feature_query: Option<InstanceFeaturesQuery>
  ) -> Result<VkInstance, InstanceCreateError> {
    use InstanceCreateError::*;

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
    } = feature_query.unwrap_or_default();

    let enabled_layers = {
      let available: HashSet<_> = self.entry.enumerate_instance_layer_properties()
        .map_err(|e| EnumerateLayerFail(e))?
        .into_iter()
        .map(|p| unsafe { CStr::from_ptr(p.layer_name.as_ptr()) }.to_owned())
        .collect();
      let missing: Vec<_> = required_layers.difference(&available).cloned().collect();
      if !missing.is_empty() {
        return Err(RequiredLayersMissing(missing));
      }
      let enabled: HashSet<_> = available.union(&wanted_layers.union(&required_layers).cloned().collect()).cloned().collect();
      enabled
    };
    let enabled_layers_raw: Vec<_> = enabled_layers.iter().map(|n| n.as_ptr()).collect();

    let enabled_extensions = {
      let available: HashSet<_> = self.entry.enumerate_instance_extension_properties()
        .map_err(|e| EnumerateLayerFail(e))?
        .into_iter()
        .map(|p| unsafe { CStr::from_ptr(p.extension_name.as_ptr()) }.to_owned())
        .collect();
      let missing: Vec<_> = required_extensions.difference(&available).cloned().collect();
      if !missing.is_empty() {
        return Err(RequiredExtensionsMissing(missing));
      }
      let enabled: HashSet<_> = available.union(&wanted_extensions.union(&required_extensions).cloned().collect()).cloned().collect();
      enabled
    };
    let enabled_extensions_raw: Vec<_> = enabled_extensions.iter().map(|n| n.as_ptr()).collect();

    let create_info = InstanceCreateInfo::builder()
      .application_info(&application_info)
      .enabled_layer_names(&enabled_layers_raw)
      .enabled_extension_names(&enabled_extensions_raw);

    let instance = unsafe { self.entry.create_instance(&create_info, None) }
      .map_err(|e| InstanceCreateFail(e))?;
    let instance_features = InstanceFeatures::new(enabled_layers, enabled_extensions);

    let debug_reporter = if instance_features.is_debug_report_extension_enabled() {
      let info = vk::DebugReportCallbackCreateInfoEXT::builder()
        .flags(DebugReportFlagsEXT::ERROR | DebugReportFlagsEXT::WARNING | DebugReportFlagsEXT::PERFORMANCE_WARNING | DebugReportFlagsEXT::INFORMATION | DebugReportFlagsEXT::DEBUG)
        .pfn_callback(Some(vulkan_debug_callback));
      let loader = DebugReport::new(&self.entry, &instance);
      let callback = unsafe { loader.create_debug_report_callback(&info, None) }
        .map_err(|e| DebugReportCallbackCreateFail(e))?;
      Some(DebugReporter::new(loader, callback))
    } else { None };

    Ok(VkInstance::new(&self, instance, instance_features, debug_reporter))
  }
}

pub(crate) struct DebugReporter {
  loader: DebugReport,
  callback: vk::DebugReportCallbackEXT,
}

impl DebugReporter {
  fn new(loader: DebugReport, callback: vk::DebugReportCallbackEXT) -> Self { Self { loader, callback } }

  pub(crate) fn destroy(&self) {
    unsafe { self.loader.destroy_debug_report_callback(self.callback, None) };
  }
}

unsafe extern "system" fn vulkan_debug_callback(
  flags: DebugReportFlagsEXT,
  _object_type: vk::DebugReportObjectTypeEXT,
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
  //dbg!(level);
  log_macro!(level, "{:?}", CStr::from_ptr(p_message));
  vk::FALSE
}
