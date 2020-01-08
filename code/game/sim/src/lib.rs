use ultraviolet::{Rotor2, Vec2, Vec2i};

pub mod legion_sim;

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct Grid(u16);

#[derive(Default, Debug)]
pub struct GridCoords {
  position: Vec2,
  rotation: Rotor2
}

impl GridCoords {
  pub fn new(x: f32, y: f32, a: f32) -> Self { Self { position: Vec2::new(x, y), rotation: Rotor2::from_angle(a) } }
}

#[derive(Default, Debug)]
pub struct GridDynamics {
  linear_velocity: Vec2,
  angular_velocity: Rotor2,
}

impl GridDynamics {
  pub fn new(x: f32, y: f32, a: f32) -> Self { Self { linear_velocity: Vec2::new(x, y), angular_velocity: Rotor2::from_angle(a) } }
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct InGrid(u16);

#[derive(Default, Debug)]
pub struct InGridPosition(pub Vec2i);

impl InGridPosition {
  pub fn new(x: i32, y: i32) -> Self { Self(Vec2i::new(x, y)) }
}

#[derive(Debug)]
pub enum InGridRotation {
  Up,
  Right,
  Down,
  Left
}

#[derive(Debug)]
pub struct InGridBackgroundRender(u16);

#[derive(Debug)]
pub struct InGridRender(u16);

#[derive(Debug)]
pub struct InGridForegroundRender(u16);
