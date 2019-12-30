use anyhow::{Context, Result};
use byte_strings::c_str;

use vkw::entry::VkEntry;
use vkw::instance::VkInstance;

pub struct GfxEntry {
  pub entry: VkEntry,
}

impl GfxEntry {
  pub fn new() -> Result<Self> {
    let entry = VkEntry::new()
      .with_context(|| "Failed to create Vulkan entry")?;
    Ok(Self { entry })
  }
}

pub struct GfxInstance<'e> {
  pub instance: VkInstance<'e>,
}

impl<'e> GfxInstance<'e> {
  pub fn new(entry: &'e GfxEntry) -> Result<Self> {
    let instance = entry.entry.create_instance(
      Some(c_str!("SG")),
      None,
      Some(c_str!("SG GFX")),
      None,
      None,
      None,
    ).with_context(|| "Failed to create Vulkan instance")?;
    let gfx = GfxInstance { instance };
    Ok(gfx)
  }
}
