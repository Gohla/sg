use std::num::NonZeroU32;

use anyhow::{Context, Result};
use ash::vk::{self, CommandBuffer, RenderPass};
use byte_strings::c_str;
use raw_window_handle::RawWindowHandle;

use vkw::command_pool::AllocateCommandBuffersError;
use vkw::prelude::*;

pub struct Gfx {
  pub instance: Instance,
  pub debug_report: Option<DebugReport>,
  pub surface: Surface,
  pub device: Device,
  pub renderer: Renderer<GameRenderState>,
  pub render_pass: RenderPass,
  pub presenter: Presenter,
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

    let presenter = Presenter::new(&device, swapchain, &render_pass, &[])?;

    Ok(Self {
      instance,
      surface,
      debug_report,
      device,
      renderer,
      render_pass,
      presenter,
    })
  }

  pub fn update(&mut self) -> Result<()> {
    if self.presenter.should_recreate() {
      unsafe { self.device.wait_idle() }?;
      self.presenter.recreate(&self.device, &self.surface, &self.render_pass, &[])?;
    }

    Ok(())
  }

  pub fn surface_size_changed<S: Into<(u32, u32)>>(&mut self, surface_size: S) {
    let (width, height) = surface_size.into();
    self.presenter.signal_surface_resize(Extent2D { width, height });
  }
}

impl Drop for Gfx {
  fn drop(&mut self) {
    unsafe {
      self.presenter.destroy(&self.device);
      self.device.destroy_render_pass(self.render_pass);
      self.renderer.destroy(&self.device);
      self.device.destroy();
      self.surface.destroy();
      if let Some(debug_report) = &mut self.debug_report {
        debug_report.destroy();
      }
      self.instance.destroy();
    }
  }
}
