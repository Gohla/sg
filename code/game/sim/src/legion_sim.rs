use legion::borrow::{Ref, RefMut};
use legion::prelude::*;

use util::timing::Duration;

use crate::{WorldDynamics, WorldTransform};

pub struct Sim {
  pub world: World,
}

impl Sim {
  pub fn new() -> Self {
    let world = World::default();
    Self { world }
  }

  pub fn simulate(&mut self, _time_step: Duration) {
    let dynamics_query = <(Read<WorldDynamics>, Write<WorldTransform>)>::query();
    for i in dynamics_query.iter_mut(&mut self.world) {
      let (dynamics, mut transform): (Ref<WorldDynamics>, RefMut<WorldTransform>) = i;
      transform.position += dynamics.linear_velocity;
      transform.orientation += dynamics.angular_velocity;
    }
  }
}
