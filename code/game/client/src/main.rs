use anyhow::{Context, Result};

use gfx::{create_debug_report, create_entry, create_instance};

fn main() -> Result<()> {
  simple_logger::init().with_context(|| "Failed to initialize logger")?;
  let entry = create_entry().with_context(|| "Failed to initialize GFX entry")?;
  dbg!(entry.instance_version());
  let instance = create_instance(&entry).with_context(|| "Failed to initialize GFX instance")?;
  let _debug_report = create_debug_report(&entry, &instance).with_context(|| "Failed to initialize GFX debug report")?;
  Ok(())
}
