use legion::entity::Entity;
use ultraviolet::{Isometry2, Rotor2, Vec2};

// World-space components.

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
/// Component indicating the transform of an entity in world-space.
pub struct WorldTransform {
  pub isometry: Isometry2
}

impl WorldTransform {
  #[inline]
  pub fn new(x: f32, y: f32, angle: f32) -> Self { Self { isometry: Isometry2::new(Vec2::new(x, y), Rotor2::from_angle(angle)) } }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
/// Component indicating the dynamics of an entity in world-space.
pub struct WorldDynamics {
  pub linear_velocity: Vec2,
  pub angular_velocity: Rotor2,
}

impl WorldDynamics {
  #[inline]
  pub fn new(x: f32, y: f32, angle: f32) -> Self { Self { linear_velocity: Vec2::new(x, y), angular_velocity: Rotor2::from_angle(angle) } }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
/// Component indicating that an entity is a grid. Typically used as a tag.
pub struct Grid;

// Grid-space components.

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
/// Component indicating that an entity is inside a grid. Typically used as a tag.
pub struct InGrid { pub grid: Entity }

impl InGrid {
  #[inline]
  pub fn new(grid: Entity) -> Self { Self { grid } }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
/// Component indicating the position of an entity in grid-space. Grid of the entity is determined by [InGrid].
pub struct GridPosition {
  pub x: i32,
  pub y: i32,
}

impl GridPosition {
  #[inline]
  pub fn new(x: i32, y: i32) -> Self { Self { x, y } }
}

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
/// Component indicating the orientation of an entity in grid-space. Grid of the entity is determined by [InGrid].
pub enum GridOrientation {
  Up,
  Right,
  Down,
  Left,
}

impl Default for GridOrientation {
  #[inline]
  fn default() -> Self { GridOrientation::Up }
}
