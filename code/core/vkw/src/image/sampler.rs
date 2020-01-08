use ash::version::DeviceV1_0;
use ash::vk::{self, Result as VkError, Sampler, SamplerCreateInfo};
use log::trace;
use thiserror::Error;

use crate::device::Device;

// Creation and destruction

#[derive(Error, Debug)]
#[error("Failed to create image sampler: {0:?}")]
pub struct SamplerCreateError(#[from] VkError);

impl Device {
  pub unsafe fn create_sampler(&self, create_info: &SamplerCreateInfo) -> Result<Sampler, SamplerCreateError> {
    let sampler = self.wrapped.create_sampler(create_info, None)?;
    trace!("Created image sampler: {:?}", sampler);
    Ok(sampler)
  }

  pub unsafe fn create_default_sampler(&self) -> Result<Sampler, SamplerCreateError> {
    use vk::{Filter, SamplerMipmapMode, SamplerAddressMode, CompareOp, BorderColor};
    self.create_sampler(&SamplerCreateInfo::builder()
      .mag_filter(Filter::NEAREST)
      .min_filter(Filter::NEAREST)
      .mipmap_mode(SamplerMipmapMode::NEAREST)
      .address_mode_u(SamplerAddressMode::REPEAT)
      .address_mode_v(SamplerAddressMode::REPEAT)
      .address_mode_w(SamplerAddressMode::REPEAT)
      .mip_lod_bias(0.0)
      .anisotropy_enable(false)
      .max_anisotropy(1.0)
      .compare_enable(false)
      .compare_op(CompareOp::NEVER)
      .min_lod(0.0)
      .max_lod(0.0)
      .border_color(BorderColor::FLOAT_OPAQUE_WHITE)
      .unnormalized_coordinates(false)
    )
  }

  pub unsafe fn destroy_sampler(&self, sampler: Sampler) {
    trace!("Destroying image sampler: {:?}", sampler);
    self.wrapped.destroy_sampler(sampler, None);
  }
}
