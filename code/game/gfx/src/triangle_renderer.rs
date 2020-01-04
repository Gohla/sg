use std::mem::size_of;

use anyhow::Result;
use ash::version::DeviceV1_0;
use ash::vk::{self, Rect2D};
use ultraviolet::{Vec2, Vec3};

use vkw::prelude::*;
use vkw::shader::ShaderModuleEx;

pub struct TriangleRenderer {
  vert_shader: ShaderModule,
  frag_shader: ShaderModule,
  pipeline_layout: PipelineLayout,
  pipeline: Pipeline,
  vertex_buffer: BufferAllocation,
  index_buffer: BufferAllocation,
}

impl TriangleRenderer {
  pub fn new(
    device: &Device,
    allocator: &Allocator,
    transient_command_pool: CommandPool,
    render_pass: RenderPass,
    pipeline_cache: PipelineCache,
  ) -> Result<Self> {
    unsafe {
      let pipeline_layout = device.create_pipeline_layout(&[], &[])?;

      let vert_shader = device.create_shader_module(include_bytes!("../../../../target/shader/triangle.vert.spv"))?;
      let frag_shader = device.create_shader_module(include_bytes!("../../../../target/shader/triangle.frag.spv"))?;

      let vertex_bindings = VertexData::bindings();
      let vertex_attributes = VertexData::attributes();

      let pipeline = {
        let stages = &[
          vert_shader.create_vertex_shader_stage(None),
          frag_shader.create_fragment_shader_stage(None),
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

      device.allocate_record_submit_wait::<_, !, _>(transient_command_pool, |command_buffer| {
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

      Ok(Self { vert_shader, frag_shader, pipeline_layout, pipeline, vertex_buffer, index_buffer })
    }
  }

  pub fn render(&self, device: &Device, command_buffer: CommandBuffer) {
    unsafe {
      device.cmd_bind_pipeline(command_buffer, PipelineBindPoint::GRAPHICS, self.pipeline);
      device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer.buffer], &[0]);
      device.cmd_bind_index_buffer(command_buffer, self.index_buffer.buffer, 0, IndexType::UINT16);
      device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 0);
    }
  }

  pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
    unsafe {
      self.vertex_buffer.destroy(allocator);
      self.index_buffer.destroy(allocator);
      device.destroy_pipeline(self.pipeline);
      device.destroy_pipeline_layout(self.pipeline_layout);
      device.destroy_shader_module(self.vert_shader);
      device.destroy_shader_module(self.frag_shader);
    }
  }
}

#[allow(dead_code)]
struct VertexData {
  pos: Vec2,
  col: Vec3,
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
      VertexInputAttributeDescription::builder()
        .location(1)
        .binding(0)
        .format(Format::R32G32B32_SFLOAT)
        .offset(size_of::<Vec2>() as u32)
        .build()
    ]
  }

  pub fn triangle_vertex_data() -> Vec<VertexData> {
    vec![
      VertexData { pos: Vec2 { x: 0.5, y: -0.5 }, col: Vec3 { x: 1.0, y: 0.0, z: 0.0 } },
      VertexData { pos: Vec2 { x: -0.5, y: 0.5 }, col: Vec3 { x: 0.0, y: 1.0, z: 0.0 } },
      VertexData { pos: Vec2 { x: 0.5, y: 0.5 }, col: Vec3 { x: 0.0, y: 0.0, z: 1.0 } },
      VertexData { pos: Vec2 { x: -0.5, y: -0.5 }, col: Vec3 { x: 1.0, y: 1.0, z: 1.0 } },
    ]
  }

  pub fn triangle_index_data() -> Vec<u16> {
    vec![0, 1, 2, 0, 3, 1]
  }
}
