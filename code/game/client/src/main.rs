use anyhow::{Context, Result};

use gfx::{GfxEntry, GfxInstance};

fn main() -> Result<()> {
  simple_logger::init().with_context(|| "Failed to initialize logger")?;
  let entry = GfxEntry::new().with_context(|| "Failed to initialize GFX entry")?;
  dbg!(entry.entry.instance_version());
  let _instance = GfxInstance::new(&entry).with_context(|| "Failed to initialize GFX instance")?;
  Ok(())
}
