use ultraviolet::{Mat4, Vec2, Vec3};
use ultraviolet::projection;

use math::screen::{PhysicalPosition, PhysicalSize};
use std::time::Duration;

#[derive(Debug)]
pub struct CameraSys {
  position: Vec3,
  zoom: f32,
  pan_speed: f32,
  mag_speed: f32,
  view_proj: Mat4,
  view_proj_inverse: Mat4,
  viewport: PhysicalSize,
  last_mouse_pos: Option<Vec2>,
}

impl CameraSys {
  pub fn new(viewport: PhysicalSize) -> CameraSys {
    CameraSys::with_speeds(viewport, 50.0, 0.05)
  }

  pub fn with_speeds(viewport: PhysicalSize, pan_speed: f32, mag_speed: f32) -> CameraSys {
    CameraSys {
      // TODO: why is z 1.0? Shouldn't Z be -1.0, since 1.0 z is going INTO the screen? Is it because the view transformation is applied BEFORE the projection transformation, which flips the Z around?
      position: Vec3::new(0.0, 0.0, 1.0),
      zoom: 1.0,
      pan_speed,
      mag_speed,
      view_proj: Mat4::identity(),
      view_proj_inverse: Mat4::identity().inversed(),
      viewport,
      last_mouse_pos: None
    }
  }

  #[inline]
  pub fn position(&self) -> Vec3 { self.position }

  #[inline]
  pub fn zoom(&self) -> f32 { self.zoom }

  #[inline]
  pub fn set_position(&mut self, position: Vec3) { self.position = position; }

  #[inline]
  pub fn set_zoom(&mut self, zoom: f32) { self.zoom = zoom; }

  #[inline]
  pub fn view_projection_matrix(&self) -> Mat4 { self.view_proj }

  /// Converts screen coordinates (in pixels, relative to the top-left of the screen) to view coordinates (in meters,
  /// relative to the center of the screen).
  #[inline]
  pub fn screen_to_view(&self, x: f32, y:f32) -> Vec3 {
    let (width, height): (f32, f32) = self.viewport.into();
    let x = 2.0 * x / width - 1.0;
    let y = 2.0 * y / height - 1.0;
    let vec = Vec3::new(x, y, 0.0);
    Vec3::from_homogeneous_point(self.view_proj_inverse * vec.into_homogeneous_point())
  }

  /// Converts screen coordinates (in pixels, relative to the top-left of the screen) to world coordinates (in meters,
  /// absolute).
  #[inline]
  pub fn screen_to_world(&self, x: f32, y:f32) -> Vec3 {
    self.position + self.screen_to_view(x, y)
  }


  pub fn panning_speed(&self) -> f32 { self.pan_speed }

  pub fn set_panning_speed(&mut self, pan_speed: f32) { self.pan_speed = pan_speed; }

  pub fn magnification_speed(&self) -> f32 { self.mag_speed }

  pub fn set_magnification_speed(&mut self, mag_speed: f32) { self.mag_speed = mag_speed; }


  pub(crate) fn signal_viewport_resize(&mut self, viewport: PhysicalSize) {
    self.viewport = viewport;
  }

  pub(crate) fn update(
    &mut self,
    input: CameraInput,
    frame_time: Duration,
  ) {
    let pan_speed = self.pan_speed * frame_time.as_secs_f32();
    let mag_speed = self.mag_speed;
    if input.move_up { self.position.y += pan_speed };
    if input.move_right { self.position.x += pan_speed };
    if input.move_down { self.position.y -= pan_speed };
    if input.move_left { self.position.x -= pan_speed };
    self.zoom *= 1.0 - input.zoom_delta * mag_speed;

    let (width, height): (f32, f32) = self.viewport.into();

    // TODO: fix mouse dragging.
    if input.drag {
      let mouse_pos = Vec2::new(input.drag_pos.x as f32, input.drag_pos.y as f32);
      if self.last_mouse_pos.is_none() {
        self.last_mouse_pos = Some(mouse_pos);
      }
      let mouse_delta = Vec2::new(width / 2.0, height / 2.0) + (mouse_pos - self.last_mouse_pos.unwrap());
      self.position -= self.screen_to_view(mouse_delta.x, mouse_delta.y);
      self.last_mouse_pos = Some(mouse_pos);
    } else {
      self.last_mouse_pos = None;
    }

    // View matrix.
    let view = Mat4::look_at_lh(
      Vec3::new(self.position.x, self.position.y, self.position.z),
      Vec3::new(self.position.x, self.position.y, 0.0),
      Vec3::unit_y()
    );

    // Orthographic (zoomable) projection matrix.
    let proj = {
      let aspect_ratio = width / height;
      let min_x = aspect_ratio * self.zoom / -2.0;
      let max_x = aspect_ratio * self.zoom / 2.0;
      let min_y = self.zoom / -2.0;
      let max_y = self.zoom / 2.0;
      let min_z = 0.01f32;
      let max_z = 1000.0f32;
      projection::lh_yup::orthographic_vk(min_x, max_x,
        min_y, max_y,
        min_z, max_z
      )
    };

    let view_proj = proj * view;
    self.view_proj = view_proj;
    self.view_proj_inverse = view_proj.inversed();
  }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct CameraInput {
  // Keyboard movement.
  pub move_up: bool,
  pub move_right: bool,
  pub move_down: bool,
  pub move_left: bool,
  // Mouse scroll zoom.
  pub zoom_delta: f32,
  // Mouse dragging.
  pub drag: bool,
  pub drag_pos: PhysicalPosition,
}
