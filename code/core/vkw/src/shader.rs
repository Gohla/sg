use std::io::{self, Cursor};

use ash::util::read_spv;
use ash::version::DeviceV1_0;
use ash::vk::{self, PipelineShaderStageCreateInfo, Result as VkError, ShaderModule, ShaderStageFlags, SpecializationInfo};
use byte_strings::c_str;
use log::debug;
use thiserror::Error;

use crate::device::Device;

// Module creation and destruction

#[derive(Error, Debug)]
pub enum ShaderModuleCreateError {
  #[error("Failed to read SPIR-V from bytes")]
  SPIRVReadFail(#[from] io::Error),
  #[error("Failed to create shader module")]
  CreateShaderModuleFail(#[from] VkError),
}

impl Device {
  pub unsafe fn create_shader_module(&self, bytes: &[u8]) -> Result<ShaderModule, ShaderModuleCreateError> {
    let mut cursor = Cursor::new(bytes);
    let code = read_spv(&mut cursor)?;
    let create_info = vk::ShaderModuleCreateInfo::builder()
      .code(&code)
      .build();
    debug!("Creating shader module from {:?}", create_info);
    Ok(self.wrapped.create_shader_module(&create_info, None)?)
  }

  pub unsafe fn destroy_shader_module(&self, shader_module: ShaderModule) {
    debug!("Destroying shader module {:?}", shader_module);
    self.wrapped.destroy_shader_module(shader_module, None);
  }
}

// Stage creation

pub trait ShaderModuleEx {
  fn create_shader_stage(&self, stage: ShaderStageFlags, specialization_info: Option<&SpecializationInfo>) -> PipelineShaderStageCreateInfo;

  fn create_vertex_shader_stage(&self, specialization_info: Option<&SpecializationInfo>) -> PipelineShaderStageCreateInfo {
    return self.create_shader_stage(ShaderStageFlags::VERTEX, specialization_info);
  }

  fn create_tessellation_control_shader_stage(&self, specialization_info: Option<&SpecializationInfo>) -> PipelineShaderStageCreateInfo {
    return self.create_shader_stage(ShaderStageFlags::TESSELLATION_CONTROL, specialization_info);
  }

  fn create_tessellation_evaluation_shader_stage(&self, specialization_info: Option<&SpecializationInfo>) -> PipelineShaderStageCreateInfo {
    return self.create_shader_stage(ShaderStageFlags::TESSELLATION_EVALUATION, specialization_info);
  }

  fn create_geometry_shader_stage(&self, specialization_info: Option<&SpecializationInfo>) -> PipelineShaderStageCreateInfo {
    return self.create_shader_stage(ShaderStageFlags::GEOMETRY, specialization_info);
  }

  fn create_fragment_shader_stage(&self, specialization_info: Option<&SpecializationInfo>) -> PipelineShaderStageCreateInfo {
    return self.create_shader_stage(ShaderStageFlags::FRAGMENT, specialization_info);
  }

  fn create_compute_shader_stage(&self, specialization_info: Option<&SpecializationInfo>) -> PipelineShaderStageCreateInfo {
    return self.create_shader_stage(ShaderStageFlags::COMPUTE, specialization_info);
  }
}

impl ShaderModuleEx for ShaderModule {
  fn create_shader_stage(
    &self,
    stage: ShaderStageFlags,
    specialization_info: Option<&SpecializationInfo>,
  ) -> PipelineShaderStageCreateInfo {
    let mut create_info = vk::PipelineShaderStageCreateInfo::builder()
      .stage(stage)
      .module(*self)
      .name(c_str!("main"));
    if let Some(specialization_info) = specialization_info {
      create_info = create_info.specialization_info(specialization_info)
    }
    create_info.build()
  }
}
