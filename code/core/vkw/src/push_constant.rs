use ash::vk::{PushConstantRange, ShaderStageFlags};

pub fn range(stage_flags: ShaderStageFlags, size: u32, offset: u32) -> PushConstantRange {
  PushConstantRange::builder()
    .stage_flags(stage_flags)
    .size(size)
    .offset(offset)
    .build()
}

pub fn vertex_range(size: u32, offset: u32) -> PushConstantRange {
  range(ShaderStageFlags::VERTEX, size, offset)
}

pub fn fragment_range(size: u32, offset: u32) -> PushConstantRange {
  range(ShaderStageFlags::FRAGMENT, size, offset)
}

pub fn vertex_and_fragment_range(size: u32, offset: u32) -> PushConstantRange {
  range(ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT, size, offset)
}
