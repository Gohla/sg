use std::mem::size_of;

use anyhow::Result;
use ash::version::DeviceV1_0;
use ash::vk;
use ultraviolet::{Mat4, Vec2};

use vkw::prelude::*;
use vkw::shader::ShaderModuleEx;

use crate::texture_def::{TextureDef, TextureIdx};

// Grid renderer component

pub struct InGridRender(TextureIdx);

// Grid renderer system

pub struct GridRendererSys {
  pipeline_layout: PipelineLayout,

  vert_shader: ShaderModule,
  frag_shader: ShaderModule,

  pipeline: Pipeline,

  quads_vertex_buffer: BufferAllocation,
  quads_index_buffer: BufferAllocation,
  quads_indices_count: usize,
}

impl GridRendererSys {
  pub fn new(
    device: &Device,
    allocator: &Allocator,
    texture_def: &TextureDef,
    _render_state_count: u32,
    render_pass: RenderPass,
    pipeline_cache: PipelineCache,
    transient_command_pool: CommandPool,
  ) -> Result<Self> {
    unsafe {
      let pipeline_layout = device.create_pipeline_layout(&[texture_def.descriptor_set_layout], &[MVPUniformData::push_constant_range()])?;

      let vert_shader = device.create_shader_module(include_bytes!("../../../../../target/shader/grid_renderer/grid.vert.spv"))?;
      let frag_shader = device.create_shader_module(include_bytes!("../../../../../target/shader/grid_renderer/grid.frag.spv"))?;

      let vertex_bindings = QuadsVertexData::bindings();
      let vertex_attributes = QuadsVertexData::attributes();

      let pipeline = {
        let stages = &[
          vert_shader.create_vertex_shader_stage(None).build(),
          frag_shader.create_fragment_shader_stage(None).build(),
        ];
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
          .vertex_binding_descriptions(&vertex_bindings)
          .vertex_attribute_descriptions(&vertex_attributes)
          ;
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
          .topology(PrimitiveTopology::TRIANGLE_LIST)
          .primitive_restart_enable(false)
          ;
        let viewports = &[vk::Viewport::builder().max_depth(1.0).build()];
        let scissors = &[Rect2D::default()];
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
          .viewports(viewports)
          .scissors(scissors)
          ;
        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
          .depth_clamp_enable(false)
          .rasterizer_discard_enable(false)
          .polygon_mode(PolygonMode::FILL)
          .cull_mode(CullModeFlags::NONE)
          .front_face(FrontFace::COUNTER_CLOCKWISE)
          .line_width(1.0)
          ;
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
          .rasterization_samples(SampleCountFlags::TYPE_1)
          .min_sample_shading(1.0)
          ;
        let color_blend_state_attachments = &[vk::PipelineColorBlendAttachmentState::builder()
          .blend_enable(true)
          .src_color_blend_factor(BlendFactor::SRC_ALPHA)
          .dst_color_blend_factor(BlendFactor::ONE_MINUS_SRC_ALPHA)
          .color_blend_op(BlendOp::ADD)
          .src_alpha_blend_factor(BlendFactor::SRC_ALPHA)
          .dst_alpha_blend_factor(BlendFactor::ONE_MINUS_SRC_ALPHA)
          .alpha_blend_op(BlendOp::ADD)
          .color_write_mask(ColorComponentFlags::all())
          .build()
        ];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
          .logic_op_enable(false)
          .logic_op(LogicOp::CLEAR)
          .attachments(color_blend_state_attachments)
          .blend_constants([0.0, 0.0, 0.0, 0.0])
          ;
        let dynamic_states = &[DynamicState::VIEWPORT, DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(dynamic_states);
        let create_info = vk::GraphicsPipelineCreateInfo::builder()
          .stages(stages)
          .vertex_input_state(&vertex_input_state)
          .input_assembly_state(&input_assembly_state)
          .viewport_state(&viewport_state)
          .rasterization_state(&rasterization_state)
          .multisample_state(&multisample_state)
          .color_blend_state(&color_blend_state)
          .dynamic_state(&dynamic_state)
          .layout(pipeline_layout)
          .render_pass(render_pass)
          ;
        // CORRECTNESS: slices are taken by pointer but are alive until `create_graphics_pipeline` is called.
        device.create_graphics_pipeline(pipeline_cache, &create_info)?
      };

      // Create GPU buffers for immutable quad vertex and index data.
      let quads_vertices = QuadsVertexData::create_vertices(8);
      let quads_vertices_count = quads_vertices.len();
      let quads_vertices_size = quads_vertices_count * size_of::<QuadsVertexData>();
      let quads_indices = QuadsIndexData::create_indices(8);
      let quads_indices_count = quads_indices.len();
      let quads_indices_size = quads_indices_count * size_of::<QuadsIndexData>();
      let vertex_staging = allocator.create_staging_buffer_from_slice(&quads_vertices)?;
      let index_staging = allocator.create_staging_buffer_from_slice(&quads_indices)?;
      let quads_vertex_buffer = allocator.create_gpu_vertex_buffer(quads_vertices_size)?;
      let quads_index_buffer = allocator.create_gpu_index_buffer(quads_indices_size)?;
      device.allocate_record_submit_wait(transient_command_pool, |command_buffer| {
        device.cmd_copy_buffer(command_buffer, vertex_staging.buffer, quads_vertex_buffer.buffer, &[
          BufferCopy::builder()
            .size(quads_vertices_size as u64)
            .build()
        ]);
        device.cmd_copy_buffer(command_buffer, index_staging.buffer, quads_index_buffer.buffer, &[
          BufferCopy::builder()
            .size(quads_indices_size as u64)
            .build()
        ]);
        Ok(())
      })?;
      index_staging.destroy(allocator);
      vertex_staging.destroy(allocator);

      Ok(Self {
        pipeline_layout,
        vert_shader,
        frag_shader,
        pipeline,
        quads_vertex_buffer,
        quads_index_buffer,
        quads_indices_count,
      })
    }
  }

  pub fn create_render_state(
    &self,
    _device: &Device,
    _allocator: &Allocator,
  ) -> Result<GridRenderState> {
    Ok(GridRenderState {})
  }

  pub fn render(
    &self,
    device: &Device,
    texture_def: &TextureDef,
    _render_state: &GridRenderState,
    view_projection: Mat4,
    command_buffer: CommandBuffer
  ) {
    let vertex_uniform_data = MVPUniformData { mvp: view_projection };
    unsafe {
      device.cmd_bind_pipeline(command_buffer, PipelineBindPoint::GRAPHICS, self.pipeline);
      device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.quads_vertex_buffer.buffer], &[0]);
      device.cmd_bind_index_buffer(command_buffer, self.quads_index_buffer.buffer, 0, QuadsIndexData::index_type());
      device.cmd_bind_descriptor_sets(command_buffer, PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &[texture_def.descriptor_set], &[]);
      device.cmd_push_constants(command_buffer, self.pipeline_layout, ShaderStageFlags::VERTEX, 0, vertex_uniform_data.as_bytes());
      device.cmd_draw_indexed(command_buffer, self.quads_indices_count as u32, 1, 0, 0, 0);
    }
  }

  pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
    unsafe {
      self.quads_vertex_buffer.destroy(allocator);
      self.quads_index_buffer.destroy(allocator);
      device.destroy_pipeline(self.pipeline);
      device.destroy_pipeline_layout(self.pipeline_layout);
      device.destroy_shader_module(self.vert_shader);
      device.destroy_shader_module(self.frag_shader);
    }
  }
}

// Render state

pub struct GridRenderState {}

impl GridRenderState {
  pub fn destroy(&self, _allocator: &Allocator) {}
}

// Quads vertex data (GPU buffer, immutable)

#[allow(dead_code)]
#[repr(C)]
struct QuadsVertexData(Vec2);

impl QuadsVertexData {
  fn bindings() -> Vec<VertexInputBindingDescription> {
    vec![
      VertexInputBindingDescription::builder()
        .binding(0)
        .stride(size_of::<Self>() as u32)
        .input_rate(VertexInputRate::VERTEX)
        .build(),
    ]
  }

  fn attributes() -> Vec<VertexInputAttributeDescription> {
    vec![
      VertexInputAttributeDescription::builder()
        .location(0)
        .binding(0)
        .format(Format::R32G32_SFLOAT)
        .offset(0)
        .build(),
    ]
  }

  fn create_vertices(grid_length: u32) -> Vec<Self> {
    let quad_count = grid_length * grid_length;
    let vertices_count = quad_count * 4;
    let mut vec = Vec::with_capacity(vertices_count as usize);
    let half = grid_length as i32 / 2;
    let half_neg = -half;
    for x in half_neg..half {
      let x = x as f32;
      for y in half_neg..half {
        let y = y as f32;
        vec.push(Self(Vec2::new(x - 0.5, y - 0.5)));
        vec.push(Self(Vec2::new(x + 0.5, y - 0.5)));
        vec.push(Self(Vec2::new(x - 0.5, y + 0.5)));
        vec.push(Self(Vec2::new(x + 0.5, y + 0.5)));
      }
    }
    vec
  }
}

// Quads index data (GPU buffer, immutable)

#[allow(dead_code)]
#[repr(C)]
struct QuadsIndexData(u16);

impl QuadsIndexData {
  #[inline]
  pub fn index_type() -> IndexType { IndexType::UINT16 }

  pub fn create_indices(grid_length: usize) -> Vec<QuadsIndexData> {
    let mut vec = Vec::with_capacity(grid_length * grid_length * 6);
    for i in 0..(grid_length * grid_length) as u16 {
      vec.push(Self((i * 4) + 0));
      vec.push(Self((i * 4) + 1));
      vec.push(Self((i * 4) + 2));
      vec.push(Self((i * 4) + 1));
      vec.push(Self((i * 4) + 3));
      vec.push(Self((i * 4) + 2));
    }
    vec
  }
}

// Texture UV vertex data (CPU-GPU buffer, mutable)

#[allow(dead_code)]
#[repr(C)]
struct TextureUVVertexData {
  tex: Vec2,
}

#[allow(dead_code)]
impl TextureUVVertexData {
  pub fn bindings() -> Vec<VertexInputBindingDescription> {
    vec![
      VertexInputBindingDescription::builder()
        .binding(1)
        .stride(size_of::<Self>() as u32)
        .input_rate(VertexInputRate::VERTEX)
        .build(),
    ]
  }

  pub fn attributes() -> Vec<VertexInputAttributeDescription> {
    vec![
      VertexInputAttributeDescription::builder()
        .location(1)
        .binding(0)
        .format(Format::R32G32_SFLOAT)
        .offset(0)
        .build(),
    ]
  }

  pub fn new(u: f32, v: f32) -> Self {
    Self { tex: Vec2::new(u, v) }
  }
}


// MVP (model-view-projection matrix) uniform data (push constant, mutable)

#[allow(dead_code)]
#[repr(C)]
struct MVPUniformData {
  mvp: Mat4,
}

impl MVPUniformData {
  pub fn push_constant_range() -> PushConstantRange {
    push_constant::vertex_range(size_of::<Self>() as u32, 0)
  }

  pub unsafe fn as_bytes(&self) -> &[u8] {
    let ptr = self as *const Self;
    let bytes_ptr = ptr as *const u8;
    std::slice::from_raw_parts(bytes_ptr, size_of::<Self>())
  }
}
