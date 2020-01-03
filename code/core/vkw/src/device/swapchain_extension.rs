//! # Safety
//!
//! Safe usage prohibits:
//!
//! * Calling methods or getting fields of [`Swapchain`] when its creating [`Instance`], [`Device`], or [`Surface`] has been destroyed.
//! * Calling methods or getting fields of [`Swapchain`] after it has been [destroyed](Swapchain::destroy).
//!
//! # Destruction
//!
//! A [`Swapchain`] must be manually destroyed with [`Swapchain::destroy`].

use std::ffi::CStr;
use std::num::NonZeroU32;
use std::ops::Deref;

use ash::extensions::khr::Swapchain as SwapchainLoader;
use ash::vk::{self, Extent2D, Fence, ImageView, PresentModeKHR, Queue, Result as VkError, Semaphore, SharingMode, SurfaceFormatKHR, SurfaceTransformFlagsKHR, SwapchainKHR, CompositeAlphaFlagsKHR};
use byte_strings::c_str;
use log::debug;
use thiserror::Error;

use crate::device::{Device, DeviceFeatures, DeviceFeaturesQuery};
use crate::image::view::ImageViewCreateError;
use crate::instance::Instance;
use crate::instance::surface_extension::{Surface, SurfaceFormatError};
use crate::timeout::Timeout;

// Wrapper

pub struct Swapchain {
  loader: SwapchainLoader,
  pub wrapped: SwapchainKHR,
  pub image_views: Vec<ImageView>,
  pub extent: Extent2D,
  pub features_query: SwapchainFeaturesQuery,
  pub features: SwapchainFeatures,
}

#[derive(Debug)]
pub struct SwapchainFeatures {
  pub min_image_count: u32,
  pub surface_format: SurfaceFormatKHR,
  pub sharing_mode: SharingMode,
  pub pre_transform: SurfaceTransformFlagsKHR,
  pub present_mode: PresentModeKHR,
}

// Creation and destruction

#[derive(Clone, Debug)]
pub struct SwapchainFeaturesQuery {
  wanted_image_count: NonZeroU32,
  wanted_present_modes_ord: Vec<PresentModeKHR>,
}

impl SwapchainFeaturesQuery {
  pub fn new() -> Self { Self::default() }

  pub fn want_image_count(&mut self, image_count: NonZeroU32) { self.wanted_image_count = image_count; }

  pub fn want_present_mode(&mut self, present_modes_ord: Vec<PresentModeKHR>) {
    self.wanted_present_modes_ord = present_modes_ord;
  }
}

impl Default for SwapchainFeaturesQuery {
  fn default() -> Self {
    Self { wanted_image_count: unsafe { NonZeroU32::new_unchecked(1) }, wanted_present_modes_ord: Vec::new() }
  }
}

#[derive(Error, Debug)]
pub enum SwapchainCreateError {
  #[error("Failed to get surface format")]
  SurfaceFormatFail(#[from] SurfaceFormatError),
  #[error("Failed to get surface capabilities: {0:?}")]
  SurfaceCapabilitiesFail(#[source] VkError),
  #[error("Failed to find support composite alpha mode")]
  NoCompositeAlphaModeFound(),
  #[error("Failed to get surface present modes: {0:?}")]
  SurfacePresentModesFail(#[source] VkError),
  #[error("Failed to find present mode")]
  NoPresentModeFound(),
  #[error("Failed to create swapchain: {0:?}")]
  SwapchainCreateFail(#[source] VkError),
  #[error("Failed to get swapchain images: {0:?}")]
  SwapchainImagesFail(#[source] VkError),
  #[error("Failed to create image views for swapchain images")]
  SwapchainImageViewsCreateFail(#[from] ImageViewCreateError),
}

impl Swapchain {
  pub fn new(
    instance: &Instance,
    device: &Device,
    surface: &Surface,
    features_query: SwapchainFeaturesQuery,
    surface_extent: Extent2D,
  ) -> Result<Self, SwapchainCreateError> {
    let loader = SwapchainLoader::new(&instance.wrapped, &device.wrapped);
    Self::new_internal(loader, device, surface, features_query, surface_extent, None)
  }

  pub unsafe fn destroy(&mut self, device: &Device) {
    debug!("Destroying swapchain {:?}", self.wrapped);
    for image_view in &self.image_views {
      device.destroy_image_view(*image_view);
    }
    self.loader.destroy_swapchain(self.wrapped, None);
  }

  fn new_internal(
    loader: SwapchainLoader,
    device: &Device,
    surface: &Surface,
    features_query: SwapchainFeaturesQuery,
    surface_extent: Extent2D,
    old_swapchain: Option<&Swapchain>
  ) -> Result<Self, SwapchainCreateError> {
    use SwapchainCreateError::*;
    use std::cmp::{min, max};

    let capabilities = unsafe { surface.get_capabilities(device.physical_device) }
      .map_err(|e| SurfaceCapabilitiesFail(e))?;
    let wanted_image_count = features_query.wanted_image_count.get();
    let min_image_count = match capabilities.max_image_count {
      0 => max(capabilities.min_image_count, wanted_image_count),
      max_image_count => max(capabilities.min_image_count, min(wanted_image_count, max_image_count)),
    };
    let surface_format = unsafe { surface.get_suitable_surface_format(device.physical_device) }?;
    let extent = match (capabilities.current_extent.width, capabilities.current_extent.height) {
      (std::u32::MAX, std::u32::MAX) => surface_extent,
      _ => capabilities.current_extent,
    };
    let extent = {
      let min = capabilities.min_image_extent;
      let max = capabilities.max_image_extent;
      let width = if extent.width < min.width { min.width } else if extent.width > max.width { max.width } else { extent.width };
      let height = if extent.height < min.height { min.height } else if extent.height > max.height { max.height } else { extent.height };
      Extent2D { width, height }
    };
    // imageExtent = (1904,991) ||| minImageExtent = (1904,1006), maxImageExtent = (1904,1006)
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
    let composite_alpha = if capabilities.supported_composite_alpha.contains(CompositeAlphaFlagsKHR::OPAQUE) {
      CompositeAlphaFlagsKHR::OPAQUE
    } else {
      return Err(NoCompositeAlphaModeFound())
    };
    let present_mode = {
      let available_present_modes = unsafe { surface.get_present_modes(device.physical_device) }
        .map_err(|e| SurfacePresentModesFail(e))?;
      Self::select_present_mode(available_present_modes, features_query.wanted_present_modes_ord.clone())
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
      .composite_alpha(composite_alpha)
      .present_mode(present_mode)
      .clipped(true)
      ;
    if let Some(old_swapchain) = old_swapchain {
      create_info = create_info.old_swapchain(old_swapchain.wrapped);
    }
    let create_info = create_info.build();
    debug!("Creating swapchain from {:?}", create_info);
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
    };

    Ok(Self {
      loader,
      wrapped: swapchain,
      image_views,
      extent,
      features_query,
      features
    })
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
      Some(available_present_modes[0]) // No preference, return first present mode.
    } else {
      None // No present mode available.
    }
  }
}

// API

impl Swapchain {
  pub unsafe fn recreate(
    &mut self,
    device: &Device,
    surface: &Surface,
    surface_extent: Extent2D
  ) -> Result<(), SwapchainCreateError> {
    debug!("Recreating swapchain");
    let mut new_swapchain = Self::new_internal(
      self.loader.clone(),
      device,
      surface,
      self.features_query.clone(),
      surface_extent,
      Some(&self),
    )?;
    std::mem::swap(self, &mut new_swapchain);
    new_swapchain.destroy(device);
    Ok(())
  }
}

#[derive(Error, Debug)]
#[error("Failed to acquire next image from swapchain: {0:?}")]
pub struct AcquireNextImageError(#[from] VkError);

impl Swapchain {
  pub unsafe fn acquire_next_image(&self, timeout: Timeout, semaphore: Option<Semaphore>, fence: Option<Fence>) -> Result<(u32, bool), AcquireNextImageError> {
    Ok(self.loader.acquire_next_image(self.wrapped, timeout.into(), semaphore.unwrap_or_default(), fence.unwrap_or_default())?)
  }
}

#[derive(Error, Debug)]
#[error("Failed to present to queue: {0:?}")]
pub struct QueuePresentError(#[from] VkError);

impl Swapchain {
  pub unsafe fn queue_present(&self, queue: Queue, create_info: &vk::PresentInfoKHR) -> Result<bool, QueuePresentError> {
    let result = self.loader.queue_present(queue, create_info);
    match result {
      Err(VkError::ERROR_OUT_OF_DATE_KHR) => Ok(true),
      result => Ok(result?)
    }
  }
}

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

impl Deref for Swapchain {
  type Target = SwapchainKHR;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.wrapped }
}

// Extension name

pub const SWAPCHAIN_EXTENSION_NAME: &'static CStr = c_str!("VK_KHR_swapchain");
