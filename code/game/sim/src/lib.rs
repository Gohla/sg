use legion::entity::Entity;
use ultraviolet::{Rotor2, Vec2};

pub mod legion_sim;

// Grid components.

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct GridCoords {
  pub position: Vec2,
  pub rotation: Rotor2
}

impl GridCoords {
  pub fn new(x: f32, y: f32, a: f32) -> Self { Self { position: Vec2::new(x, y), rotation: Rotor2::from_angle(a) } }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct GridDynamics {
  pub linear_velocity: Vec2,
  pub angular_velocity: Rotor2,
}

impl GridDynamics {
  pub fn new(x: f32, y: f32, a: f32) -> Self { Self { linear_velocity: Vec2::new(x, y), angular_velocity: Rotor2::from_angle(a) } }
}

// Inside-of grid components.

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct InGrid(pub Entity);

#[repr(C)]
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct InGridPosition { pub x: i32, pub y: i32 }

impl InGridPosition {
  pub fn new(x: i32, y: i32) -> Self { Self { x, y } }
}

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum InGridRotation {
  Up,
  Right,
  Down,
  Left
}
