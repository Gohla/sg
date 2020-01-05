//! # Safety
//!
//! Safe usage prohibits:
//!
//! * Calling methods or getting fields of [`Device`] when its creating [`Instance`] has been destroyed.
//! * Calling methods or getting fields of [`Device`] after it has been [destroyed](Device::destroy).
//!
//! # Destruction
//!
//! A [`Device`] must be manually destroyed with [`Device::destroy`].

use std::borrow::Borrow;
use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::ops::Deref;

use ash::{
  Device as VkDevice,
  version::DeviceV1_0,
  version::InstanceV1_0,
  vk::{self, PhysicalDevice as VkPhysicalDevice, PhysicalDeviceFeatures, QueueFlags, Result as VkError},
  vk::Queue
};
use log::debug;
use thiserror::Error;

use crate::instance::Instance;
use crate::instance::surface_extension::Surface;

pub mod swapchain_extension;

// Wrapper

pub struct Device {
  pub wrapped: VkDevice,
  pub physical_device: VkPhysicalDevice,
  pub graphics_queue_index: u32,
  pub graphics_queue: Queue,
  pub present_queue_index: u32,
  pub present_queue: Queue,
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

// Creation and destruction

#[derive(Default, Debug)]
pub struct DeviceFeaturesQuery {
  wanted_extensions: HashSet<CString>,
  required_extensions: HashSet<CString>,
  required_features: PhysicalDeviceFeatures,
}

impl DeviceFeaturesQuery {
  pub fn new() -> Self { Self::default() }

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
TODO: provide a more sophisticated way to select a suitable device and queues, while also creating a user-defined
      struct that contains the requested configuration.
*/

#[derive(Error, Debug)]
pub enum PhysicalDeviceCreateError {
  #[error("Failed to enumerate physical devices: {0:?}")]
  EnumeratePhysicalDevicesFail(#[source] VkError),
  #[error("Failed to enumerate extension properties of physical device: {0:?}")]
  EnumerateExtensionPropertiesFail(#[source] VkError),
  #[error("Failed to create a device: {0:?}")]
  DeviceCreateFail(#[source] VkError),
  #[error("Failed to find a suitable physical device")]
  NoSuitablePhysicalDeviceFound,
}

impl Device {
  pub fn new(
    instance: &Instance,
    features_query: DeviceFeaturesQuery,
    required_surface_support: Option<&Surface>,
  ) -> Result<Self, PhysicalDeviceCreateError> {
    use PhysicalDeviceCreateError::*;
    use crate::util::get_enabled_or_missing;
    use vk::DeviceQueueCreateInfo;
    use vk::DeviceCreateInfo;

    let DeviceFeaturesQuery {
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

      let (graphics_queue_index, present_queue_index) = {
        let mut graphics = None;
        let mut present = None;
        let queue_families_properties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        for (index, queue_family_properties) in queue_families_properties.into_iter().enumerate() {
          if graphics.is_none() && queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS) {
            graphics = Some(index as u32);
          }
          if present.is_none() {
            if let Some(surface) = required_surface_support {
              if !unsafe { surface.loader.get_physical_device_surface_support(physical_device, index as u32, surface.wrapped) } {
                continue;
              }
            }
            present = Some(index as u32);
          }
        }
        // TODO: don't assume that we're always rendering to a display
        if let (Some(graphics), Some(present)) = (graphics, present) {
          (graphics, present)
        } else {
          continue;
        }
      };

      let queue_priorities = [1.0]; // TODO: don't assume we only want one queue.
      let queue_create_infos = {
        let mut infos = Vec::new();
        infos.push(DeviceQueueCreateInfo::builder()
          .queue_family_index(graphics_queue_index)
          .queue_priorities(&queue_priorities)
          .build()
        );
        if present_queue_index != graphics_queue_index {
          infos.push(DeviceQueueCreateInfo::builder()
            .queue_family_index(present_queue_index)
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
      // CORRECTNESS: `queue_priorities` is taken by pointer but is alive until `create_device` is called.
      let device = unsafe { instance.create_device(physical_device, &create_info, None) }
        .map_err(|e| DeviceCreateFail(e))?;
      debug!("Created device {:?}", device.handle());
      let graphics_queue = unsafe { device.get_device_queue(graphics_queue_index, 0) };
      let present_queue = unsafe { device.get_device_queue(present_queue_index, 0) };
      let features = DeviceFeatures::new(enabled_extensions, required_features);
      return Ok(Self {
        wrapped: device,
        physical_device,
        graphics_queue_index,
        graphics_queue,
        present_queue_index,
        present_queue,
        features,
      });
    }
    Err(NoSuitablePhysicalDeviceFound)
  }

  pub unsafe fn destroy(&mut self) {
    debug!("Destroying device {:?}", self.wrapped.handle());
    self.wrapped.destroy_device(None);
  }
}

// Implementations

impl Deref for Device {
  type Target = VkDevice;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.wrapped }
}
