use std::ffi::CStr;
use std::ops::Deref;

use ash::extensions::khr::Swapchain as VkSwapchainLoader;
use ash::vk::{self, Extent2D, ImageView, PresentModeKHR, Result as VkError, SharingMode, SurfaceFormatKHR, SurfaceTransformFlagsKHR, SwapchainKHR};
use byte_strings::c_str;
use thiserror::Error;

use crate::device::{Device, DeviceFeatures, DeviceFeaturesQuery};
use crate::image::view::ImageViewCreateError;
use crate::instance::Instance;
use crate::instance::surface_extension::{Surface, SurfaceFormatError};

// Wrapper

pub struct SwapchainLoader {
  pub wrapped: VkSwapchainLoader,
}

pub struct Swapchain<'a> {
  pub loader: &'a SwapchainLoader,
  pub device: &'a Device<'a>,
  pub wrapped: SwapchainKHR,
  pub image_views: Vec<ImageView>,
  pub features: SwapchainFeatures,
}

#[derive(Debug)]
pub struct SwapchainFeatures {
  pub min_image_count: u32,
  pub surface_format: SurfaceFormatKHR,
  pub sharing_mode: SharingMode,
  pub pre_transform: SurfaceTransformFlagsKHR,
  pub present_mode: PresentModeKHR,
  pub extent: Extent2D,
}

// Creation

impl SwapchainLoader {
  pub fn new(instance: &Instance, device: &Device) -> SwapchainLoader {
    let loader = VkSwapchainLoader::new(&instance.wrapped, &device.wrapped);
    Self { wrapped: loader }
  }
}

#[derive(Default, Debug)]
pub struct SwapchainFeaturesQuery {
  wanted_image_count: u32,
  wanted_present_modes_ord: Vec<PresentModeKHR>,
}

impl SwapchainFeaturesQuery {
  pub fn new() -> Self { Self::default() }


  pub fn want_image_count(&mut self, image_count: u32) { self.wanted_image_count = image_count; }

  pub fn want_present_mode(&mut self, present_modes_ord: Vec<PresentModeKHR>) {
    self.wanted_present_modes_ord = present_modes_ord;
  }
}

#[derive(Error, Debug)]
pub enum SwapchainCreateError {
  #[error("Failed to get surface format")]
  SurfaceFormatFail(#[from] SurfaceFormatError),
  #[error("Failed to get surface capabilities")]
  SurfaceCapabilitiesFail(#[source] VkError),
  #[error("Failed to get surface present modes")]
  SurfacePresentModesFail(#[source] VkError),
  #[error("Failed to find present mode")]
  NoPresentModeFound(),
  #[error("Failed to create swapchain")]
  SwapchainCreateFail(#[source] VkError),
  #[error("Failed to get swapchain images")]
  SwapchainImagesFail(#[source] VkError),
  #[error("Failed to create image views for swapchain images")]
  SwapchainImageViewsCreateFail(#[from] ImageViewCreateError),
}

impl<'a> Swapchain<'a> {
  pub fn new(
    loader: &'a SwapchainLoader,
    device: &'a Device,
    surface: &Surface,
    features_query: SwapchainFeaturesQuery,
    surface_extent: vk::Extent2D,
    old_swapchain: Option<Swapchain>
  ) -> Result<Self, SwapchainCreateError> {
    use SwapchainCreateError::*;
    use std::cmp::{min, max};

    let capabilities = surface.get_capabilities(device.physical_device)
      .map_err(|e| SurfaceCapabilitiesFail(e))?;
    let min_image_count = match capabilities.max_image_count {
      0 => max(capabilities.min_image_count, features_query.wanted_image_count),
      max_image_count => max(capabilities.min_image_count, min(features_query.wanted_image_count, max_image_count)),
    };
    let surface_format = surface.get_suitable_surface_format(device.physical_device)?;
    let extent = match (capabilities.current_extent.width, capabilities.current_extent.height) {
      (std::u32::MAX, std::u32::MAX) => surface_extent,
      _ => capabilities.current_extent,
    };
    let (sharing_mode, queue_family_indices) = {
      let (graphics, present) = (device.graphics_queue_index, device.present_queue_index);
      if graphics == present {
        (SharingMode::EXCLUSIVE, vec![])
      } else {
        (SharingMode::CONCURRENT, vec![graphics, present])
      }
    };
    let pre_transform = if capabilities.supported_transforms.contains(SurfaceTransformFlagsKHR::IDENTITY) {
      SurfaceTransformFlagsKHR::IDENTITY
    } else {
      capabilities.current_transform
    };
    let present_mode = {
      let available_present_modes = surface.get_present_modes(device.physical_device)
        .map_err(|e| SurfacePresentModesFail(e))?;
      Self::select_present_mode(available_present_modes, features_query.wanted_present_modes_ord)
        .ok_or(NoPresentModeFound())?
    };

    let mut create_info = vk::SwapchainCreateInfoKHR::builder()
      .surface(surface.wrapped)
      .min_image_count(min_image_count)
      .image_color_space(surface_format.color_space)
      .image_format(surface_format.format)
      .image_extent(extent)
      .image_array_layers(1)
      .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
      .image_sharing_mode(sharing_mode)
      .queue_family_indices(&queue_family_indices)
      .pre_transform(pre_transform)
      .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
      .present_mode(present_mode)
      .clipped(true)
      ;
    if let Some(old_swapchain) = old_swapchain {
      create_info = create_info.old_swapchain(old_swapchain.wrapped);
    }
    let swapchain = unsafe { loader.create_swapchain(&create_info, None) }
      .map_err(|e| SwapchainCreateFail(e))?;

    let images = unsafe { loader.get_swapchain_images(swapchain) }
      .map_err(|e| SwapchainImagesFail(e))?;
    let image_views = {
      let image_views: Result<Vec<_>, _> = images
        .into_iter()
        .map(|image| {
          device.create_image_view(image, surface_format.format, vk::ImageViewType::TYPE_2D, vk::ImageAspectFlags::COLOR, 1)
        })
        .collect();
      image_views?
    };

    let features = SwapchainFeatures {
      min_image_count,
      surface_format,
      sharing_mode,
      pre_transform,
      present_mode,
      extent
    };

    Ok(Self { loader, device, wrapped: swapchain, image_views, features })
  }

  fn select_present_mode(available_present_modes: Vec<PresentModeKHR>, wanted_present_modes_ord: Vec<PresentModeKHR>) -> Option<PresentModeKHR> {
    for wanted_mode in &wanted_present_modes_ord {
      for available_mode in &available_present_modes {
        if available_mode == wanted_mode {
          return Some(*available_mode);
        }
      }
    }
    if !available_present_modes.is_empty() {
      Some(available_present_modes[0])// No preference, return first present mode.
    } else {
      None // No present mode available.
    }
  }
}

// API

impl DeviceFeatures {
  pub fn is_swapchain_extension_enabled(&self) -> bool {
    self.is_extension_enabled(self::SWAPCHAIN_EXTENSION_NAME)
  }
}

impl DeviceFeaturesQuery {
  pub fn want_swapchain_extension(&mut self) {
    self.want_extension(self::SWAPCHAIN_EXTENSION_NAME);
  }

  pub fn require_swapchain_extension(&mut self) {
    self.require_extension(self::SWAPCHAIN_EXTENSION_NAME);
  }
}

// Implementations

impl Deref for SwapchainLoader {
  type Target = VkSwapchainLoader;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.wrapped }
}

impl Deref for Swapchain<'_> {
  type Target = SwapchainKHR;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.wrapped }
}

impl Drop for Swapchain<'_> {
  fn drop(&mut self) {
    unsafe {
      for image_view in &self.image_views {
        self.device.destroy_image_view(*image_view);
      }
      self.loader.destroy_swapchain(self.wrapped, None);
    }
  }
}

// Extension name

pub const SWAPCHAIN_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_swapchain");
