use legion::borrow::{Ref, RefMut};
use legion::prelude::*;

use util::timing::Duration;

use crate::{GridCoords, GridDynamics};


pub struct Sim {
  pub world: World,
}

impl Sim {
  pub fn new() -> Self {
    let world = World::default();
    Self { world }
  }

  pub fn simulate(&mut self, _time_step: Duration) {
    let dynamics_query = <(Write<GridCoords>, Read<GridDynamics>)>::query();
    for i in dynamics_query.iter(&mut self.world) {
      let (mut coords, dynamics): (RefMut<GridCoords>, Ref<GridDynamics>) = i;
      coords.position += dynamics.linear_velocity;
      coords.rotation += dynamics.angular_velocity;
    }
  }
}
