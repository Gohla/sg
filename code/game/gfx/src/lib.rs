use std::num::NonZeroU32;

use anyhow::{Context, Result};
use ash::vk::{self, ClearColorValue, ClearValue, CommandBuffer, PipelineStageFlags, RenderPass};
use byte_strings::c_str;
use raw_window_handle::RawWindowHandle;

use vkw::command_pool::AllocateCommandBuffersError;
use vkw::framebuffer::FramebufferCreateError;
use vkw::prelude::*;

pub struct Gfx {
  pub instance: Instance,
  pub debug_report: Option<DebugReport>,
  pub surface: Surface,
  pub device: Device,
  pub swapchain: Swapchain,
  pub renderer: Renderer<GameRenderState>,
  pub render_pass: RenderPass,
  pub presenter: Presenter,
  pub surface_change_handler: SurfaceChangeHandler,
}

pub struct GameRenderState {
  pub command_buffer: CommandBuffer,
}

impl CustomRenderState for GameRenderState {
  unsafe fn destroy(&mut self, device: &Device, render_state: &RenderState) {
    device.free_command_buffer(render_state.command_pool, self.command_buffer);
  }
}

impl Gfx {
  pub fn new<S: Into<(u32, u32)>>(
    require_validation_layer: bool,
    max_frames_in_flight: NonZeroU32,
    window: RawWindowHandle,
    surface_size: S
  ) -> Result<Gfx> {
    let entry = Entry::new()
      .with_context(|| "Failed to create VKW entry")?;
    let instance = {
      let features_query = {
        let mut query = InstanceFeaturesQuery::new();
        if require_validation_layer {
          query.require_validation_layer();
        }
        query.require_surface();
        query
      };
      let instance = Instance::new(
        entry,
        Some(c_str!("SG")),
        None,
        Some(c_str!("SG GFX")),
        None,
        None,
        features_query
      ).with_context(|| "Failed to create VKW instance")?;
      instance
    };

    let debug_report = if require_validation_layer {
      Some(DebugReport::new(&instance).with_context(|| "Failed to create VKW debug report")?)
    } else {
      None
    };
    let surface = Surface::new(&instance, window).with_context(|| "Failed to create VKW surface")?;

    let device = {
      let features_query = {
        let mut query = DeviceFeaturesQuery::new();
        query.require_swapchain_extension();
        query.require_features(PhysicalDeviceFeatures::builder().build());
        query
      };
      Device::new(&instance, features_query, Some(&surface))
        .with_context(|| "Failed to create VKW device")?
    };

    let swapchain = {
      let features_query = {
        let mut query = SwapchainFeaturesQuery::new();
        query.want_image_count(max_frames_in_flight);
        query.want_present_mode(vec![
          PresentModeKHR::IMMEDIATE,
          PresentModeKHR::MAILBOX,
          PresentModeKHR::FIFO_RELAXED,
          PresentModeKHR::FIFO,
        ]);
        query
      };
      let (width, height) = surface_size.into();
      Swapchain::new(&instance, &device, &surface, features_query, Extent2D { width, height })
        .with_context(|| "Failed to create VKW swapchain")?
    };

    let renderer = Renderer::new::<AllocateCommandBuffersError, _>(&device, max_frames_in_flight, |state| {
      Ok(GameRenderState {
        command_buffer: unsafe { device.allocate_command_buffer(state.command_pool, false) }?
      })
    })?;

    let render_pass = {
      use vk::{AttachmentDescription, AttachmentLoadOp, AttachmentStoreOp, SubpassDescription, PipelineBindPoint, AttachmentReference, ImageLayout};
      let attachments = vec![
        AttachmentDescription::builder()
          .format(swapchain.features.surface_format.format)
          .load_op(AttachmentLoadOp::CLEAR)
          .store_op(AttachmentStoreOp::STORE)
          .stencil_load_op(AttachmentLoadOp::DONT_CARE)
          .stencil_store_op(AttachmentStoreOp::DONT_CARE)
          .initial_layout(ImageLayout::UNDEFINED)
          .final_layout(ImageLayout::PRESENT_SRC_KHR)
          .build(),
      ];
      let color_attachments = vec![
        AttachmentReference::builder()
          .attachment(0)
          .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
          .build()
      ];
      let subpasses = vec![
        SubpassDescription::builder()
          .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
          .color_attachments(&color_attachments)
          .build()
      ];
      let create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        ;
      unsafe { device.create_render_pass(&create_info) }
        .with_context(|| "Failed to create Vulkan render pass")?
    };
    let framebuffers = Self::create_framebuffers(&device, &swapchain, &render_pass)
      .with_context(|| "Failed to create Vulkan framebuffer")?;
    let presenter = Presenter::new(framebuffers)?;

    let surface_change_handler = SurfaceChangeHandler::new();

    Ok(Self {
      instance,
      surface,
      debug_report,
      device,
      swapchain,
      renderer,
      render_pass,
      presenter,
      surface_change_handler,
    })
  }

  pub fn render_frame(&mut self) -> Result<()> {
    // Recreate surface-extent dependent items if needed.
    if let Some(extent) = self.surface_change_handler.query_surface_change(self.swapchain.extent) {
      unsafe {
        self.swapchain.recreate(&self.device, &self.surface, extent)
          .with_context(|| "Failed to recreate VKW swapchain")?;
        let framebuffers = Self::create_framebuffers(&self.device, &self.swapchain, &self.render_pass)
          .with_context(|| "Failed to recreate Vulkan framebuffer")?;
        self.presenter.recreate(&self.device, framebuffers)
          .with_context(|| "Failed to recreate VKW presenter")?;
      }
    }
    let extent = self.swapchain.extent;

    // Acquire render state.
    let (render_state, game_render_state) = self.renderer.next_render_state(&self.device)
      .with_context(|| "Failed to acquire render state")?;
    let command_buffer = game_render_state.command_buffer;

    // Acquire swapchain image.
    let swapchain_image_state = self.presenter.acquire_image_state(&self.swapchain, Some(render_state.image_acquired_semaphore), &self.surface_change_handler)
      .with_context(|| "Failed to acquire swapchain image state")?;

    unsafe {
      // Record primary command buffer.
      self.device.begin_command_buffer(command_buffer, true)
        .with_context(|| "Failed to begin command buffer")?;
      self.presenter.set_dynamic_state(&self.device, command_buffer, extent);
      self.device.begin_render_pass(command_buffer, self.render_pass, swapchain_image_state.framebuffer, self.presenter.full_render_area(extent), &[ClearValue { color: ClearColorValue { float32: [0.5, 0.5, 1.0, 1.0] } }]);

      // Done recording primary command buffer.
      self.device.end_render_pass(command_buffer);
      self.device.end_command_buffer(command_buffer)
        .with_context(|| "Failed to end command buffer")?;

      // Submit command buffer: render to swapchain image.
      self.device.submit_command_buffer(
        command_buffer,
        &[render_state.image_acquired_semaphore],
        &[PipelineStageFlags::TOP_OF_PIPE],
        &[render_state.render_complete_semaphore],
        render_state.render_complete_fence
      ).with_context(|| "Failed to submit command buffer")?;
    }

    // Present: take rendered swapchain image and present to the user.
    self.presenter.present(&self.device, &self.swapchain, swapchain_image_state, &[render_state.render_complete_semaphore], &self.surface_change_handler)
      .with_context(|| "Failed to present")?;

    Ok(())
  }

  pub fn surface_size_changed<S: Into<(u32, u32)>>(&mut self, surface_size: S) {
    let (width, height) = surface_size.into();
    self.surface_change_handler.signal_surface_resize(Extent2D { width, height });
  }


  fn create_framebuffers(device: &Device, swapchain: &Swapchain, render_pass: &RenderPass) -> Result<Vec<Framebuffer>, FramebufferCreateError> {
    swapchain.image_views.iter().map(|v| {
      let attachments = vec![*v];
      let create_info = vk::FramebufferCreateInfo::builder()
        .render_pass(*render_pass)
        .attachments(&attachments)
        .width(swapchain.extent.width)
        .height(swapchain.extent.height)
        .layers(1)
        .build()
        ;
      Ok(device.create_framebuffer(&create_info)?)
    }).collect()
  }
}

impl Drop for Gfx {
  fn drop(&mut self) {
    unsafe {
      self.presenter.destroy(&self.device);
      self.device.destroy_render_pass(self.render_pass);
      self.renderer.destroy(&self.device);
      self.swapchain.destroy(&self.device);
      self.device.destroy();
      self.surface.destroy();
      if let Some(debug_report) = &mut self.debug_report {
        debug_report.destroy();
      }
      self.instance.destroy();
    }
  }
}
