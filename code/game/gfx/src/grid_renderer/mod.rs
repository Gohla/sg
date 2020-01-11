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
  descriptor_set_layout: DescriptorSetLayout,
  pipeline_layout: PipelineLayout,

  descriptor_pool: DescriptorPool,

  vert_shader: ShaderModule,
  frag_shader: ShaderModule,

  pipeline: Pipeline,

  vertex_buffer: BufferAllocation,
  index_buffer: BufferAllocation,
}

impl GridRendererSys {
  pub fn new(
    device: &Device,
    allocator: &Allocator,
    texture_def: &TextureDef,
    render_state_count: u32,
    render_pass: RenderPass,
    pipeline_cache: PipelineCache,
    transient_command_pool: CommandPool,
  ) -> Result<Self> {
    unsafe {
      let descriptor_set_layout = device.create_descriptor_set_layout(&FragmentUniformData::bindings(), &FragmentUniformData::flags())?;
      let pipeline_layout = device.create_pipeline_layout(&[texture_def.descriptor_set_layout, descriptor_set_layout], &[VertexUniformData::push_constant_range()])?;

      let descriptor_pool = device.create_descriptor_pool(render_state_count, &[descriptor_set::uniform_pool_size(render_state_count, false)])?;

      let vert_shader = device.create_shader_module(include_bytes!("../../../../../target/shader/grid_renderer/grid.vert.spv"))?;
      let frag_shader = device.create_shader_module(include_bytes!("../../../../../target/shader/grid_renderer/grid.frag.spv"))?;

      let vertex_bindings = VertexData::bindings();
      let vertex_attributes = VertexData::attributes();

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
          .cull_mode(CullModeFlags::BACK)
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

      let vertex_data = VertexData::triangle_vertex_data();
      let vertex_data_size = size_of::<VertexData>() * vertex_data.len();
      let index_data = VertexData::triangle_index_data();
      let index_data_size = size_of::<u16>() * index_data.len();

      let vertex_staging = allocator.create_staging_buffer(vertex_data_size)?;
      vertex_staging.map(allocator)?.copy_from_slice(&vertex_data);
      let index_staging = allocator.create_staging_buffer(index_data_size)?;
      index_staging.map(allocator)?.copy_from_slice(&index_data);

      let vertex_buffer = allocator.create_static_vertex_buffer(vertex_data_size)?;
      let index_buffer = allocator.create_static_index_buffer(vertex_data_size)?;

      device.allocate_record_submit_wait(transient_command_pool, |command_buffer| {
        device.cmd_copy_buffer(command_buffer, vertex_staging.buffer, vertex_buffer.buffer, &[
          BufferCopy::builder()
            .size(vertex_data_size as u64)
            .build()
        ]);
        device.cmd_copy_buffer(command_buffer, index_staging.buffer, index_buffer.buffer, &[
          BufferCopy::builder()
            .size(index_data_size as u64)
            .build()
        ]);
        Ok(())
      })?;

      index_staging.destroy(allocator);
      vertex_staging.destroy(allocator);

      Ok(Self {
        descriptor_set_layout,
        pipeline_layout,
        descriptor_pool,
        vert_shader,
        frag_shader,
        pipeline,
        vertex_buffer,
        index_buffer,
      })
    }
  }

  pub fn create_render_state(
    &self,
    device: &Device,
    allocator: &Allocator,
  ) -> Result<GridRenderState> {
    unsafe {
      let uniform_buffer = allocator.create_dynamic_uniform_buffer_mapped(size_of::<FragmentUniformData>())?;
      let descriptor_set = device.allocate_descriptor_set(self.descriptor_pool, self.descriptor_set_layout)?;
      DescriptorSetUpdateBuilder::new()
        .add_uniform_buffer_write(
          descriptor_set,
          0,
          0,
          false,
          uniform_buffer.buffer,
          0,
          size_of::<FragmentUniformData>() as DeviceSize
        )
        .do_update(device)
      ;
      Ok(GridRenderState { uniform_buffer, descriptor_set })
    }
  }

  pub fn render(&self, device: &Device, texture_def: &TextureDef, render_state: &GridRenderState, viewport: Extent2D, command_buffer: CommandBuffer) {
    let vertex_uniform_data = VertexUniformData { mvp: Mat4::identity() };
    let fragment_uniform_data = FragmentUniformData::new(viewport.width as f32, viewport.height as f32);
    unsafe {
      render_state.uniform_buffer.get_mapped_data().unwrap(/* CORRECTNESS: buffer is persistently mapped */).copy_from(&fragment_uniform_data);
      device.cmd_bind_pipeline(command_buffer, PipelineBindPoint::GRAPHICS, self.pipeline);
      device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer.buffer], &[0]);
      device.cmd_bind_index_buffer(command_buffer, self.index_buffer.buffer, 0, IndexType::UINT16);
      device.cmd_bind_descriptor_sets(command_buffer, PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &[texture_def.descriptor_set, render_state.descriptor_set], &[]);
      device.cmd_push_constants(command_buffer, self.pipeline_layout, ShaderStageFlags::VERTEX, 0, vertex_uniform_data.as_bytes());
      device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 0);
    }
  }

  pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
    unsafe {
      self.vertex_buffer.destroy(allocator);
      self.index_buffer.destroy(allocator);
      device.destroy_pipeline(self.pipeline);
      device.destroy_pipeline_layout(self.pipeline_layout);
      device.destroy_descriptor_set_layout(self.descriptor_set_layout);
      device.destroy_descriptor_pool(self.descriptor_pool);
      device.destroy_shader_module(self.vert_shader);
      device.destroy_shader_module(self.frag_shader);
    }
  }
}

// Render state

pub struct GridRenderState {
  uniform_buffer: BufferAllocation,
  descriptor_set: DescriptorSet,
}

impl GridRenderState {
  pub fn destroy(&self, allocator: &Allocator) {
    unsafe {
      self.uniform_buffer.destroy(allocator);
    }
  }
}

// Vertex data

#[allow(dead_code)]
#[repr(C)]
struct VertexData {
  pos: Vec2,
}

impl VertexData {
  pub fn bindings() -> Vec<VertexInputBindingDescription> {
    vec![
      VertexInputBindingDescription::builder()
        .binding(0)
        .stride(size_of::<VertexData>() as u32)
        .input_rate(VertexInputRate::VERTEX)
        .build(),
    ]
  }

  pub fn attributes() -> Vec<VertexInputAttributeDescription> {
    vec![
      VertexInputAttributeDescription::builder()
        .location(0)
        .binding(0)
        .format(Format::R32G32_SFLOAT)
        .offset(0)
        .build(),
    ]
  }

  pub fn triangle_vertex_data() -> Vec<VertexData> {
    vec![
      VertexData { pos: Vec2 { x: 0.5, y: -0.5 } },
      VertexData { pos: Vec2 { x: -0.5, y: 0.5 } },
      VertexData { pos: Vec2 { x: 0.5, y: 0.5 } },
      VertexData { pos: Vec2 { x: -0.5, y: -0.5 } },
    ]
  }

  pub fn triangle_index_data() -> Vec<u16> {
    vec![0, 1, 2, 0, 3, 1]
  }
}

// Vertex uniform data (push constant)

#[allow(dead_code)]
#[repr(C)]
struct VertexUniformData {
  mvp: Mat4,
}

impl VertexUniformData {
  pub fn push_constant_range() -> PushConstantRange {
    push_constant::vertex_range(size_of::<Self>() as u32, 0)
  }

  pub unsafe fn as_bytes(&self) -> &[u8] {
    let ptr = self as *const Self;
    let bytes_ptr = ptr as *const u8;
    std::slice::from_raw_parts(bytes_ptr, size_of::<Self>())
  }
}

// Fragment uniform data

#[allow(dead_code)]
#[repr(C)]
struct FragmentUniformData {
  texture_ids: [u32; 64],
  viewport: Vec2,
}

impl FragmentUniformData {
  pub fn bindings() -> Vec<DescriptorSetLayoutBinding> {
    vec![
      descriptor_set::uniform_layout_binding(0, 1, false, ShaderStageFlags::FRAGMENT),
    ]
  }

  pub fn flags() -> Vec<DescriptorBindingFlagsEXT> {
    vec![
      DescriptorBindingFlagsEXT::empty(),
    ]
  }

  pub fn new(width: f32, height: f32) -> Self {
    Self { texture_ids: [2; 64], viewport: Vec2::new(width, height), }
  }
}

