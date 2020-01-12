use ultraviolet::{Mat4, Vec2, Vec3};

use util::screen_size::LogicalSize;
use util::timing::Duration;

pub struct CameraSys {
  position: Vec2,
  zoom: f32,
  pan_speed: f32,
  mag_speed: f32,
  view_proj: Mat4,
  viewport: LogicalSize,
  last_mouse_pos: Option<Vec2>,
}

impl CameraSys {
  pub fn new(viewport: LogicalSize) -> CameraSys {
    CameraSys::with_speeds(viewport, 50.0, 0.05)
  }

  pub fn with_speeds(viewport: LogicalSize, pan_speed: f32, mag_speed: f32) -> CameraSys {
    CameraSys { position: Vec2::new(0.0, 0.0), zoom: 1.0, pan_speed, mag_speed, view_proj: Mat4::identity(), viewport, last_mouse_pos: None }
  }

  #[inline]
  pub fn position(&self) -> Vec2 { self.position }

  #[inline]
  pub fn zoom(&self) -> f32 { self.zoom }

  #[inline]
  pub fn set_position(&mut self, position: Vec2) { self.position = position; }

  #[inline]
  pub fn set_zoom(&mut self, zoom: f32) { self.zoom = zoom; }

  #[inline]
  pub fn view_projection_matrix(&self) -> Mat4 { self.view_proj }

  /// Converts screen coordinates (in pixels, relative to the top-left of the screen) to view coordinates (in meters,
  /// relative to the center of the screen).
  #[inline]
  pub fn screen_to_view(&self, vector: Vec2) -> Vec3 {
    if let Some(view_proj_inverted) = self.view_proj.inverse_transform() {
      let (width, height): (f32, f32) = self.viewport.into();
      let x = 2.0 * vector.x / width - 1.0;
      let y = 2.0 * vector.y / height - 1.0;
      let vector = Vec3::new(x, y, 0.0);
      view_proj_inverted.transform_vector(vector)
    } else {
      Vec3::new(0.0, 0.0, 0.0)
    }
  }

  /// Converts screen coordinates (in pixels, relative to the top-left of the screen)
  /// to world coordinates (in meters, absolute).
  #[inline]
  pub fn screen_to_world(&self, vector: Vec2) -> Vec3 {
    self.position.to_vec().extend(0.0) + self.screen_to_view(vector)
  }


  pub fn panning_speed(&self) -> f32 { self.pan_speed }

  pub fn set_panning_speed(&mut self, pan_speed: f32) { self.pan_speed = pan_speed; }

  pub fn magnification_speed(&self) -> f32 { self.mag_speed }

  pub fn set_magnification_speed(&mut self, mag_speed: f32) { self.mag_speed = mag_speed; }


  pub(crate) fn signal_viewport_resize(&mut self, viewport: LogicalSize) {
    self.viewport = viewport;
  }

  pub(crate) fn update(
    &mut self,
    input: &CameraInput,
    frame_time: Duration,
  ) {
    let pan_speed = self.pan_speed * frame_time.as_s() as f32;
    let mag_speed = self.mag_speed;
    if input.move_up { self.position.y += pan_speed };
    if input.move_right { self.position.x += pan_speed };
    if input.move_down { self.position.y -= pan_speed };
    if input.move_left { self.position.x -= pan_speed };
    self.zoom *= 1.0 - input.zoom_delta * mag_speed;

    let (width, height): (f32, f32) = self.viewport.into();

    if input.drag {
      // Move the camera
      let mouse_pos = Vec2::new(input.drag_pos.x as f32, input.drag_pos.y as f32);
      if self.last_mouse_pos == None {
        self.last_mouse_pos = Some(mouse_pos);
      }
      let mouse_delta = Vec2::new(width / 2.0, height / 2.0) + (mouse_pos - self.last_mouse_pos.unwrap());
      self.position -= self.screen_to_view(mouse_delta).truncate();
      self.last_mouse_pos = Some(mouse_pos);
    } else {
      self.last_mouse_pos = None;
    }

    let view = {
      let dir = Vec3::unit_z();
      let right = Vec3::unit_y().cross(dir).normalized();
      let up = dir.cross(right);
      let view = Mat4::from([
        right.x, up.x, dir.x, 0.0,
        right.y, up.y, dir.y, 0.0,
        right.z, up.z, dir.z, 0.0,
        0.0, 0.0, 0.0, 1.0,
      ]) * Mat4::from([
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        -self.position.x, -self.position.y, -10.0, 1.0,
      ]);
      view
    };

    // Orthographic (zoomable) projection
    let proj = {
      let aspect_ratio = width / height;
      let min_x = aspect_ratio * self.zoom / -2.0;
      let max_x = aspect_ratio * self.zoom / 2.0;
      let min_y = self.zoom / -2.0;
      let max_y = self.zoom / 2.0;
      let min_z = 0.01f32;
      let max_z = 1000.0f32;
      cgmath::ortho(min_x, max_x,
        min_y, max_y,
        min_z, max_z)
    };

    // Clip matrix to 'fix' Vulkan's co-ordinate space: https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
    let clip = Mat4::from([
      1.0, 0.0, 0.0, 0.0,
      0.0, -1.0, 0.0, 0.0,
      0.0, 0.0, 0.5, 0.0,
      0.0, 0.0, 0.5, 1.0,
    ]);
    let view_proj = clip * proj * view;
    self.view_proj = view_proj;
  }
}

#[derive(Clone, Default, Debug)]
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
  pub drag_pos: MousePos,
}
