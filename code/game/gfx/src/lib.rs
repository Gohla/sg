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
    query.require_features(PhysicalDeviceFeatures::builder().build());
    query
  };
  Ok(Device::new(instance, features_query, Some(surface))?)
}
