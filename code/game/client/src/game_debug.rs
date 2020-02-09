use legion::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;
use rand_pcg::Pcg64Mcg;
use ultraviolet::{Isometry2, Rotor2, Vec2, Vec3};

use gfx::Gfx;
use gfx::grid_renderer::GridTileRender;
use sim::prelude::*;

use crate::game::Game;
use crate::game_def::GameDef;
use crate::metrics::Metrics;

pub struct GameDebug {
  grid: Entity,
}

impl GameDebug {
  pub fn new(game_def: &GameDef, sim: &mut Sim, _gfx: &mut Gfx, _game: &mut Game) -> Self {
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

    GameDebug { grid }
  }
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

  pub activate_setup_1: bool,
  pub activate_setup_2: bool,
  pub activate_setup_3: bool,
  pub activate_setup_4: bool,
  pub activate_setup_5: bool,
  pub activate_setup_6: bool,
  pub activate_setup_7: bool,
  pub activate_setup_8: bool,
  pub activate_setup_9: bool,
  pub activate_setup_0: bool,

  pub print_metrics: bool,
}

impl GameDebug {
  pub fn update_before_tick(
    &mut self,
    input: &GameDebugInput,
    game_def: &GameDef,
    sim: &mut Sim,
    gfx: &mut Gfx,
    _game: &mut Game,
    metrics: &mut Metrics,
  ) {
    if input.grid_randomize {
      self.clear_grid_tiles(sim);
      let mut rng = rand::thread_rng();
      let lower_bound = rng.gen_range(-100, 0);
      let upper_bound = rng.gen_range(0, 100);
      self.randomize_grid_tiles(lower_bound, upper_bound, game_def, sim);
    }

    if input.grid_reset {
      if let Some(mut grid_world_dynamics) = sim.world.get_component_mut::<WorldDynamics>(self.grid) {
        grid_world_dynamics.linear_velocity = Vec2::zero();
        grid_world_dynamics.angular_velocity = Rotor2::identity();
      }
      if let Some(mut grid_world_transform) = sim.world.get_component_mut::<WorldTransform>(self.grid) {
        grid_world_transform.isometry = Isometry2::identity();
      }
    }

    if input.activate_setup_1 {
      gfx.camera_sys.set_position(Vec3::new(-0.5, -0.5, 1.0));
      gfx.camera_sys.set_zoom(16.0*7.0);
      self.clear_grid_tiles(sim);
      self.randomize_grid_tiles(16*-1, 16*6, game_def, sim);
    }

    if input.print_metrics {
      metrics.print_metrics();
    }
  }

  pub fn tick_before_sim(
    &mut self,
    input: &GameDebugInput,
    _game_def: &GameDef,
    sim: &mut Sim,
    _gfx: &mut Gfx,
    _game: &mut Game,
  ) {
    let mut grid_world_dynamics = sim.world.get_component_mut::<WorldDynamics>(self.grid).unwrap();
    if input.grid_linear_velocity_x_inc {
      grid_world_dynamics.linear_velocity.x += 0.001;
    }
    if input.grid_linear_velocity_x_dec {
      grid_world_dynamics.linear_velocity.x -= 0.001;
    }
    if input.grid_linear_velocity_y_inc {
      grid_world_dynamics.linear_velocity.y += 0.001;
    }
    if input.grid_linear_velocity_y_dec {
      grid_world_dynamics.linear_velocity.y -= 0.001;
    }
    if input.grid_angular_velocity_inc {
      grid_world_dynamics.angular_velocity = grid_world_dynamics.angular_velocity * Rotor2::from_angle(0.01);
    }
    if input.grid_angular_velocity_dec {
      grid_world_dynamics.angular_velocity = grid_world_dynamics.angular_velocity * Rotor2::from_angle(-0.01);
    }
  }
}

impl GameDebug {
  fn clear_grid_tiles(&mut self, sim: &mut Sim) {
    let mut command_buffer = legion::command::CommandBuffer::new(&sim.world);
    let in_grid = InGrid::new(self.grid);
    let query = Read::<GridPosition>::query().filter(tag_value::<InGrid>(&in_grid));
    for (entity, _) in query.iter_entities(&sim.world) {
      command_buffer.delete(entity);
    }
    command_buffer.write(&mut sim.world);
  }

  fn randomize_grid_tiles(&mut self, lower_bound: i32, upper_bound: i32, game_def: &GameDef, sim: &mut Sim) {
    let mut rng = Pcg64Mcg::new(0xcafef00dd15ea5e5);
    let mut command_buffer = legion::command::CommandBuffer::new(&sim.world);
    for y in lower_bound..upper_bound {
      for x in lower_bound..upper_bound {
        if let Some(texture_idx) = game_def.grid_tile_textures.choose(&mut rng) {
          command_buffer.insert((InGrid::new(self.grid), ), vec![
            (GridPosition::new(x, y), GridOrientation::default(), GridTileRender(*texture_idx)),
          ]);
        }
      }
    }
    command_buffer.write(&mut sim.world);
  }
}
