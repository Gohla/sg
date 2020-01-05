use ash::version::DeviceV1_0;
use ash::vk::{self, Buffer, BufferView, DescriptorBufferInfo, DescriptorImageInfo, DescriptorPool, DescriptorPoolSize, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorType, DeviceSize, ImageLayout, ImageView, Result as VkError, Sampler, ShaderStageFlags, WriteDescriptorSet};
use log::debug;
use thiserror::Error;

use crate::device::Device;

// Descriptor set layout binding

pub fn layout_binding(
  binding: u32,
  descriptor_type: DescriptorType,
  descriptor_count: u32,
  stage_flags: ShaderStageFlags,
) -> DescriptorSetLayoutBinding {
  DescriptorSetLayoutBinding::builder()
    .binding(binding)
    .descriptor_type(descriptor_type)
    .descriptor_count(descriptor_count)
    .stage_flags(stage_flags)
    .build()
}

fn uniform_descriptor_type(dynamic: bool) -> DescriptorType {
  if dynamic { DescriptorType::UNIFORM_BUFFER_DYNAMIC } else { DescriptorType::UNIFORM_BUFFER }
}

pub fn uniform_layout_binding(binding: u32, count: u32, dynamic: bool, stage_flags: ShaderStageFlags) -> DescriptorSetLayoutBinding {
  layout_binding(binding, uniform_descriptor_type(dynamic), count, stage_flags)
}

pub fn sampler_layout_binding(binding: u32, count: u32) -> DescriptorSetLayoutBinding {
  layout_binding(binding, DescriptorType::COMBINED_IMAGE_SAMPLER, count, ShaderStageFlags::FRAGMENT)
}

// Descriptor set layout creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create descriptor set layout: {0:?}")]
pub struct DescriptorSetLayoutCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_descriptor_set_layout(&self, bindings: &[DescriptorSetLayoutBinding]) -> Result<DescriptorSetLayout, DescriptorSetLayoutCreateError> {
    let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
      .bindings(bindings)
      ;
    let descriptor_set_layout = self.wrapped.create_descriptor_set_layout(&create_info, None)?;
    debug!("Created descriptor set layout {:?}", descriptor_set_layout);
    Ok(descriptor_set_layout)
  }

  pub unsafe fn destroy_descriptor_set_layout(&self, layout: DescriptorSetLayout) {
    debug!("Destroying descriptor set layout {:?}", layout);
    self.wrapped.destroy_descriptor_set_layout(layout, None)
  }
}

// Descriptor pool sizes

pub fn pool_size(ty: DescriptorType, count: u32) -> DescriptorPoolSize {
  DescriptorPoolSize::builder().ty(ty).descriptor_count(count).build()
}

pub fn uniform_pool_size(count: u32, dynamic: bool) -> DescriptorPoolSize {
  pool_size(uniform_descriptor_type(dynamic), count)
}

pub fn sampler_pool_size(count: u32) -> DescriptorPoolSize {
  pool_size(DescriptorType::COMBINED_IMAGE_SAMPLER, count)
}

// Descriptor pool creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create descriptor pool: {0:?}")]
pub struct DescriptorPoolCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_descriptor_pool(&self, max_sets: u32, pool_sizes: &[DescriptorPoolSize]) -> Result<DescriptorPool, DescriptorPoolCreateError> {
    let create_info = vk::DescriptorPoolCreateInfo::builder()
      .max_sets(max_sets)
      .pool_sizes(&pool_sizes)
      ;
    let descriptor_pool = self.wrapped.create_descriptor_pool(&create_info, None)?;
    debug!("Created descriptor pool {:?}", descriptor_pool);
    Ok(descriptor_pool)
  }

  pub unsafe fn destroy_descriptor_pool(&self, pool: DescriptorPool) {
    debug!("Destroying descriptor pool {:?}", pool);
    self.wrapped.destroy_descriptor_pool(pool, None);
  }
}

// Descriptor set allocation and freeing

#[derive(Error, Debug)]
#[error("Failed to allocate descriptor sets: {0:?}")]
pub struct DescriptorSetsAllocateError(#[from] VkError);

impl Device {
  pub unsafe fn allocate_descriptor_sets(&self, pool: DescriptorPool, layout: DescriptorSetLayout, count: usize) -> Result<Vec<DescriptorSet>, DescriptorSetsAllocateError> {
    let set_layouts = vec![layout; count];
    let create_info = vk::DescriptorSetAllocateInfo::builder()
      .descriptor_pool(pool)
      .set_layouts(&set_layouts)
      ;
    let descriptor_sets = self.wrapped.allocate_descriptor_sets(&create_info)?;
    debug!("Created descriptor sets {:?}", descriptor_sets);
    Ok(descriptor_sets)
  }

  pub unsafe fn allocate_descriptor_set(&self, pool: DescriptorPool, layout: DescriptorSetLayout) -> Result<DescriptorSet, DescriptorSetsAllocateError> {
    Ok(self.allocate_descriptor_sets(pool, layout, 1)?[0])
  }

  pub unsafe fn free_descriptor_sets(&self, pool: DescriptorPool, descriptor_sets: &[DescriptorSet]) {
    self.wrapped.free_descriptor_sets(pool, descriptor_sets);
  }

  pub unsafe fn free_descriptor_set(&self, pool: DescriptorPool, descriptor_set: DescriptorSet) {
    self.free_descriptor_sets(pool, &[descriptor_set])
  }
}

// Descriptor set update

#[derive(Default)]
pub struct DescriptorSetUpdateBuilder {
  writes: Vec<WriteDescriptorSetBuilder>,
}

impl DescriptorSetUpdateBuilder {
  pub fn new() -> Self { Self::default() }

  pub fn writes(mut self, writes: Vec<WriteDescriptorSetBuilder>) -> Self {
    self.writes = writes;
    self
  }

  pub fn add_write(mut self, write: WriteDescriptorSetBuilder) -> Self {
    self.writes.push(write);
    self
  }

  pub fn add_buffer_write(
    self,
    dst_set: DescriptorSet,
    dst_binding: u32,
    dst_array_element: u32,
    descriptor_type: DescriptorType,
    buffer: Buffer,
    buffer_offset: DeviceSize,
    buffer_range: DeviceSize
  ) -> Self {
    self.add_write(WriteDescriptorSetBuilder::new_buffer_write(dst_set, dst_binding, dst_array_element, descriptor_type, buffer, buffer_offset, buffer_range))
  }

  pub fn add_uniform_buffer_write(
    self,
    dst_set: DescriptorSet,
    dst_binding: u32,
    dst_array_element: u32,
    dynamic: bool,
    buffer: Buffer,
    buffer_offset: DeviceSize,
    buffer_range: DeviceSize
  ) -> Self {
    self.add_buffer_write(dst_set, dst_binding, dst_array_element, uniform_descriptor_type(dynamic), buffer, buffer_offset, buffer_range)
  }

  pub unsafe fn do_update(&self, device: &Device) {
    let writes: Vec<_> = self.writes.iter().map(|w| w.build()).collect();
    device.wrapped.update_descriptor_sets(&writes, &[]);
  }
}

#[derive(Default)]
pub struct WriteDescriptorSetBuilder {
  dst_set: DescriptorSet,
  dst_binding: u32,
  dst_array_element: u32,
  descriptor_type: DescriptorType,
  image_infos: Option<Vec<DescriptorImageInfo>>,
  buffer_infos: Option<Vec<DescriptorBufferInfo>>,
  texel_buffer_views: Option<Vec<BufferView>>,
}

impl WriteDescriptorSetBuilder {
  pub fn new() -> Self { Self::default() }

  pub fn new_buffer_write(
    dst_set: DescriptorSet,
    dst_binding: u32,
    dst_array_element: u32,
    descriptor_type: DescriptorType,
    buffer: Buffer,
    buffer_offset: DeviceSize,
    buffer_range: DeviceSize
  ) -> Self {
    Self {
      dst_set,
      dst_binding,
      dst_array_element,
      descriptor_type,
      buffer_infos: Some(vec![DescriptorBufferInfo { buffer, offset: buffer_offset, range: buffer_range }]),
      ..Self::default()
    }
  }

  pub fn dst_set(mut self, dst_set: DescriptorSet) -> Self {
    self.dst_set = dst_set;
    self
  }

  pub fn dst_binding(mut self, dst_binding: u32) -> Self {
    self.dst_binding = dst_binding;
    self
  }

  pub fn dst_array_element(mut self, dst_array_element: u32) -> Self {
    self.dst_array_element = dst_array_element;
    self
  }

  pub fn descriptor_type(mut self, descriptor_type: DescriptorType) -> Self {
    self.descriptor_type = descriptor_type;
    self
  }

  pub fn image_infos(mut self, image_info: Vec<DescriptorImageInfo>) -> Self {
    self.image_infos = Some(image_info);
    self
  }

  pub fn add_image_info(mut self, sampler: Sampler, image_view: ImageView, image_layout: ImageLayout) -> Self {
    let info = DescriptorImageInfo { sampler, image_view, image_layout };
    if let Some(image_infos) = &mut self.image_infos {
      image_infos.push(info);
    } else {
      self.image_infos = Some(vec![info]);
    }
    self
  }

  pub fn buffer_infos(mut self, buffer_info: Vec<DescriptorBufferInfo>) -> Self {
    self.buffer_infos = Some(buffer_info);
    self
  }

  pub fn add_buffer_info(mut self, buffer: Buffer, offset: DeviceSize, range: DeviceSize) -> Self {
    let info = DescriptorBufferInfo { buffer, offset, range };
    if let Some(buffer_infos) = &mut self.buffer_infos {
      buffer_infos.push(info);
    } else {
      self.buffer_infos = Some(vec![info]);
    }
    self
  }

  pub fn texel_buffer_views(mut self, texel_buffer_view: Vec<BufferView>) -> Self {
    self.texel_buffer_views = Some(texel_buffer_view);
    self
  }

  fn build(&self) -> WriteDescriptorSet {
    let mut builder = WriteDescriptorSet::builder()
      .dst_set(self.dst_set)
      .dst_binding(self.dst_binding)
      .dst_array_element(self.dst_array_element)
      .descriptor_type(self.descriptor_type)
      ;
    if let Some(image_infos) = &self.image_infos {
      builder = builder.image_info(image_infos)
    }
    if let Some(buffer_infos) = &self.buffer_infos {
      builder = builder.buffer_info(buffer_infos)
    }
    if let Some(texel_buffer_views) = &self.texel_buffer_views {
      builder = builder.texel_buffer_view(texel_buffer_views)
    }
    builder.build()
  }
}
