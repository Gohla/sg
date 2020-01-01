use std::num::NonZeroUsize;

use ash::vk::{CommandPool, Fence, Semaphore};
use thiserror::Error;

use crate::command_pool::{CommandPoolCreateError, CommandPoolResetError};
use crate::device::Device;
use crate::sync::{FenceCreateError, FenceResetError, FenceWaitError, SemaphoreCreateError};
use crate::timeout::Timeout;

// Renderer

pub struct Renderer<'a, T> {
  count: usize,
  index: usize,
  states: Box<[RenderState<'a>]>,
  states_custom: Box<[T]>,
}

pub struct RenderState<'a> {
  device: &'a Device<'a>,
  pub command_pool: CommandPool,
  pub image_acquired_semaphore: Semaphore,
  pub render_complete_semaphore: Semaphore,
  pub render_complete_fence: Fence,
  // TODO: track buffer allocations
}

// Creation

#[derive(Error, Debug)]
pub enum RenderCreateError {
  #[error("Failed to create command pool")]
  RenderStateCommandPoolCreateFail(#[from] CommandPoolCreateError),
  #[error("Failed to create command pool")]
  RenderStateImageAcquiredSemaphoreCreateFail(#[source] SemaphoreCreateError),
  #[error("Failed to create command pool")]
  RenderStateRenderCompleteSemaphoreCreateFail(#[source] SemaphoreCreateError),
  #[error("Failed to create command pool")]
  RenderStateRenderCompleteFenceCreateFail(#[from] FenceCreateError),
  #[error("Failed to create command pool")]
  CustomRenderStateCreateFail(#[from] Box<dyn std::error::Error>),
}

impl<'a, T> Renderer<'a, T> {
  pub fn new<F: Fn(&RenderState) -> Result<T, Box<dyn std::error::Error>>>(
    &self,
    device: &'a Device,
    state_count: NonZeroUsize,
    create_custom_state: F
  ) -> Result<Renderer<T>, RenderCreateError> {
    use RenderCreateError::*;
    let count = state_count.get();
    let (states, states_custom) = {
      let mut states = Vec::with_capacity(count);
      let mut states_custom: Vec<T> = Vec::with_capacity(count);
      for _i in 0..count {
        let state = RenderState {
          device,
          command_pool: device.create_command_pool(false, false)?,
          image_acquired_semaphore: device.create_semaphore().map_err(|e| RenderStateImageAcquiredSemaphoreCreateFail(e))?,
          render_complete_semaphore: device.create_semaphore().map_err(|e| RenderStateRenderCompleteSemaphoreCreateFail(e))?,
          render_complete_fence: device.create_fence(true)?,
        };
        let state_custom = create_custom_state(&state)?;
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
}

// API

impl<'a, T> Renderer<'a, T> {
  pub fn next_render_state(&mut self) -> Result<(&mut RenderState<'a>, &T), RenderStateWaitAndResetError> {
    self.index = (self.index + 1) % self.count;
    let state = &mut self.states[self.index];
    state.wait_and_reset()?;
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

impl RenderState<'_> {
  pub fn wait_and_reset(&mut self) -> Result<(), RenderStateWaitAndResetError> {
    unsafe {
      self.device.wait_for_fence(self.render_complete_fence, Timeout::Infinite)?;
      self.device.reset_fence(self.render_complete_fence)?;
      self.device.reset_command_pool(self.command_pool, false)?;
      // TODO: clear allocated buffers
    }
    Ok(())
  }
}

// Implementations

impl Drop for RenderState<'_> {
  fn drop(&mut self) {
    unsafe {
      self.device.destroy_command_pool(self.command_pool);
      self.device.destroy_semaphore(self.image_acquired_semaphore);
      self.device.destroy_semaphore(self.render_complete_semaphore);
      self.device.destroy_fence(self.render_complete_fence);
    }
  }
}