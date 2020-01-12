#![allow(dead_code)]

use std::ops::{Div, Mul};

//
// Scale (DPI) factor.
//

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct Scale(f64);

impl Scale {
  pub fn new(scale: f64) -> Self {
    debug_assert!(scale.is_sign_positive(), "Scale {} is not positive", scale);
    debug_assert!(scale.is_normal(), "Scale {} is not normal", scale);
    Scale(scale)
  }
}

impl Mul<Scale> for f64 {
  type Output = f64;

  #[inline]
  fn mul(self, rhs: Scale) -> f64 { self * rhs.0 }
}

impl Div<Scale> for f64 {
  type Output = f64;

  #[inline]
  fn div(self, rhs: Scale) -> f64 { self / rhs.0 }
}

impl Mul<Scale> for u32 {
  type Output = f64;

  #[inline]
  fn mul(self, rhs: Scale) -> f64 { self as f64 * rhs.0 }
}

impl Div<Scale> for u32 {
  type Output = f64;

  #[inline]
  fn div(self, rhs: Scale) -> f64 { self as f64 / rhs.0 }
}

impl Mul<Scale> for i32 {
  type Output = f64;

  #[inline]
  fn mul(self, rhs: Scale) -> f64 { self as f64 * rhs.0 }
}

impl Div<Scale> for i32 {
  type Output = f64;

  #[inline]
  fn div(self, rhs: Scale) -> f64 { self as f64 / rhs.0 }
}

impl From<f64> for Scale {
  fn from(scale: f64) -> Self { Scale(scale) }
}

impl From<f32> for Scale {
  fn from(scale: f32) -> Self { Scale(scale as _) }
}

impl From<Scale> for f64 {
  #[inline]
  fn from(scale: Scale) -> Self { scale.0 }
}

impl Default for Scale {
  #[inline]
  fn default() -> Self { Scale(1.0) }
}


//
// Size
//

// Physical size: size in physical (real) pixels on the device.

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct PhysicalSize {
  width: u32,
  height: u32,
}

impl PhysicalSize {
  #[inline]
  pub fn new(width: u32, height: u32) -> Self { Self { width, height } }

  /// Loss of precision in physical size: conversion from f64 into u32.
  #[inline]
  pub fn from_logical<L: Into<LogicalSize>, S: Into<Scale>>(logical: L, scale: S) -> Self { logical.into().into_physical(scale) }

  #[inline]
  pub fn into_logical<S: Into<Scale>>(self, scale: S) -> LogicalSize {
    let scale = scale.into();
    LogicalSize::new(self.width / scale, self.height / scale)
  }

  #[inline]
  pub fn width(&self) -> u32 { self.width }

  #[inline]
  pub fn height(&self) -> u32 { self.height }
}

impl From<(u64, u64)> for PhysicalSize {
  #[inline]
  fn from((width, height): (u64, u64)) -> Self { Self::new(width as _, height as _) }
}

impl From<(u32, u32)> for PhysicalSize {
  #[inline]
  fn from((width, height): (u32, u32)) -> Self { Self::new(width, height) }
}

impl From<PhysicalSize> for (u64, u64) {
  #[inline]
  fn from(physical_size: PhysicalSize) -> Self { (physical_size.width as _, physical_size.height as _) }
}

impl From<PhysicalSize> for (u32, u32) {
  #[inline]
  fn from(physical_size: PhysicalSize) -> Self { (physical_size.width, physical_size.height) }
}


// Logical size: size after scaling. That is, the physical size divided by the scale factor.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct LogicalSize {
  width: f64,
  height: f64,
}

impl LogicalSize {
  #[inline]
  pub fn new(width: f64, height: f64) -> Self {
    debug_assert!(width.is_sign_positive(), "Width {} is not positive", width);
    debug_assert!(width.is_finite(), "Width {} is not finite", width);
    debug_assert!(!width.is_nan(), "Width is NaN");
    debug_assert!(height.is_sign_positive(), "Height {} is not positive", height);
    debug_assert!(height.is_finite(), "Height {} is not finite", height);
    debug_assert!(!height.is_nan(), "Height is NaN");
    Self { width, height }
  }

  #[inline]
  pub fn from_physical<P: Into<PhysicalSize>, S: Into<Scale>>(physical: P, scale: S) -> Self { physical.into().into_logical(scale) }

  /// Loss of precision in physical size: conversion from f64 into u32.
  #[inline]
  pub fn into_physical<S: Into<Scale>>(self, scale: S) -> PhysicalSize {
    let scale = scale.into();
    PhysicalSize::new((self.width * scale).round() as u32, (self.height * scale).round() as u32)
  }

  #[inline]
  pub fn width(&self) -> f64 { self.width }

  #[inline]
  pub fn height(&self) -> f64 { self.height }
}

impl From<(f64, f64)> for LogicalSize {
  #[inline]
  fn from((width, height): (f64, f64)) -> Self { Self::new(width, height) }
}

impl From<(f32, f32)> for LogicalSize {
  #[inline]
  fn from((width, height): (f32, f32)) -> Self { Self::new(width as _, height as _) }
}

impl From<(u64, u64)> for LogicalSize {
  #[inline]
  fn from((width, height): (u64, u64)) -> Self { Self::new(width as _, height as _) }
}

impl From<(u32, u32)> for LogicalSize {
  #[inline]
  fn from((width, height): (u32, u32)) -> Self { Self::new(width as _, height as _) }
}

impl From<LogicalSize> for (f64, f64) {
  #[inline]
  fn from(logical_size: LogicalSize) -> Self { (logical_size.width, logical_size.height) }
}


// Screen size: combination of physical size, scale, and logical size.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct ScreenSize {
  pub physical: PhysicalSize,
  pub scale: Scale,
  pub logical: LogicalSize,
}

impl ScreenSize {
  #[inline]
  pub fn new(physical: PhysicalSize, scale: Scale, logical: LogicalSize) -> Self { Self { physical, scale, logical } }

  /// Loss of precision in physical size: conversion from f64 into u32.
  #[inline]
  pub fn from_logical_scale<L: Into<LogicalSize>, S: Into<Scale>>(logical: L, scale: S) -> Self {
    let logical = logical.into();
    let scale = scale.into();
    let physical = logical.into_physical(scale);
    Self::new(physical, scale, logical)
  }

  #[inline]
  pub fn from_physical_scale<P: Into<PhysicalSize>, S: Into<Scale>>(physical: P, scale: S) -> Self {
    let physical = physical.into();
    let scale = scale.into();
    let logical = physical.into_logical(scale);
    Self::new(physical, scale, logical)
  }

  #[inline]
  pub fn from_unscaled(width: u32, height: u32) -> Self {
    let physical = PhysicalSize::new(width, height);
    let scale = Scale::default();
    let logical = physical.into_logical(scale);
    Self::new(physical, scale, logical)
  }
}

impl From<ScreenSize> for LogicalSize {
  #[inline]
  fn from(screen_size: ScreenSize) -> Self { screen_size.logical }
}

impl From<ScreenSize> for PhysicalSize {
  #[inline]
  fn from(screen_size: ScreenSize) -> Self { screen_size.physical }
}

impl From<ScreenSize> for Scale {
  #[inline]
  fn from(screen_size: ScreenSize) -> Self { screen_size.scale }
}


//
// Position
//

// Position in physical screen space.

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct PhysicalPosition {
  x: i32,
  y: i32,
}

impl PhysicalPosition {
  #[inline]
  pub fn new(x: i32, y: i32) -> Self { Self { x, y } }

  /// Loss of precision in physical position: conversion from f64 into i32.
  #[inline]
  pub fn from_logical<L: Into<LogicalPosition>, S: Into<Scale>>(logical: L, scale: S) -> Self { logical.into().into_physical(scale) }

  #[inline]
  pub fn into_logical<S: Into<Scale>>(self, scale: S) -> LogicalPosition {
    let scale = scale.into();
    LogicalPosition::new(self.x / scale, self.y / scale)
  }

  #[inline]
  pub fn x(&self) -> i32 { self.x }

  #[inline]
  pub fn y(&self) -> i32 { self.y }
}

impl From<(i64, i64)> for PhysicalPosition {
  #[inline]
  fn from((x, y): (i64, i64)) -> Self { Self::new(x as _, y as _) }
}

impl From<(i32, i32)> for PhysicalPosition {
  #[inline]
  fn from((x, y): (i32, i32)) -> Self { Self::new(x, y) }
}

impl From<PhysicalPosition> for (i64, i64) {
  #[inline]
  fn from(physical_position: PhysicalPosition) -> Self { (physical_position.x as _, physical_position.y as _) }
}

impl From<PhysicalPosition> for (i32, i32) {
  #[inline]
  fn from(physical_position: PhysicalPosition) -> Self { (physical_position.x, physical_position.y) }
}


// Position in logical screen space.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct LogicalPosition {
  x: f64,
  y: f64,
}

impl LogicalPosition {
  #[inline]
  pub fn new(x: f64, y: f64) -> Self {
    debug_assert!(x.is_finite(), "X {} is not finite", x);
    debug_assert!(!x.is_nan(), "X {} is NaN", x);
    debug_assert!(y.is_finite(), "Y {} is not finite", y);
    debug_assert!(!y.is_nan(), "Y {} is NaN", y);
    Self { x, y }
  }

  #[inline]
  pub fn from_physical<P: Into<PhysicalPosition>, S: Into<Scale>>(physical: P, scale: S) -> Self { physical.into().into_logical(scale) }

  /// Loss of precision in physical position: conversion from f64 into i32.
  #[inline]
  pub fn into_physical<S: Into<Scale>>(self, scale: S) -> PhysicalPosition {
    let scale = scale.into();
    PhysicalPosition::new((self.x * scale).round() as _, (self.y * scale).round() as _)
  }

  #[inline]
  pub fn x(&self) -> f64 { self.x }

  #[inline]
  pub fn y(&self) -> f64 { self.y }
}

impl From<(f64, f64)> for LogicalPosition {
  #[inline]
  fn from((x, y): (f64, f64)) -> Self { Self::new(x, y) }
}

impl From<(f32, f32)> for LogicalPosition {
  #[inline]
  fn from((x, y): (f32, f32)) -> Self { Self::new(x as _, y as _) }
}

impl From<(i32, i32)> for LogicalPosition {
  #[inline]
  fn from((x, y): (i32, i32)) -> Self { Self::new(x as _, y as _) }
}

impl From<LogicalPosition> for (f64, f64) {
  #[inline]
  fn from(logical_position: LogicalPosition) -> Self { (logical_position.x, logical_position.y) }
}


// Screen position: combination of physical position, scale, and logical position.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct ScreenPosition {
  pub physical: PhysicalPosition,
  pub scale: Scale,
  pub logical: LogicalPosition,
}

impl ScreenPosition {
  #[inline]
  pub fn new(physical: PhysicalPosition, scale: Scale, logical: LogicalPosition) -> Self { Self { physical, scale, logical } }

  /// Loss of precision in physical position: conversion from f64 into i32.
  #[inline]
  pub fn from_logical_scale<L: Into<LogicalPosition>, S: Into<Scale>>(logical: L, scale: S) -> Self {
    let logical = logical.into();
    let scale = scale.into();
    let physical = logical.into_physical(scale);
    Self::new(physical, scale, logical)
  }

  #[inline]
  pub fn from_physical_scale<P: Into<PhysicalPosition>, S: Into<Scale>>(physical: P, scale: S) -> Self {
    let physical = physical.into();
    let scale = scale.into();
    let logical = physical.into_logical(scale);
    Self::new(physical, scale, logical)
  }

  #[inline]
  pub fn from_unscaled(x: i32, y: i32) -> Self {
    let physical = PhysicalPosition::new(x, y);
    let scale = Scale::default();
    let logical = physical.into_logical(scale);
    Self::new(physical, scale, logical)
  }
}

impl From<ScreenPosition> for LogicalPosition {
  #[inline]
  fn from(screen_position: ScreenPosition) -> Self { screen_position.logical }
}

impl From<ScreenPosition> for PhysicalPosition {
  #[inline]
  fn from(screen_position: ScreenPosition) -> Self { screen_position.physical }
}

impl From<ScreenPosition> for Scale {
  #[inline]
  fn from(screen_position: ScreenPosition) -> Self { screen_position.scale }
}


//
// Delta
//

// Delta in physical screen space.

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct PhysicalDelta {
  x: i32,
  y: i32,
}

impl PhysicalDelta {
  #[inline]
  pub fn new(x: i32, y: i32) -> Self { Self { x, y } }

  /// Loss of precision in physical delta: conversion from f64 into i32.
  #[inline]
  pub fn from_logical<L: Into<LogicalDelta>, S: Into<Scale>>(logical: L, scale: S) -> Self { logical.into().into_physical(scale) }

  #[inline]
  pub fn into_logical<S: Into<Scale>>(self, scale: S) -> LogicalDelta {
    let scale = scale.into();
    LogicalDelta::new(self.x / scale, self.y / scale)
  }

  #[inline]
  pub fn x(&self) -> i32 { self.x }

  #[inline]
  pub fn y(&self) -> i32 { self.y }
}

impl From<(i64, i64)> for PhysicalDelta {
  #[inline]
  fn from((x, y): (i64, i64)) -> Self { Self::new(x as _, y as _) }
}

impl From<(i32, i32)> for PhysicalDelta {
  #[inline]
  fn from((x, y): (i32, i32)) -> Self { Self::new(x, y) }
}

impl From<PhysicalDelta> for (i64, i64) {
  #[inline]
  fn from(physical_position: PhysicalDelta) -> Self { (physical_position.x as _, physical_position.y as _) }
}

impl From<PhysicalDelta> for (i32, i32) {
  #[inline]
  fn from(physical_position: PhysicalDelta) -> Self { (physical_position.x, physical_position.y) }
}


// Delta in logical screen space.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct LogicalDelta {
  x: f64,
  y: f64,
}

impl LogicalDelta {
  #[inline]
  pub fn new(x: f64, y: f64) -> Self {
    debug_assert!(x.is_finite(), "X {} is not finite", x);
    debug_assert!(!x.is_nan(), "X {} is NaN", x);
    debug_assert!(y.is_finite(), "Y {} is not finite", y);
    debug_assert!(!y.is_nan(), "Y {} is NaN", y);
    Self { x, y }
  }

  #[inline]
  pub fn from_physical<P: Into<PhysicalDelta>, S: Into<Scale>>(physical: P, scale: S) -> Self { physical.into().into_logical(scale) }

  /// Loss of precision in physical delta: conversion from f64 into i32.
  #[inline]
  pub fn into_physical<S: Into<Scale>>(self, scale: S) -> PhysicalDelta {
    let scale = scale.into();
    PhysicalDelta::new((self.x * scale).round() as _, (self.y * scale).round() as _)
  }

  #[inline]
  pub fn x(&self) -> f64 { self.x }

  #[inline]
  pub fn y(&self) -> f64 { self.y }
}

impl From<(f64, f64)> for LogicalDelta {
  #[inline]
  fn from((x, y): (f64, f64)) -> Self { Self::new(x, y) }
}

impl From<(f32, f32)> for LogicalDelta {
  #[inline]
  fn from((x, y): (f32, f32)) -> Self { Self::new(x as _, y as _) }
}

impl From<(i32, i32)> for LogicalDelta {
  #[inline]
  fn from((x, y): (i32, i32)) -> Self { Self::new(x as _, y as _) }
}

impl From<LogicalDelta> for (f64, f64) {
  #[inline]
  fn from(logical_position: LogicalDelta) -> Self { (logical_position.x, logical_position.y) }
}


// Screen delta: combination of physical delta, scale, and logical delta.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct ScreenDelta {
  pub physical: PhysicalDelta,
  pub scale: Scale,
  pub logical: LogicalDelta,
}

impl ScreenDelta {
  #[inline]
  pub fn new(physical: PhysicalDelta, scale: Scale, logical: LogicalDelta) -> Self { Self { physical, scale, logical } }

  /// Loss of precision in physical position: conversion from f64 into i32.
  #[inline]
  pub fn from_logical_scale<L: Into<LogicalDelta>, S: Into<Scale>>(logical: L, scale: S) -> Self {
    let logical = logical.into();
    let scale = scale.into();
    let physical = logical.into_physical(scale);
    Self::new(physical, scale, logical)
  }

  #[inline]
  pub fn from_physical_scale<P: Into<PhysicalDelta>, S: Into<Scale>>(physical: P, scale: S) -> Self {
    let physical = physical.into();
    let scale = scale.into();
    let logical = physical.into_logical(scale);
    Self::new(physical, scale, logical)
  }

  #[inline]
  pub fn from_unscaled(x: i32, y: i32) -> Self {
    let physical = PhysicalDelta::new(x, y);
    let scale = Scale::default();
    let logical = physical.into_logical(scale);
    Self::new(physical, scale, logical)
  }
}

impl From<ScreenDelta> for LogicalDelta {
  #[inline]
  fn from(screen_position: ScreenDelta) -> Self { screen_position.logical }
}

impl From<ScreenDelta> for PhysicalDelta {
  #[inline]
  fn from(screen_position: ScreenDelta) -> Self { screen_position.physical }
}

impl From<ScreenDelta> for Scale {
  #[inline]
  fn from(screen_position: ScreenDelta) -> Self { screen_position.scale }
}
