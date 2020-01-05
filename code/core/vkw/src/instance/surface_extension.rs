//! # Safety
//!
//! Safe usage prohibits:
//!
//! * Calling methods or getting fields of [`Surface`] when its creating [`Instance`] has been destroyed.
//! * Calling methods or getting fields of [`Surface`] after it has been [destroyed](Surface::destroy).
//!
//! # Destruction
//!
//! A [`Surface`] must be manually destroyed with [`Surface::destroy`].

use std::ffi::CStr;
use std::ops::Deref;

use ash::extensions::khr::Surface as SurfaceLoader;
use ash::vk::{self, Result as VkError, SurfaceKHR};
use byte_strings::c_str;
use log::debug;
use raw_window_handle::RawWindowHandle;
use thiserror::Error;

use crate::instance::{Instance, InstanceFeatures, InstanceFeaturesQuery};

// Wrapper

pub struct Surface {
  pub loader: SurfaceLoader,
  pub wrapped: SurfaceKHR,
}

// Creation and destruction

#[derive(Error, Debug)]
pub enum SurfaceCreateError {
  #[error("Got a window handle that does not match with the current platform")]
  WindowHandleMismatch,
  #[error("Failed to create surface: {0:?}")]
  SurfaceCreateFail(#[source] VkError)
}

impl Surface {
  pub fn new(instance: &Instance, window: RawWindowHandle) -> Result<Self, SurfaceCreateError> {
    let loader = SurfaceLoader::new(&instance.entry.wrapped, &instance.wrapped);
    debug!("Created surface loader");
    let surface = Self::create_surface(instance, window)?;
    debug!("Created surface {:?}", surface);
    Ok(Self { loader, wrapped: surface })
  }

  pub unsafe fn destroy(&mut self) {
    debug!("Destroying surface {:?}", self.wrapped);
    self.loader.destroy_surface(self.wrapped, None);
  }

  fn create_surface(instance: &Instance, window: RawWindowHandle) -> Result<SurfaceKHR, SurfaceCreateError> {
    use SurfaceCreateError::*;
    use std::os::raw::c_void;

    #[cfg(target_os = "windows")] {
      use ash::extensions::khr::Win32Surface;

      if let RawWindowHandle::Windows(handle) = window {
        let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
          .hinstance(handle.hinstance)
          .hwnd(handle.hwnd as *const c_void)
          ;
        let loader = Win32Surface::new(&instance.entry.wrapped, &instance.wrapped);
        let surface = unsafe { loader.create_win32_surface(&create_info, None) }
          .map_err(|e| SurfaceCreateFail(e))?;
        Ok(surface)
      } else {
        Err(WindowHandleMismatch)
      }
    }

    // TODO: support macOS
    // TODO: support UNIX
  }
}

// API

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

#[derive(Error, Debug)]
pub enum SurfaceFormatError {
  #[error("Failed to get physical device surface formats: {0:?}")]
  PhysicalDeviceSurfaceFormatsFail(#[source] VkError),
  #[error("Failed to find a suitable surface format")]
  NoSuitableSurfaceFormatFound,
}

impl Surface {
  pub unsafe fn get_suitable_surface_format(&self, physical_device: vk::PhysicalDevice) -> Result<vk::SurfaceFormatKHR, SurfaceFormatError> {
    use SurfaceFormatError::*;
    let surface_formats = self.loader.get_physical_device_surface_formats(physical_device, self.wrapped)
      .map_err(|e| PhysicalDeviceSurfaceFormatsFail(e))?;
    for surface_format in surface_formats {
      // TODO: more sophisticated way to select suitable surface format.
      if surface_format.format == vk::Format::B8G8R8A8_UNORM && surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
        return Ok(surface_format);
      }
    }
    Err(NoSuitableSurfaceFormatFound)
  }

  pub unsafe fn get_capabilities(&self, physical_device: vk::PhysicalDevice) -> Result<vk::SurfaceCapabilitiesKHR, VkError> {
    self.loader.get_physical_device_surface_capabilities(physical_device, self.wrapped)
  }

  pub unsafe fn get_present_modes(&self, physical_device: vk::PhysicalDevice) -> Result<Vec<vk::PresentModeKHR>, VkError> {
    self.loader.get_physical_device_surface_present_modes(physical_device, self.wrapped)
  }
}

// Implementations

impl Deref for Surface {
  type Target = SurfaceKHR;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.wrapped }
}

// Extension names

pub const SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_surface");

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_xlib_surface");
#[cfg(target_os = "macos")]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_MVK_macos_surface");
#[cfg(all(windows))]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_win32_surface");
