use winit::dpi::{LogicalPosition as WinitLogicalPosition, LogicalSize as WinitLogicalSize, PhysicalPosition as WinitPhysicalPosition, PhysicalSize as WinitPhysicalSize, Pixel};

use math::screen::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};

pub trait LogicalSizeExt {
  fn into_winit(self) -> WinitLogicalSize<f64>;
}

impl LogicalSizeExt for LogicalSize {
  fn into_winit(self) -> WinitLogicalSize<f64> {
    let size: (f64, f64) = self.into();
    WinitLogicalSize::from(size)
  }
}

pub trait PhysicalSizeExt {
  fn into_winit(self) -> WinitPhysicalSize<u32>;
}

impl PhysicalSizeExt for PhysicalSize {
  fn into_winit(self) -> WinitPhysicalSize<u32> {
    let size: (u32, u32) = self.into();
    WinitPhysicalSize::from(size)
  }
}

pub trait WinitLogicalSizeExt {
  fn into_util(self) -> LogicalSize;
}

impl<P: Pixel> WinitLogicalSizeExt for WinitLogicalSize<P> {
  fn into_util(self) -> LogicalSize {
    let size: (f64, f64) = self.into();
    LogicalSize::from(size)
  }
}

pub trait WinitPhysicalSizeExt {
  fn into_util(self) -> PhysicalSize;
}

impl<P: Pixel> WinitPhysicalSizeExt for WinitPhysicalSize<P> {
  fn into_util(self) -> PhysicalSize {
    let size: (u32, u32) = self.into();
    PhysicalSize::from(size)
  }
}


pub trait LogicalPositionExt {
  fn into_winit(self) -> WinitLogicalPosition<f64>;
}

impl LogicalPositionExt for LogicalPosition {
  fn into_winit(self) -> WinitLogicalPosition<f64> {
    let size: (f64, f64) = self.into();
    WinitLogicalPosition::from(size)
  }
}

pub trait PhysicalPositionExt {
  fn into_winit(self) -> WinitPhysicalPosition<i32>;
}

impl PhysicalPositionExt for PhysicalPosition {
  fn into_winit(self) -> WinitPhysicalPosition<i32> {
    let size: (i32, i32) = self.into();
    WinitPhysicalPosition::from(size)
  }
}

pub trait WinitLogicalPositionExt {
  fn into_util(self) -> LogicalPosition;
}

impl<P: Pixel> WinitLogicalPositionExt for WinitLogicalPosition<P> {
  fn into_util(self) -> LogicalPosition {
    let size: (f64, f64) = self.into();
    LogicalPosition::from(size)
  }
}

pub trait WinitPhysicalPositionExt {
  fn into_util(self) -> PhysicalPosition;
}

impl<P: Pixel> WinitPhysicalPositionExt for WinitPhysicalPosition<P> {
  fn into_util(self) -> PhysicalPosition {
    let size: (i32, i32) = self.into();
    PhysicalPosition::from(size)
  }
}
