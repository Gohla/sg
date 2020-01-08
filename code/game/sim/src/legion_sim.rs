use legion::borrow::{Ref, RefMut};
use legion::prelude::*;

use util::timing::Duration;

use crate::{Grid, GridCoords, GridDynamics, InGridBackgroundRender, InGridPosition};

pub struct Sim {
  world: World
}

impl Sim {
  pub fn new() -> Self {
    let mut world = World::new();

    let grid = Grid(0);
    world.insert((), vec![
      (grid, GridCoords::new(1.0, 1.0, 10.0), GridDynamics::new(0.0, 0.1, 0.0)),
    ]);

    world.insert((grid,), vec![
      (InGridPosition::new(0, 0), InGridBackgroundRender(0)),
      (InGridPosition::new(-1, 0), InGridBackgroundRender(0)),
      (InGridPosition::new(0, 10), InGridBackgroundRender(1)),
    ]);

    Self { world }
  }

  pub fn simulate(&mut self, _time_step: Duration) {
    let dynamics_query = <(Write<GridCoords>, Read<GridDynamics>)>::query();
    for i in dynamics_query.iter(&mut self.world) {
      let (mut coords, dynamics): (RefMut<GridCoords>, Ref<GridDynamics>) = i;
      coords.position += dynamics.linear_velocity;
      coords.rotation += dynamics.angular_velocity;
    }

//    let print_query = <(Read<Grid>, Read<GridCoords>)>::query();
//    for (grid, coords) in print_query.iter_immutable(&self.world) {
//      println!("{:?} {:?}", grid, coords);
//    }
  }
}
