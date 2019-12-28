use std::collections::HashSet;
use std::ffi::{CStr, CString, IntoStringError, NulError};

use ash::{Entry, InstanceError, LoadingError};
use ash::version::EntryV1_0;
use ash::vk::{ApplicationInfo, InstanceCreateInfo, Result as AshError};
use thiserror::Error;

use crate::instance::VkInstance;
use crate::util::cstring_from_str;
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
pub struct InstanceLayerQuery {
  pub(crate) wanted: Vec<String>,
  pub(crate) required: Vec<String>,
}

impl InstanceLayerQuery {
  pub fn new() -> Self {
    Self { wanted: Vec::new(), required: Vec::new() }
  }

  pub fn want<S: Into<String>>(&mut self, name: S) {
    self.wanted.push(name.into());
  }

  pub fn require<S: Into<String>>(&mut self, name: S) {
    self.required.push(name.into());
  }

  pub const VALIDATION_LAYER_NAME: &'static str = "VK_LAYER_LUNARG_standard_validation";

  pub fn want_validation_layer(&mut self) {
    self.want(Self::VALIDATION_LAYER_NAME)
  }

  pub fn require_validation_layer(&mut self) {
    self.require(Self::VALIDATION_LAYER_NAME)
  }
}

impl Default for InstanceLayerQuery {
  fn default() -> Self {
    let mut query = Self::new();
    if cfg!(debug_assertions) {
      query.require_validation_layer();
    }
    query
  }
}


pub struct InstanceExtensionQuery {
  pub(crate) wanted: Vec<String>,
  pub(crate) required: Vec<String>,
}

impl InstanceExtensionQuery {
  pub fn new() -> Self {
    Self { wanted: Vec::new(), required: Vec::new() }
  }

  pub fn want<S: Into<String>>(&mut self, name: S) {
    self.wanted.push(name.into());
  }

  pub fn require<S: Into<String>>(&mut self, name: S) {
    self.required.push(name.into());
  }

  pub const DEBUG_REPORT_EXTENSION_NAME: &'static str = "VK_EXT_debug_report";

  pub fn want_debug_report_extension(&mut self) {
    self.want(Self::DEBUG_REPORT_EXTENSION_NAME)
  }

  pub fn require_debug_report_extension(&mut self) {
    self.require(Self::DEBUG_REPORT_EXTENSION_NAME)
  }
}

impl Default for InstanceExtensionQuery {
  fn default() -> Self {
    let mut query = Self::new();
    if cfg!(debug_assertions) {
      query.require_debug_report_extension();
    }
    query
  }
}


#[derive(Error, Debug)]
pub enum InstanceCreateError {
  #[error("Failed to convert application name into a C-String")]
  ApplicationNameConvertFail(#[source] NulError),
  #[error("Failed to convert engine name into a C-String")]
  EngineNameConvertFail(#[source] NulError),
  #[error("Failed to enumerate instance layer properties")]
  EnumerateLayerFail(#[source] AshError),
  #[error("Failed to convert a layer name into a C-String")]
  LayerNameConvertToCStringFail(#[source] NulError),
  #[error("Failed to convert a layer name into a String")]
  LayerNameConvertToStringFail(#[source] IntoStringError),
  #[error("One or more required instance layers are missing: {0:?}")]
  RequiredLayersMissing(Vec<String>),
  #[error("Failed to convert an extension name into a C-String")]
  ExtensionNameConvertToCStringFail(#[source] NulError),
  #[error("Failed to convert an extension name into a String")]
  ExtensionNameConvertToStringFail(#[source] IntoStringError),
  #[error("Failed to enumerate instance extension properties")]
  EnumerateExtensionFail(#[source] AshError),
  #[error("One or more required instance extensions are missing: {0:?}")]
  RequiredExtensionsMissing(Vec<String>),
  #[error("Failed to create Vulkan instance")]
  InstanceCreateFail(#[from] InstanceError)
}

impl VkEntry {
  pub fn create_instance(
    &self,
    application_name: Option<&str>,
    application_version: Option<VkVersion>,
    engine_name: Option<&str>,
    engine_version: Option<VkVersion>,
    max_vulkan_api_version: Option<VkVersion>,
    layer_query: Option<InstanceLayerQuery>,
    extension_query: Option<InstanceExtensionQuery>,
  ) -> Result<VkInstance, InstanceCreateError> {
    use InstanceCreateError::*;

    let (_, application_name_ptr) = cstring_from_str(application_name).map_err(|e| ApplicationNameConvertFail(e))?;
    let (_, engine_name_ptr) = cstring_from_str(engine_name).map_err(|e| ApplicationNameConvertFail(e))?;
    let application_info = ApplicationInfo {
      p_application_name: application_name_ptr,
      application_version: application_version.unwrap_or_default().into(),
      p_engine_name: engine_name_ptr,
      engine_version: engine_version.unwrap_or_default().into(),
      api_version: max_vulkan_api_version.unwrap_or_default().into(),
      ..ApplicationInfo::default()
    };

    let enabled_layers = {
      let InstanceLayerQuery { wanted, required } = layer_query.unwrap_or_default();
      let wanted: Result<HashSet<_>, _> = wanted.into_iter().map(|s| CString::new(s).map_err(|e| LayerNameConvertToCStringFail(e))).collect();
      let wanted = wanted?;
      let required: Result<HashSet<_>, _> = required.into_iter().map(|s| CString::new(s).map_err(|e| LayerNameConvertToCStringFail(e))).collect();
      let required = required?;
      let available: HashSet<_> = self.entry.enumerate_instance_layer_properties()
        .map_err(|e| EnumerateLayerFail(e))?
        .into_iter()
        .map(|p| unsafe { CStr::from_ptr(p.layer_name.as_ptr()) }.to_owned())
        .collect();
      let missing: Result<Vec<_>, _> = required.difference(&available).map(|s| s.clone().into_string().map_err(|e| LayerNameConvertToStringFail(e))).collect();
      let missing = missing?;
      if !missing.is_empty() {
        return Err(RequiredLayersMissing(missing));
      }
      let enabled: Vec<_> = available.union(&wanted.union(&required).cloned().collect()).cloned().collect();
      enabled
    };
    let enabled_layers_raw: Vec<_> = enabled_layers.iter().map(|n| n.as_ptr()).collect();

    let enabled_extensions = {
      let InstanceExtensionQuery { wanted, required } = extension_query.unwrap_or_default();
      let wanted: Result<HashSet<_>, _> = wanted.into_iter().map(|s| CString::new(s).map_err(|e| ExtensionNameConvertToCStringFail(e))).collect();
      let wanted = wanted?;
      let required: Result<HashSet<_>, _> = required.into_iter().map(|s| CString::new(s).map_err(|e| ExtensionNameConvertToCStringFail(e))).collect();
      let required = required?;
      let available: HashSet<_> = self.entry.enumerate_instance_extension_properties()
        .map_err(|e| EnumerateLayerFail(e))?
        .into_iter()
        .map(|p| unsafe { CStr::from_ptr(p.extension_name.as_ptr()) }.to_owned())
        .collect();
      let missing: Result<Vec<_>, _> = required.difference(&available).map(|s| s.clone().into_string().map_err(|e| ExtensionNameConvertToStringFail(e))).collect();
      let missing = missing?;
      if !missing.is_empty() {
        return Err(RequiredExtensionsMissing(missing));
      }
      let enabled: Vec<_> = available.union(&wanted.union(&required).cloned().collect()).cloned().collect();
      enabled
    };
    let enabled_extensions_raw: Vec<_> = enabled_extensions.iter().map(|n| n.as_ptr()).collect();

    let create_info = InstanceCreateInfo::builder()
      .application_info(&application_info)
      .enabled_layer_names(&enabled_layers_raw)
      .enabled_extension_names(&enabled_extensions_raw)
      ;

    let instance = unsafe { self.entry.create_instance(&create_info, None) }
      .map_err(|e| InstanceCreateFail(e))?;
    Ok(VkInstance::new(instance))
  }
}
