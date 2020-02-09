use ultraviolet::Vec3;

use gfx::Gfx;
use sim::legion_sim::Sim;

use crate::game_def::GameDef;

pub struct Game {}

impl Game {
  pub fn new(_game_def: &GameDef, _sim: &mut Sim, gfx: &mut Gfx) -> Self {
    gfx.camera_sys.set_position(Vec3::new(-0.5, -0.5, 1.0));
    gfx.camera_sys.set_zoom(33.0);
    Self {}
  }
}
