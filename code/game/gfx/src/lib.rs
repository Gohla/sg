#![feature(never_type)]

use std::num::NonZeroU32;

use anyhow::{Context, Result};
use ash::vk::{self, ClearColorValue, ClearValue, CommandBuffer, DebugReportFlagsEXT, PhysicalDeviceDescriptorIndexingFeaturesEXT, PipelineStageFlags, RenderPass};
use byte_strings::c_str;
use log::debug;
use raw_window_handle::RawWindowHandle;

use math::prelude::*;
use util::image::{Components, ImageData};
use util::timing::Duration;
use vkw::framebuffer::FramebufferCreateError;
use vkw::prelude::*;

use crate::camera::{CameraInput, CameraSys};
use crate::grid_renderer::{GridRendererSys, GridRenderState};
use crate::texture_def::{TextureDef, TextureDefBuilder};

pub mod grid_renderer;
pub mod texture_def;
pub mod camera;

pub struct Gfx {
  pub instance: Instance,
  pub debug_report: Option<DebugReport>,
  pub surface: Surface,
  pub device: Device,
  pub allocator: Allocator,
  pub transient_command_pool: CommandPool,
  pub swapchain: Swapchain,
  pub pipeline_cache: PipelineCache,
  pub render_pass: RenderPass,
  pub presenter: Presenter,
  pub surface_change_handler: SurfaceChangeHandler,

  pub texture_def: TextureDef,

  pub camera_sys: CameraSys,
  pub grid_render_sys: GridRendererSys,

  pub renderer: Renderer<GameRenderState>,
}

pub struct GameRenderState {
  pub command_buffer: CommandBuffer,
  pub grid_render_sys: GridRenderState,
}

impl Gfx {
  pub fn new(
    require_validation_layer: bool,
    max_frames_in_flight: NonZeroU32,
    window: RawWindowHandle,
    initial_screen_size: ScreenSize
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
        Some(VkVersion::new(1, 1, 0)),
        features_query
      ).with_context(|| "Failed to create VKW instance")?;
      instance
    };
    debug!("{:#?}", &instance.features);

    let debug_report = if require_validation_layer {
      Some(DebugReport::new(&instance, DebugReportFlagsEXT::all() - DebugReportFlagsEXT::INFORMATION).with_context(|| "Failed to create VKW debug report")?)
    } else {
      None
    };
    let surface = Surface::new(&instance, window).with_context(|| "Failed to create VKW surface")?;

    let device = {
      let features_query = {
        let mut query = DeviceFeaturesQuery::new();
        query.require_swapchain_extension();
        query.require_descriptor_indexing_extension();
        query.require_features(PhysicalDeviceFeatures::builder()
          .build());
        query.require_descriptor_indexing_features(PhysicalDeviceDescriptorIndexingFeaturesEXT::builder()
          .shader_sampled_image_array_non_uniform_indexing(true)
          .descriptor_binding_variable_descriptor_count(true)
          .runtime_descriptor_array(true)
          .build());
        query
      };
      Device::new(&instance, features_query, Some(&surface))
        .with_context(|| "Failed to create VKW device")?
    };
    debug!("{:#?}", &device.features);

    let allocator = unsafe { device.create_allocator(&instance) }
      .with_context(|| "Failed to create vk-mem allocator")?;

    let transient_command_pool = unsafe { device.create_command_pool(true, false) }
      .with_context(|| "Failed to create transient command pool")?;

    let swapchain = {
      let features_query = {
        let mut query = SwapchainFeaturesQuery::new();
        query.want_image_count(unsafe { NonZeroU32::new_unchecked(max_frames_in_flight.get() + 1) });
        query.want_present_mode(vec![
          PresentModeKHR::IMMEDIATE,
          PresentModeKHR::MAILBOX,
          PresentModeKHR::FIFO_RELAXED,
          PresentModeKHR::FIFO,
        ]);
        query
      };
      let (width, height) = initial_screen_size.physical.into();
      Swapchain::new(&instance, &device, &surface, features_query, Extent2D { width, height })
        .with_context(|| "Failed to create VKW swapchain")?
    };
    debug!("{:#?}", &swapchain.features);

    let pipeline_cache = unsafe { device.create_pipeline_cache() }
      .with_context(|| "Failed to create Vulkan pipeline cache")?;

    let render_pass = {
      use vk::{AttachmentDescription, AttachmentLoadOp, AttachmentStoreOp, SubpassDescription, AttachmentReference, ImageLayout};
      let attachments = &[
        AttachmentDescription::builder()
          .format(swapchain.features.surface_format.format)
          .samples(SampleCountFlags::TYPE_1)
          .load_op(AttachmentLoadOp::CLEAR)
          .store_op(AttachmentStoreOp::STORE)
          .stencil_load_op(AttachmentLoadOp::DONT_CARE)
          .stencil_store_op(AttachmentStoreOp::DONT_CARE)
          .initial_layout(ImageLayout::UNDEFINED)
          .final_layout(ImageLayout::PRESENT_SRC_KHR)
          .build(),
      ];
      let color_attachments = &[
        AttachmentReference::builder()
          .attachment(0)
          .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
          .build(),
      ];
      let subpasses = &[
        SubpassDescription::builder()
          .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
          .color_attachments(color_attachments)
          .build(),
      ];
      let create_info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        ;
      // CORRECTNESS: slices are taken by pointer but are alive until `create_render_pass` is called.
      unsafe { device.create_render_pass(&create_info) }
        .with_context(|| "Failed to create Vulkan render pass")?
    };
    let framebuffers = Self::create_framebuffers(&device, &swapchain, render_pass)
      .with_context(|| "Failed to create Vulkan framebuffer")?;
    let presenter = Presenter::new(framebuffers)?;

    let surface_change_handler = SurfaceChangeHandler::new();

    let texture_def = {
      let mut builder = TextureDefBuilder::new();
      builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/dark.png"), Some(Components::Components4))?);
      builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/light.png"), Some(Components::Components4))?);
      builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/green.png"), Some(Components::Components4))?);
      unsafe { builder.build(&device, &allocator, transient_command_pool) }?
    };

    let camera_sys = CameraSys::new(initial_screen_size.physical);
    let grid_render_sys = GridRendererSys::new(&device, &allocator, &texture_def, max_frames_in_flight.get(), render_pass, pipeline_cache, transient_command_pool)
      .with_context(|| "Failed to create triangle renderer")?;

    let renderer = Renderer::new(&device, max_frames_in_flight, |state| {
      Ok(GameRenderState {
        command_buffer: unsafe { device.allocate_command_buffer(state.command_pool, false) }?,
        grid_render_sys: grid_render_sys.create_render_state(&device, &allocator)?,
      })
    })?;

    Ok(Self {
      instance,
      surface,
      debug_report,
      device,
      allocator,
      transient_command_pool,
      swapchain,
      pipeline_cache,
      render_pass,
      presenter,
      surface_change_handler,

      texture_def,

      camera_sys,
      grid_render_sys,

      renderer,
    })
  }

  pub fn render_frame(
    &mut self,
    camera_input: CameraInput,
    _extrapolation: f64,
    frame_time: Duration
  ) -> Result<()> {
    // Recreate surface-extent dependent items if needed.
    if let Some(extent) = self.surface_change_handler.query_surface_change(self.swapchain.extent) {
      unsafe {
        self.device.device_wait_idle()
          .with_context(|| "Failed to wait for device idle before recreating surface-extent dependent items")?;
        self.swapchain.recreate(&self.device, &self.surface, extent)
          .with_context(|| "Failed to recreate VKW swapchain")?;
        let framebuffers = Self::create_framebuffers(&self.device, &self.swapchain, self.render_pass)
          .with_context(|| "Failed to recreate Vulkan framebuffer")?;
        self.presenter.recreate(&self.device, framebuffers)
          .with_context(|| "Failed to recreate VKW presenter")?;
      }
    }
    let extent = self.swapchain.extent;

    // Update camera
    self.camera_sys.update(camera_input, frame_time);

    // Acquire render state.
    let (render_state, game_render_state) = self.renderer.next_render_state(&self.device)
      .with_context(|| "Failed to acquire render state")?;
    let command_buffer = game_render_state.command_buffer;

    // Acquire swapchain image.
    let swapchain_image_state = self.presenter.acquire_image_state(&self.swapchain, Some(render_state.image_acquired_semaphore), &mut self.surface_change_handler)
      .with_context(|| "Failed to acquire swapchain image state")?;

    unsafe {
      // Record primary command buffer.
      self.device.begin_command_buffer(command_buffer, true)
        .with_context(|| "Failed to begin command buffer")?;
      self.presenter.set_dynamic_state(&self.device, command_buffer, extent);
      self.device.begin_render_pass(command_buffer, self.render_pass, swapchain_image_state.framebuffer, self.presenter.full_render_area(extent), &[ClearValue { color: ClearColorValue { float32: [0.5, 0.5, 1.0, 1.0] } }]);

      self.grid_render_sys.render(&self.device, &self.texture_def, &game_render_state.grid_render_sys, self.camera_sys.view_projection_matrix(), extent, command_buffer);

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
        Some(render_state.render_complete_fence)
      ).with_context(|| "Failed to submit command buffer")?;
    }

    // Present: take rendered swapchain image and present to the user.
    self.presenter.present(&self.device, &self.swapchain, swapchain_image_state, &[render_state.render_complete_semaphore], &mut self.surface_change_handler)
      .with_context(|| "Failed to present")?;

    Ok(())
  }

  pub fn wait_idle(&self) -> Result<()> {
    Ok(unsafe { self.device.device_wait_idle() }.with_context(|| "Failed to wait for device idle")?)
  }

  pub fn screen_size_changed(&mut self, screen_size: ScreenSize) {
    self.camera_sys.signal_viewport_resize(screen_size.physical);
    let (width, height) = screen_size.physical.into();
    self.surface_change_handler.signal_screen_resize(Extent2D { width, height });
  }


  fn create_framebuffers(device: &Device, swapchain: &Swapchain, render_pass: RenderPass) -> Result<Vec<Framebuffer>, FramebufferCreateError> {
    swapchain.image_views.iter().map(|v| {
      let attachments = &[*v];
      let create_info = vk::FramebufferCreateInfo::builder()
        .render_pass(render_pass)
        .attachments(attachments)
        .width(swapchain.extent.width)
        .height(swapchain.extent.height)
        .layers(1)
        ;
      Ok(unsafe { device.create_framebuffer(&create_info) }?)
    }).collect()
  }
}

impl Drop for Gfx {
  fn drop(&mut self) {
    unsafe {
      self.renderer.destroy(&self.device, |render_state, game_render_state| {
        self.device.free_command_buffer(render_state.command_pool, game_render_state.command_buffer);
        game_render_state.grid_render_sys.destroy(&self.allocator);
      });

      self.grid_render_sys.destroy(&self.device, &self.allocator);

      self.texture_def.destroy(&self.device, &self.allocator);

      self.presenter.destroy(&self.device);
      self.device.destroy_render_pass(self.render_pass);
      self.device.destroy_command_pool(self.transient_command_pool);
      self.allocator.destroy();
      self.device.destroy_pipeline_cache(self.pipeline_cache);
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
