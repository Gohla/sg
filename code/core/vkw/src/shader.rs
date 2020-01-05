use std::io::{self, Cursor};

use ash::util::read_spv;
use ash::version::DeviceV1_0;
use ash::vk::{self, PipelineShaderStageCreateInfoBuilder, Result as VkError, ShaderModule, ShaderStageFlags, SpecializationInfo};
use byte_strings::c_str;
use log::debug;
use thiserror::Error;

use crate::device::Device;

// Module creation and destruction

#[derive(Error, Debug)]
pub enum ShaderModuleCreateError {
  #[error("Failed to read SPIR-V from bytes: {0:?}")]
  SPIRVReadFail(#[from] io::Error),
  #[error("Failed to create shader module: {0:?}")]
  CreateShaderModuleFail(#[from] VkError),
}

impl Device {
  pub unsafe fn create_shader_module(&self, bytes: &[u8]) -> Result<ShaderModule, ShaderModuleCreateError> {
    let mut cursor = Cursor::new(bytes);
    let code = read_spv(&mut cursor)?;
    let create_info = vk::ShaderModuleCreateInfo::builder()
      .code(&code)
      ;
    let shader_module = self.wrapped.create_shader_module(&create_info, None)?;
    debug!("Created shader module {:?}", shader_module);
    Ok(shader_module)
  }

  pub unsafe fn destroy_shader_module(&self, shader_module: ShaderModule) {
    debug!("Destroying shader module {:?}", shader_module);
    self.wrapped.destroy_shader_module(shader_module, None);
  }
}

// Stage creation

pub trait ShaderModuleEx {
  fn create_shader_stage<'a>(&self, stage: ShaderStageFlags, specialization_info: Option<&'a SpecializationInfo>) -> PipelineShaderStageCreateInfoBuilder<'a>;

  fn create_vertex_shader_stage<'a>(&self, specialization_info: Option<&'a SpecializationInfo>) -> PipelineShaderStageCreateInfoBuilder<'a> {
    return self.create_shader_stage(ShaderStageFlags::VERTEX, specialization_info);
  }

  fn create_tessellation_control_shader_stage<'a>(&self, specialization_info: Option<&'a SpecializationInfo>) -> PipelineShaderStageCreateInfoBuilder<'a> {
    return self.create_shader_stage(ShaderStageFlags::TESSELLATION_CONTROL, specialization_info);
  }

  fn create_tessellation_evaluation_shader_stage<'a>(&self, specialization_info: Option<&'a SpecializationInfo>) -> PipelineShaderStageCreateInfoBuilder<'a> {
    return self.create_shader_stage(ShaderStageFlags::TESSELLATION_EVALUATION, specialization_info);
  }

  fn create_geometry_shader_stage<'a>(&self, specialization_info: Option<&'a SpecializationInfo>) -> PipelineShaderStageCreateInfoBuilder<'a> {
    return self.create_shader_stage(ShaderStageFlags::GEOMETRY, specialization_info);
  }

  fn create_fragment_shader_stage<'a>(&self, specialization_info: Option<&'a SpecializationInfo>) -> PipelineShaderStageCreateInfoBuilder<'a> {
    return self.create_shader_stage(ShaderStageFlags::FRAGMENT, specialization_info);
  }

  fn create_compute_shader_stage<'a>(&self, specialization_info: Option<&'a SpecializationInfo>) -> PipelineShaderStageCreateInfoBuilder<'a> {
    return self.create_shader_stage(ShaderStageFlags::COMPUTE, specialization_info);
  }
}

impl ShaderModuleEx for ShaderModule {
  fn create_shader_stage<'a>(
    &self,
    stage: ShaderStageFlags,
    specialization_info: Option<&'a SpecializationInfo>,
  ) -> PipelineShaderStageCreateInfoBuilder<'a> {
    let mut create_info = vk::PipelineShaderStageCreateInfo::builder()
      .stage(stage)
      .module(*self)
      // CORRECTNESS: `name` is taken by pointer but is always alive because it is a 'static literal.
      .name(c_str!("main"))
      ;
    if let Some(specialization_info) = specialization_info {
      create_info = create_info.specialization_info(specialization_info)
    }
    create_info
  }
}
