use anyhow::{Context, Result};

use vk::entry::VkEntry;
use vk::instance::VkInstance;

pub struct Gfx {
  pub entry: VkEntry,
  pub instance: VkInstance,
}

impl Gfx {
  pub fn new() -> Result<Self> {
    let entry = VkEntry::new()
      .with_context(|| "Failed to create Vulkan entry")?;

    let instance = entry.create_instance(
      Some(env!("CARGO_PKG_NAME")),
      None,
      None,
      None,
      None,
      None,
      None,
    )
      .with_context(|| "Failed to create Vulkan instance")?;

    let gfx = Gfx { entry, instance };
    Ok(gfx)
  }
}
