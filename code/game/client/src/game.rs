use anyhow::Result;
use legion::prelude::{IntoQuery, Read, tag_value};
use ultraviolet::{Isometry2, Rotor2, Vec2, Vec3};

use gfx::Gfx;
use gfx::grid_renderer::GridTileRender;
use gfx::texture_def::{TextureDefBuilder, TextureIdx};
use sim::prelude::*;
use util::image::{Components, ImageData};

pub struct GameDef {
  pub grid_tile_textures: Vec<TextureIdx>,
}

impl GameDef {
  pub fn new() -> Result<(GameDef, TextureDefBuilder)> {
    let mut texture_def_builder = TextureDefBuilder::new();
    let tex1 = texture_def_builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/dark.png"), Some(Components::Components4))?);
    let tex2 = texture_def_builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/light.png"), Some(Components::Components4))?);
    let tex3 = texture_def_builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/green.png"), Some(Components::Components4))?);
    let game_def = GameDef { grid_tile_textures: vec![tex1, tex2, tex3] };
    Ok((game_def, texture_def_builder))
  }
}

pub struct Game {
  _game_def: GameDef,
  grid: Entity,
}

impl Game {
  pub fn new(game_def: GameDef, sim: &mut Sim, gfx: &mut Gfx) -> Self {
    let world = &mut sim.world;
    let grid = world.insert((Grid, ), vec![
      (WorldTransform::new(0.0, 0.0, 0.0), WorldDynamics::new(0.0, 0.0, 0.0)),
    ])[0];

    let tex1 = game_def.grid_tile_textures[0];
    let tex2 = game_def.grid_tile_textures[1];
    let tex3 = game_def.grid_tile_textures[2];

    world.insert((InGrid::new(grid), ), vec![
      (GridPosition::new(0, 0), GridOrientation::default(), GridTileRender(tex1)),
      (GridPosition::new(-1, 0), GridOrientation::default(), GridTileRender(tex2)),
      (GridPosition::new(0, -1), GridOrientation::default(), GridTileRender(tex1)),
      (GridPosition::new(-1, -1), GridOrientation::default(), GridTileRender(tex1)),
      (GridPosition::new(0, 7), GridOrientation::default(), GridTileRender(tex2)),
      (GridPosition::new(0, 8), GridOrientation::default(), GridTileRender(tex3)),
    ]);

    gfx.camera_sys.set_position(Vec3::new(-0.5, -0.5, 1.0));
    gfx.camera_sys.set_zoom(33.0);

    Self { _game_def: game_def, grid }
  }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct GameInput {
  pub debug: GameDebugInput,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct GameDebugInput {
  pub grid_linear_velocity_x_inc: bool,
  pub grid_linear_velocity_x_dec: bool,
  pub grid_linear_velocity_y_inc: bool,
  pub grid_linear_velocity_y_dec: bool,
  pub grid_angular_velocity_inc: bool,
  pub grid_angular_velocity_dec: bool,
  pub grid_randomize: bool,
  pub grid_reset: bool,
}

impl Game {
  pub fn simulate_tick(&mut self, input: GameInput, sim: &mut Sim, _gfx: &mut Gfx, ) {
    {
      let mut grid_world_dynamics = sim.world.get_component_mut::<WorldDynamics>(self.grid).unwrap();
      if input.debug.grid_linear_velocity_x_inc {
        grid_world_dynamics.linear_velocity.x += 0.001;
      }
      if input.debug.grid_linear_velocity_x_dec {
        grid_world_dynamics.linear_velocity.x -= 0.001;
      }
      if input.debug.grid_linear_velocity_y_inc {
        grid_world_dynamics.linear_velocity.y += 0.001;
      }
      if input.debug.grid_linear_velocity_y_dec {
        grid_world_dynamics.linear_velocity.y -= 0.001;
      }
      if input.debug.grid_angular_velocity_inc {
        grid_world_dynamics.angular_velocity = grid_world_dynamics.angular_velocity * Rotor2::from_angle(0.01);
      }
      if input.debug.grid_angular_velocity_dec {
        grid_world_dynamics.angular_velocity = grid_world_dynamics.angular_velocity * Rotor2::from_angle(-0.01);
      }
    }
    if input.debug.grid_randomize {
      self.clear_grid_tiles(sim);
    }
    if input.debug.grid_reset {
      {
        let mut grid_world_dynamics = sim.world.get_component_mut::<WorldDynamics>(self.grid).unwrap();
        grid_world_dynamics.linear_velocity = Vec2::zero();
        grid_world_dynamics.angular_velocity = Rotor2::identity();
      }
      {
        let mut grid_world_transform = sim.world.get_component_mut::<WorldTransform>(self.grid).unwrap();
        grid_world_transform.isometry = Isometry2::identity();
      }
      self.clear_grid_tiles(sim);
    }
  }

  fn clear_grid_tiles(&mut self, sim: &mut Sim) {
    let mut command_buffer = legion::command::CommandBuffer::new(&sim.world);
    let in_grid = InGrid::new(self.grid);
    let query = Read::<GridPosition>::query().filter(tag_value::<InGrid>(&in_grid));
    for (entity, _) in query.iter_entities(&sim.world) {
      dbg!(entity);
      command_buffer.delete(entity);
    }
    command_buffer.write(&mut sim.world);
  }
}
