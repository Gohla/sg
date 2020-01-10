use anyhow::Result;

use util::idx_assigner::IdxAssigner;
use util::image::ImageData;
use vkw::prelude::*;

// Texture index

#[derive(Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub struct TextureIdx(u16);

// Texture def builder

pub struct TextureDefBuilder {
  assigner: IdxAssigner<u16, TextureIdx>,
  data: Vec<ImageData>,
}

impl TextureDefBuilder {
  pub fn new() -> Self {
    Self { assigner: IdxAssigner::new(), data: Vec::new() }
  }


  pub fn add_texture(&mut self, data: ImageData) -> TextureIdx {
    let idx = self.assigner.assign_item();
    self.data.push(data);
    idx
  }

  pub unsafe fn build(self, device: &Device, allocator: &Allocator, transient_command_pool: CommandPool) -> Result<TextureDef> {
    let format = device.find_suitable_format(&[Format::R8G8B8A8_UNORM], ImageTiling::OPTIMAL, FormatFeatureFlags::SAMPLED_IMAGE | FormatFeatureFlags::TRANSFER_DST)?;
    let texture_data = device.allocate_record_resources_submit_wait(allocator, transient_command_pool, |command_buffer| {
      Ok(device.allocate_record_copy_textures(self.data, allocator, format, command_buffer)?)
    })?;
    Ok(TextureDef::new(texture_data))
  }
}

// Texture definition

pub struct TextureDef {
  data: Vec<Texture>
}

impl TextureDef {
  fn new(data: Vec<Texture>) -> Self { Self { data } }

  pub unsafe fn destroy(&self, device: &Device, allocator: &Allocator) {
    for data in &self.data {
      data.destroy(device, allocator);
    }
  }
}

// Implementations

impl Into<TextureIdx> for u16 {
  #[inline]
  fn into(self) -> TextureIdx { TextureIdx(self) }
}

impl Into<u16> for TextureIdx {
  #[inline]
  fn into(self) -> u16 { self.0 }
}
