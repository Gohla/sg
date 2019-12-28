use anyhow::Result;

use gfx::Gfx;

fn main() -> Result<()> {
  let gfx = Gfx::new()?;
  dbg!(gfx.entry.instance_version());
  Ok(())
}
