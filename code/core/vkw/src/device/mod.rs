// Wrapper

use std::borrow::Borrow;
use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::ops::Deref;

use ash::{
  Device as VkDevice,
  version::InstanceV1_0,
  vk::{self, PhysicalDevice as VkPhysicalDevice, PhysicalDeviceFeatures, QueueFlags, Result as VkError}
};
use ash::version::DeviceV1_0;
use thiserror::Error;

use crate::instance::Instance;
use crate::instance::surface_extension::Surface;

pub mod swapchain_extension;

// Wrapper

pub struct Device<'e, 'i> {
  pub instance: &'i Instance<'e>,
  pub wrapped: VkDevice,
  pub physical_device: VkPhysicalDevice,
  pub graphics_queue_index: Option<u32>,
  pub present_queue_index: Option<u32>,
  pub compute_queue_index: Option<u32>,
  pub features: DeviceFeatures,
}

#[derive(Debug)]
pub struct DeviceFeatures {
  pub enabled_extensions: HashSet<CString>,
  pub enabled_features: PhysicalDeviceFeatures,
}

impl DeviceFeatures {
  fn new(enabled_extensions: HashSet<CString>, enabled_features: PhysicalDeviceFeatures) -> Self {
    Self { enabled_extensions, enabled_features }
  }


  pub fn is_extension_enabled<B: Borrow<CStr> + ?Sized>(&self, extension_name: &B) -> bool {
    self.enabled_extensions.contains(extension_name.borrow())
  }
}

// Creation

#[derive(Default, Debug)]
pub struct DeviceFeaturesQuery {
  require_graphics_queue: bool,
  require_present_queue: bool,
  require_compute_queue: bool,
  wanted_extensions: HashSet<CString>,
  required_extensions: HashSet<CString>,
  required_features: PhysicalDeviceFeatures,
}

impl<'s> DeviceFeaturesQuery {
  pub fn new() -> Self { Self::default() }


  pub fn require_graphics_queue(&mut self) { self.require_graphics_queue = true; }

  pub fn require_present_queue(&mut self) { self.require_present_queue = true; }

  pub fn require_compute_queue(&mut self) { self.require_compute_queue = true; }

  pub fn want_extension<S: Into<CString>>(&mut self, name: S) {
    self.wanted_extensions.insert(name.into());
  }

  pub fn require_extension<S: Into<CString>>(&mut self, name: S) {
    self.required_extensions.insert(name.into());
  }

  pub fn require_features(&mut self, required_features: PhysicalDeviceFeatures) {
    self.required_features = required_features;
  }
}

/*
TODO: provide a more sophisticated way to select a suitable device, while also creating a (user-defined) struct that
      contains the requested queues.
*/

#[derive(Error, Debug)]
pub enum PhysicalDeviceCreateError {
  #[error("Failed to enumerate Vulkan physical devices")]
  EnumeratePhysicalDevicesFail(#[source] VkError),
  #[error("Failed to enumerate extension properties of Vulkan physical device")]
  EnumerateExtensionPropertiesFail(#[source] VkError),
  #[error("Failed to create a Vulkan device")]
  DeviceCreateFail(#[source] VkError),
  #[error("Failed to find a suitable Vulkan physical device")]
  NoSuitablePhysicalDeviceFound,
}

impl<'e, 'i> Device<'e, 'i> {
  pub fn new(
    instance: &'i Instance<'e>,
    features_query: DeviceFeaturesQuery,
    required_surface_support: Option<&Surface>,
  ) -> Result<Self, PhysicalDeviceCreateError> {
    use PhysicalDeviceCreateError::*;
    use crate::util::get_enabled_or_missing;
    use vk::DeviceQueueCreateInfo;
    use vk::DeviceCreateInfo;

    let DeviceFeaturesQuery {
      require_graphics_queue,
      require_present_queue,
      require_compute_queue,
      wanted_extensions,
      required_extensions,
      required_features,
    } = features_query;

    let physical_devices = unsafe { instance.enumerate_physical_devices() }
      .map_err(|e| EnumeratePhysicalDevicesFail(e))?;
    for physical_device in physical_devices {
      let (enabled_extensions, enabled_extensions_raw) = {
        let available = unsafe { instance.enumerate_device_extension_properties(physical_device) }
          .map_err(|e| EnumerateExtensionPropertiesFail(e))?
          .into_iter()
          .map(|p| unsafe { CStr::from_ptr(p.extension_name.as_ptr()) }.to_owned());
        match get_enabled_or_missing(available, &wanted_extensions, &required_extensions) {
          Ok(t) => t,
          Err(_) => continue,
        }
      };

      // TODO: check features

      let (graphics_queue_index, present_queue_index, compute_queue_index) = {
        let mut graphics = None;
        let mut present = None;
        let mut compute = None;
        let queue_families_properties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        for (index, queue_family_properties) in queue_families_properties.into_iter().enumerate() {
          if require_graphics_queue && graphics.is_none() && queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS) {
            graphics = Some(index as u32);
          }
          if require_present_queue && present.is_none() {
            if let Some(surface) = required_surface_support {
              if !unsafe { surface.loader.get_physical_device_surface_support(physical_device, index as u32, surface.wrapped) } {
                continue;
              }
            }
            present = Some(index as u32);
          }
          if require_compute_queue && compute.is_none() && queue_family_properties.queue_flags.contains(QueueFlags::COMPUTE) {
            compute = Some(index as u32);
          }
        }
        if require_graphics_queue && graphics.is_none() { continue; }
        if require_present_queue && present.is_none() { continue; }
        if require_compute_queue && compute.is_none() { continue; }
        (graphics, present, compute)
      };

      let queue_priorities = [1.0];
      let queue_create_infos = {
        let mut infos = Vec::new();
        if let Some(idx) = graphics_queue_index {
          infos.push(DeviceQueueCreateInfo::builder()
            .queue_family_index(idx)
            .queue_priorities(&queue_priorities)
            .build()
          );
        }
        if let Some(idx) = present_queue_index {
          infos.push(DeviceQueueCreateInfo::builder()
            .queue_family_index(idx)
            .queue_priorities(&queue_priorities)
            .build()
          );
        }
        if let Some(idx) = compute_queue_index {
          infos.push(DeviceQueueCreateInfo::builder()
            .queue_family_index(idx)
            .queue_priorities(&queue_priorities)
            .build()
          );
        }
        infos
      };
      let create_info = DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&enabled_extensions_raw)
        .enabled_features(&required_features);
      let device = unsafe { instance.create_device(physical_device, &create_info, None) }
        .map_err(|e| DeviceCreateFail(e))?;
      let features = DeviceFeatures::new(enabled_extensions, required_features);
      return Ok(Self {
        instance,
        wrapped: device,
        physical_device,
        graphics_queue_index,
        present_queue_index,
        compute_queue_index,
        features,
      });
    }
    Err(NoSuitablePhysicalDeviceFound)
  }
}

// Implementations

impl<'e, 'i> Deref for Device<'e, 'i> {
  type Target = VkDevice;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.wrapped }
}

impl<'e, 'i> Drop for Device<'e, 'i> {
  fn drop(&mut self) {
    unsafe {
      self.wrapped.destroy_device(None);
    }
  }
}
