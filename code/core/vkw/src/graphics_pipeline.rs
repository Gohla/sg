use ash::version::DeviceV1_0;
use ash::vk::{self, DescriptorSetLayout, GraphicsPipelineCreateInfo, Pipeline, PipelineCache, PipelineLayout, PushConstantRange, Result as VkError};
use log::debug;
use thiserror::Error;

use crate::device::Device;

// Pipeline layout creation and destruction.

#[derive(Error, Debug)]
#[error("Failed to create pipeline layout: {0:?}")]
pub struct PipelineLayoutCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_pipeline_layout(
    &self,
    descriptor_set_layouts: &[DescriptorSetLayout],
    push_constant_ranges: &[PushConstantRange],
  ) -> Result<PipelineLayout, PipelineLayoutCreateError> {
    let create_info = vk::PipelineLayoutCreateInfo::builder()
      .set_layouts(descriptor_set_layouts)
      .push_constant_ranges(push_constant_ranges)
      .build();
    debug!("Creating pipeline layout from {:?}", create_info);
    Ok(self.wrapped.create_pipeline_layout(&create_info, None)?)
  }

  pub unsafe fn destroy_pipeline_layout(&self, pipeline_layout: PipelineLayout) {
    debug!("Destroying pipeline layout {:?}", pipeline_layout);
    self.wrapped.destroy_pipeline_layout(pipeline_layout, None);
  }
}

// Pipeline cache creation and destruction.

#[derive(Error, Debug)]
#[error("Failed to create pipeline cache: {0:?}")]
pub struct PipelineCacheCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_pipeline_cache(&self) -> Result<PipelineCache, PipelineCacheCreateError> {
    let create_info = vk::PipelineCacheCreateInfo::builder().build();
    debug!("Creating pipeline cache from {:?}", create_info);
    Ok(self.wrapped.create_pipeline_cache(&create_info, None)?)
  }

  pub unsafe fn destroy_pipeline_cache(&self, pipeline_cache: PipelineCache) {
    debug!("Destroying pipeline cache {:?}", pipeline_cache);
    self.wrapped.destroy_pipeline_cache(pipeline_cache, None);
  }
}

// Graphics pipeline creation and destruction.

#[derive(Error, Debug)]
#[error("Failed to create graphics pipeline: {0:?}")]
pub struct GraphicsPipelineCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_graphics_pipelines(
    &self,
    pipeline_cache: PipelineCache,
    create_infos: &[GraphicsPipelineCreateInfo]
  ) -> Result<Vec<Pipeline>, GraphicsPipelineCreateError> {
    debug!("Creating graphics pipelines from {:?}", create_infos);
    match self.wrapped.create_graphics_pipelines(pipeline_cache, create_infos, None) {
      Err((_, e)) => Err(e)?,
      Ok(v) => Ok(v),
    }
  }

  pub unsafe fn create_graphics_pipeline(
    &self,
    pipeline_cache: PipelineCache,
    create_info: &GraphicsPipelineCreateInfo
  ) -> Result<Pipeline, GraphicsPipelineCreateError> {
    Ok(self.create_graphics_pipelines(pipeline_cache, &[*create_info])?[0])
  }

  pub unsafe fn destroy_pipeline(&self, pipeline: Pipeline) {
    debug!("Destroying pipeline {:?}", pipeline);
    self.wrapped.destroy_pipeline(pipeline, None);
  }
}
