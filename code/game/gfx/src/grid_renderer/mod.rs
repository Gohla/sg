use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::size_of;

use anyhow::Result;
use ash::version::DeviceV1_0;
use ash::vk;
use legion::world::World;
use ultraviolet::{Mat4, Vec2, Vec3};

use sim::{InGrid, InGridPosition, InGridRotation};
use util::idx_assigner::Item;
use vkw::prelude::*;
use vkw::shader::ShaderModuleEx;

use crate::texture_def::{TextureDef, TextureIdx};

// Grid renderer component

#[repr(C)]
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct InGridRender(pub TextureIdx);

// Grid chunks

#[repr(C)]
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
struct InGridChunk { x: i8, y: i8 }

// Grid renderer system

pub struct GridRendererSys {
  pipeline_layout: PipelineLayout,

  vert_shader: ShaderModule,
  frag_shader: ShaderModule,

  pipeline: Pipeline,

  quads_vertex_buffer: BufferAllocation,
  quads_index_buffer: BufferAllocation,
}

const GRID_LENGTH: usize = 8;
const GRID_LENGTH_I32: i32 = GRID_LENGTH as i32;
const GRID_TILE_COUNT: usize = GRID_LENGTH * GRID_LENGTH;

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

      let vertex_bindings = {
        let mut vec = QuadsVertexData::bindings();
        vec.extend(TextureUVVertexData::bindings());
        vec
      };
      let vertex_attributes = {
        let mut vec = QuadsVertexData::attributes();
        vec.extend(TextureUVVertexData::attributes());
        vec
      };

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
      let quads_vertices = QuadsVertexData::create_vertices();
      let quads_indices = QuadsIndexData::create_indices();
      let vertex_staging = allocator.create_staging_buffer_from_slice(&quads_vertices)?;
      let index_staging = allocator.create_staging_buffer_from_slice(&quads_indices)?;
      let quads_vertex_buffer = allocator.create_gpu_vertex_buffer(QuadsVertexData::vertices_size())?;
      let quads_index_buffer = allocator.create_gpu_index_buffer(QuadsIndexData::indices_size())?;
      device.allocate_record_submit_wait(transient_command_pool, |command_buffer| {
        device.cmd_copy_buffer(command_buffer, vertex_staging.buffer, quads_vertex_buffer.buffer, &[
          BufferCopy::builder()
            .size(QuadsVertexData::vertices_size() as u64)
            .build()
        ]);
        device.cmd_copy_buffer(command_buffer, index_staging.buffer, quads_index_buffer.buffer, &[
          BufferCopy::builder()
            .size(QuadsIndexData::indices_size() as u64)
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
      })
    }
  }

  pub fn create_render_state(
    &self,
    _device: &Device,
    _allocator: &Allocator,
  ) -> Result<GridRenderState> {
    Ok(GridRenderState { grids: HashMap::default() })
  }

  pub fn render(
    &self,
    device: &Device,
    allocator: &Allocator,
    command_buffer: CommandBuffer,
    texture_def: &TextureDef,
    render_state: &mut GridRenderState,
    world: &mut World,
    view_projection: Mat4,
  ) -> Result<()> {
    use legion::borrow::Ref;
    use legion::prelude::*;

    // Update chunks of grid entities.
    let entity_command_buffer = legion::command::CommandBuffer::default();
    let chunk_query = Read::<InGridPosition>::query()
      .filter(tag::<InGrid>() & component::<InGridRender>() & changed::<InGridPosition>());
    for i in chunk_query.iter_entities_immutable(world) {
      let (entity, pos): (_, Ref<InGridPosition>) = i;
      let (x, y) = ((pos.x / GRID_LENGTH_I32) as i8, (pos.y / GRID_LENGTH_I32) as i8);
      entity_command_buffer.add_tag(entity, InGridChunk { x, y })
    }
    entity_command_buffer.write(world);

    // Update chunk buffers with texture UVs.
    let update_query = <(Read<InGridPosition>, Read<InGridRotation>, Read<InGridRender>)>::query()
      .filter(tag::<InGrid>() & tag::<InGridChunk>());
    for chunk in update_query.iter_chunks_immutable(world) {
      let in_grid: &InGrid = chunk.tag().unwrap();
      let grid_chunk: &InGridChunk = chunk.tag().unwrap();
      let buffer_allocation = match render_state.grids.entry((*in_grid, *grid_chunk)) {
        Entry::Occupied(e) => {
          e.into_mut()
        }
        Entry::Vacant(e) => {
          let buffer_allocation = unsafe { allocator.create_cpugpu_vertex_buffer(TextureUVVertexData::uv_size())? };
          e.insert(buffer_allocation)
        }
      };

      {
        let map = unsafe { buffer_allocation.map(allocator)? };
        let buffer_slice = unsafe { std::slice::from_raw_parts_mut(map.ptr() as *mut TextureUVVertexData, TextureUVVertexData::uv_count()) };
        let positions = chunk.components::<InGridPosition>().unwrap();
        let rotations = chunk.components::<InGridRotation>().unwrap();
        let renderers = chunk.components::<InGridRender>().unwrap();
        for ((pos, _rot), render) in positions.iter().zip(rotations.iter()).zip(renderers.iter()) {
          let texture_index = render.0.into_idx() as f32;
          let (x,y) = dbg!((pos.x % GRID_LENGTH_I32, pos.y % GRID_LENGTH_I32)); // TODO: positions can be negative, have to deal with that.
          let slice_index = dbg!(((x / GRID_LENGTH_I32) + (y * GRID_LENGTH_I32) * 4)) as usize;
          buffer_slice[slice_index + 0] = TextureUVVertexData::new(0.0, 1.0, texture_index);
          buffer_slice[slice_index + 1] = TextureUVVertexData::new(1.0, 1.0, texture_index);
          buffer_slice[slice_index + 2] = TextureUVVertexData::new(0.0, 0.0, texture_index);
          buffer_slice[slice_index + 3] = TextureUVVertexData::new(1.0, 0.0, texture_index);
        }
        allocator.flush_allocation(&buffer_allocation.allocation, 0, ash::vk::WHOLE_SIZE as usize)?;
      }
    }

    // Issue bind and draw commands.
    unsafe {
      device.cmd_bind_pipeline(command_buffer, PipelineBindPoint::GRAPHICS, self.pipeline);
      device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.quads_vertex_buffer.buffer], &[0]);
      device.cmd_bind_index_buffer(command_buffer, self.quads_index_buffer.buffer, 0, QuadsIndexData::index_type());
      device.cmd_bind_descriptor_sets(command_buffer, PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &[texture_def.descriptor_set], &[]);
      for ((_in_grid, in_grid_chunk), buffer_allocation) in render_state.grids.iter() {
        let model = Mat4::from_translation(Vec3 { x: in_grid_chunk.x as f32 * 8.0, y: in_grid_chunk.y as f32 * 8.0, z: 0.0 });
        let mvp_uniform_data = MVPUniformData(view_projection * model);
        device.cmd_push_constants(command_buffer, self.pipeline_layout, ShaderStageFlags::VERTEX, 0, mvp_uniform_data.as_bytes());
        device.cmd_bind_vertex_buffers(command_buffer, 1, &[buffer_allocation.buffer], &[0]);
        device.cmd_draw_indexed(command_buffer, QuadsIndexData::index_count() as u32, 1, 0, 0, 0);
      }
    }

    Ok(())
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

pub struct GridRenderState {
  grids: HashMap<(InGrid, InGridChunk), BufferAllocation>
}

impl GridRenderState {
  pub fn destroy(&self, _allocator: &Allocator) {}
}

// Quads vertex data (GPU buffer, immutable)

#[allow(dead_code)]
#[repr(C)]
struct QuadsVertexData(Vec2);

#[allow(dead_code)]
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


  fn vertex_count() -> usize { GRID_TILE_COUNT * 4 }

  fn create_vertices() -> Vec<Self> {
    let mut vec = Vec::with_capacity(Self::vertex_count());
    let half = GRID_LENGTH as i32 / 2;
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

  fn vertices_size() -> usize { Self::vertex_count() * size_of::<Self>() }
}

// Quads index data (GPU buffer, immutable)

#[allow(dead_code)]
#[repr(C)]
struct QuadsIndexData(u16);

#[allow(dead_code)]
impl QuadsIndexData {
  #[inline]
  fn index_type() -> IndexType { IndexType::UINT16 }


  fn index_count() -> usize { GRID_TILE_COUNT * 6 }

  fn create_indices() -> Vec<QuadsIndexData> {
    let mut vec = Vec::with_capacity(Self::index_count());
    for i in 0..GRID_TILE_COUNT as u16 {
      vec.push(Self((i * 4) + 0));
      vec.push(Self((i * 4) + 1));
      vec.push(Self((i * 4) + 2));
      vec.push(Self((i * 4) + 1));
      vec.push(Self((i * 4) + 3));
      vec.push(Self((i * 4) + 2));
    }
    vec
  }

  fn indices_size() -> usize { Self::index_count() * size_of::<Self>() }
}

// Texture UV vertex data (CPU-GPU buffer, mutable)

#[allow(dead_code)]
#[repr(C)]
struct TextureUVVertexData {
  u: f32,
  v: f32,
  i: f32,
}

#[allow(dead_code)]
impl TextureUVVertexData {
  fn bindings() -> Vec<VertexInputBindingDescription> {
    vec![
      VertexInputBindingDescription::builder()
        .binding(1)
        .stride(size_of::<Self>() as u32)
        .input_rate(VertexInputRate::VERTEX)
        .build(),
    ]
  }

  fn attributes() -> Vec<VertexInputAttributeDescription> {
    vec![
      VertexInputAttributeDescription::builder()
        .location(1)
        .binding(1)
        .format(Format::R32G32B32_SFLOAT)
        .offset(0)
        .build(),
    ]
  }


  fn new(u: f32, v: f32, i: f32) -> Self {
    Self { u, v, i }
  }

  fn uv_count() -> usize { GRID_TILE_COUNT * 4 }

  fn uv_size() -> usize { Self::uv_count() * size_of::<Self>() }
}


// MVP (model-view-projection matrix) uniform data (push constant, mutable)

#[allow(dead_code)]
#[repr(C)]
struct MVPUniformData(Mat4);


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
