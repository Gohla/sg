use anyhow::Result;
use byte_strings::c_str;
use raw_window_handle::RawWindowHandle;

use vkw::prelude::*;

pub fn create_entry() -> Result<Entry> {
  Ok(Entry::new()?)
}

pub fn create_instance(entry: &Entry) -> Result<Instance> {
  let features_query = {
    let mut query = InstanceFeaturesQuery::new();
    query.require_validation_layer();
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
  )?;
  Ok(instance)
}

pub fn create_debug_report(entry: &Entry, instance: &Instance) -> Result<DebugReport> {
  Ok(DebugReport::new(entry, instance)?)
}

pub fn create_surface(entry: &Entry, instance: &Instance, window: RawWindowHandle) -> Result<Surface> {
  Ok(Surface::new(entry, instance, window)?)
}

pub fn create_device<'e, 'i>(instance: &'i Instance<'e>, surface: &Surface) -> Result<Device<'e, 'i>> {
  let features_query = {
    let mut query = DeviceFeaturesQuery::new();
    query.require_graphics_queue();
    query.require_present_queue();
    query.require_swapchain_extension();
    query.require_features(PhysicalDeviceFeatures::builder().build());
    query
  };
  Ok(Device::new(instance, features_query, Some(surface))?)
}

pub fn create_swapchain_loader(instance: &Instance, device: &Device) -> SwapchainLoader {
  SwapchainLoader::new(instance, device)
}

pub fn create_swapchain<'l, S: Into<(u32, u32)>>(
  loader: &'l SwapchainLoader,
  device: &Device,
  surface: &Surface,
  surface_size: S,
  old_swapchain: Option<Swapchain>
) -> Result<Swapchain<'l>> {
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
  Ok(Swapchain::new(loader, device, surface, features_query, Extent2D { width, height }, old_swapchain)?)
}
