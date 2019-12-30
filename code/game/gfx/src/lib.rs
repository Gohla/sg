use anyhow::Result;
use byte_strings::c_str;

use vkw::prelude::*;

pub fn create_entry() -> Result<Entry> {
  Ok(Entry::new()?)
}

pub fn create_instance(entry: &Entry) -> Result<Instance> {
  let feature_query = {
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
    feature_query
  )?;
  Ok(instance)
}

pub fn create_debug_report(entry: &Entry, instance: &Instance) -> Result<DebugReport> {
  Ok(DebugReport::new(entry, instance)?)
}
