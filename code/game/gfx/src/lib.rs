use std::mem::ManuallyDrop;

use anyhow::{Context, Result};
use byte_strings::c_str;
use raw_window_handle::RawWindowHandle;

use vkw::prelude::*;

pub struct GfxDevice {
  pub instance: ManuallyDrop<Instance>,
  pub debug_report: ManuallyDrop<Option<DebugReport>>,
  pub surface: ManuallyDrop<Surface>,
  pub device: ManuallyDrop<Device>,
  pub swapchain: ManuallyDrop<Swapchain>,
}

impl GfxDevice {
  pub fn new<S: Into<(u32, u32)>>(require_validation_layer: bool, window: RawWindowHandle, surface_size: S) -> Result<GfxDevice> {
    let entry = Entry::new()
      .with_context(|| "Failed to create VKW entry")?;
    let instance = {
      let features_query = {
        let mut query = InstanceFeaturesQuery::new();
        if require_validation_layer {
          query.require_validation_layer();
        }
        query.require_surface();
        query
      };
      let instance = Instance::new(
        entry,
        Some(c_str!("SG")),
        None,
        Some(c_str!("SG GFX")),
        None,
        None,
        features_query
      ).with_context(|| "Failed to create VKW instance")?;
      instance
    };

    let debug_report = if require_validation_layer {
      Some(DebugReport::new(&instance).with_context(|| "Failed to create VKW debug report")?)
    } else {
      None
    };
    let surface = Surface::new(&instance, window).with_context(|| "Failed to create VKW surface")?;

    let device = {
      let features_query = {
        let mut query = DeviceFeaturesQuery::new();
        query.require_swapchain_extension();
        query.require_features(PhysicalDeviceFeatures::builder().build());
        query
      };
      Device::new(&instance, features_query, Some(&surface))?
    };

    let swapchain = {
      let features_query = {
        let mut query = SwapchainFeaturesQuery::new();
        query.want_image_count(2);
        query.want_present_mode(vec![
          PresentModeKHR::IMMEDIATE,
          PresentModeKHR::MAILBOX,
          PresentModeKHR::FIFO_RELAXED,
          PresentModeKHR::FIFO,
        ]);
        query
      };
      let (width, height) = surface_size.into();
      Swapchain::new(&instance, &device, &surface, features_query, Extent2D { width, height })?
    };

    Ok(Self {
      instance: ManuallyDrop::new(instance),
      surface: ManuallyDrop::new(surface),
      debug_report: ManuallyDrop::new(debug_report),
      device: ManuallyDrop::new(device),
      swapchain: ManuallyDrop::new(swapchain),
    })
  }
}

impl Drop for GfxDevice {
  fn drop(&mut self) {
    unsafe {
      ManuallyDrop::drop(&mut self.swapchain);
      ManuallyDrop::drop(&mut self.device);
      ManuallyDrop::drop(&mut self.surface);
      ManuallyDrop::drop(&mut self.debug_report);
      ManuallyDrop::drop(&mut self.instance);
    }
  }
}
