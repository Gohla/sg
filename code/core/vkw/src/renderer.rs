use std::fmt::Debug;
use std::num::NonZeroU32;

use ash::vk::{CommandPool, Fence, Semaphore};
use thiserror::Error;

use crate::command_pool::{CommandPoolCreateError, CommandPoolResetError};
use crate::device::Device;
use crate::sync::{FenceCreateError, FenceResetError, FenceWaitError, SemaphoreCreateError};
use crate::timeout::Timeout;

// Renderer

pub struct Renderer<T> {
  count: usize,
  index: usize,
  states: Box<[RenderState]>,
  states_custom: Box<[T]>,
}

pub struct RenderState {
  pub command_pool: CommandPool,
  pub image_acquired_semaphore: Semaphore,
  pub render_complete_semaphore: Semaphore,
  pub render_complete_fence: Fence,
  // TODO: track buffer allocations
}

// Creation and destruction

#[derive(Error, Debug)]
pub enum RenderCreateError {
  #[error(transparent)]
  CommandPoolCreateFail(#[from] CommandPoolCreateError),
  #[error("Failed to create image acquired semaphore")]
  ImageAcquiredSemaphoreCreateFail(#[source] SemaphoreCreateError),
  #[error("Failed to create render complete semaphore")]
  RenderCompleteSemaphoreCreateFail(#[source] SemaphoreCreateError),
  #[error("Failed to create render complete fence")]
  RenderCompleteFenceCreateFail(#[from] FenceCreateError),
  #[error("Failed to create custom render state")]
  CustomRenderStateCreateFail(#[source] anyhow::Error),
}

impl<T> Renderer<T> {
  pub fn new<F: Fn(&RenderState) -> Result<T, anyhow::Error>>(
    device: &Device,
    state_count: NonZeroU32,
    create_custom_state: F
  ) -> Result<Renderer<T>, RenderCreateError> {
    use RenderCreateError::*;
    let count = state_count.get() as usize;
    let (states, states_custom) = {
      let mut states = Vec::with_capacity(count);
      let mut states_custom: Vec<T> = Vec::with_capacity(count);
      for _i in 0..count {
        let state = unsafe {
          RenderState {
            command_pool: device.create_command_pool(false, false)?,
            image_acquired_semaphore: device.create_semaphore().map_err(|e| ImageAcquiredSemaphoreCreateFail(e))?,
            render_complete_semaphore: device.create_semaphore().map_err(|e| RenderCompleteSemaphoreCreateFail(e))?,
            render_complete_fence: device.create_fence(true)?,
          }
        };
        let state_custom = create_custom_state(&state).map_err(|e| CustomRenderStateCreateFail(e))?;
        states.push(state);
        states_custom.push(state_custom);
      }
      (states.into_boxed_slice(), states_custom.into_boxed_slice())
    };

    Ok(Renderer {
      count,
      index: count - 1,
      states,
      states_custom,
    })
  }

  pub unsafe fn destroy<F: Fn(&RenderState, &T)>(&self, device: &Device, destroy_fn: F) {
    for (state, state_custom) in self.states.iter().zip(self.states_custom.iter()) {
      destroy_fn(state, state_custom);
      device.destroy_command_pool(state.command_pool);
      device.destroy_semaphore(state.image_acquired_semaphore);
      device.destroy_semaphore(state.render_complete_semaphore);
      device.destroy_fence(state.render_complete_fence);
    }
  }
}

// API

impl<T> Renderer<T> {
  pub fn next_render_state(&mut self, device: &Device) -> Result<(&mut RenderState, &T), RenderStateWaitAndResetError> {
    self.index = (self.index + 1) % self.count;
    let state = &mut self.states[self.index];
    state.wait_and_reset(device)?;
    let state_custom = &self.states_custom[self.index];
    return Ok((state, state_custom));
  }
}

#[derive(Error, Debug)]
pub enum RenderStateWaitAndResetError {
  #[error("Failed to wait for render complete fence")]
  FenceWaitFail(#[from] FenceWaitError),
  #[error("Failed to reset render complete fence")]
  FenceResetFail(#[from] FenceResetError),
  #[error("Failed to reset command pool")]
  CommandPoolResetFail(#[from] CommandPoolResetError),
}

impl RenderState {
  pub fn wait_and_reset(&mut self, device: &Device) -> Result<(), RenderStateWaitAndResetError> {
    unsafe {
      device.wait_for_fence(self.render_complete_fence, Timeout::Infinite)?;
      device.reset_fence(self.render_complete_fence)?;
      device.reset_command_pool(self.command_pool, false)?;
      // TODO: clear allocated buffers
    }
    Ok(())
  }
}
