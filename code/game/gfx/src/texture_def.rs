use anyhow::Result;
use ash::vk::ImageLayout;

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
    let textures = device.allocate_record_resources_submit_wait(allocator, transient_command_pool, |command_buffer| {
      Ok(device.allocate_record_copy_textures(self.data, allocator, format, command_buffer)?)
    })?;
    let count = textures.len() as u32;

    let descriptor_set_layout_bindings = &[descriptor_set::sampler_layout_binding(0, count)];
    let descriptor_set_layout_flags = &[DescriptorBindingFlagsEXT::VARIABLE_DESCRIPTOR_COUNT];
    let descriptor_set_layout = device.create_descriptor_set_layout(descriptor_set_layout_bindings, descriptor_set_layout_flags)?;

    let descriptor_pool = device.create_descriptor_pool(1, &[descriptor_set::sampler_pool_size(count)])?;

    let descriptor_set = device.allocate_descriptor_set(descriptor_pool, descriptor_set_layout)?;
    let mut write_builder = WriteDescriptorSetBuilder::new(descriptor_set, 0, 0, DescriptorType::COMBINED_IMAGE_SAMPLER);
    for texture in &textures {
      write_builder = write_builder.add_image_info(texture.sampler, texture.view, ImageLayout::SHADER_READ_ONLY_OPTIMAL)
    }
    DescriptorSetUpdateBuilder::new()
      .add_write(write_builder)
      .do_update(device);
    Ok(TextureDef::new(textures, descriptor_set_layout, descriptor_pool, descriptor_set))
  }
}

// Texture definition

pub struct TextureDef {
  pub textures: Vec<Texture>,
  pub descriptor_set_layout: DescriptorSetLayout,
  pub descriptor_pool: DescriptorPool,
  pub descriptor_set: DescriptorSet,
}

impl TextureDef {
  fn new(
    textures: Vec<Texture>,
    descriptor_set_layout: DescriptorSetLayout,
    descriptor_pool: DescriptorPool,
    descriptor_set: DescriptorSet,
  ) -> Self {
    Self {
      textures,
      descriptor_set_layout,
      descriptor_pool,
      descriptor_set,
    }
  }

  pub unsafe fn destroy(&self, device: &Device, allocator: &Allocator) {
    device.destroy_descriptor_pool(self.descriptor_pool);
    device.destroy_descriptor_set_layout(self.descriptor_set_layout);
    for data in &self.textures {
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
