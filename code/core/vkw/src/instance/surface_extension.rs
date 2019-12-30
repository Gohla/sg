use std::ffi::CStr;

use ash::extensions::khr::Surface as VkSurface;
use ash::vk::{self, Result as VkError, SurfaceKHR};
use byte_strings::c_str;
use raw_window_handle::RawWindowHandle;
use thiserror::Error;

use crate::entry::Entry;
use crate::instance::{Instance, InstanceFeatures, InstanceFeaturesQuery};

// Extension names

pub const SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_surface");

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_xlib_surface");
#[cfg(target_os = "macos")]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_MVK_macos_surface");
#[cfg(all(windows))]
pub const PLATFORM_SURFACE_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_win32_surface");

// Wrapper

pub struct Surface {
  loader: VkSurface,
  surface: SurfaceKHR,
}

// Creation

#[derive(Error, Debug)]
pub enum SurfaceCreateError {
  #[error("Got a window handle that does not match with the current platform")]
  WindowHandleMismatch,
  #[error("Failed to create Vulkan surface")]
  SurfaceCreateFail(#[source] VkError)
}

impl Surface {
  pub fn new(entry: &Entry, instance: &Instance, window: RawWindowHandle) -> Result<Self, SurfaceCreateError> {
    let loader = VkSurface::new(&entry.wrapped, &instance.wrapped);
    let surface = Self::create_surface(entry, instance, window)?;
    Ok(Self { loader, surface })
  }

  fn create_surface(entry: &Entry, instance: &Instance, window: RawWindowHandle) -> Result<SurfaceKHR, SurfaceCreateError> {
    use SurfaceCreateError::*;
    use std::ptr;
    use std::os::raw::c_void;

    #[cfg(target_os = "windows")] {
      use ash::extensions::khr::Win32Surface;

      if let RawWindowHandle::Windows(handle) = window {
        let create_info = vk::Win32SurfaceCreateInfoKHR {
          s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
          p_next: ptr::null(),
          flags: Default::default(),
          hinstance: handle.hinstance,
          hwnd: handle.hwnd as *const c_void,
        };
        let loader = Win32Surface::new(&entry.wrapped, &instance.wrapped);
        let surface = unsafe { loader.create_win32_surface(&create_info, None) }
          .map_err(|e| SurfaceCreateFail(e))?;
        Ok(surface)
      } else {
        Err(WindowHandleMismatch)
      }
    }

    // TODO: support other platforms
  }
}

// Implementations

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

impl Drop for Surface {
  fn drop(&mut self) {
    unsafe { self.loader.destroy_surface(self.surface, None); }
  }
}
