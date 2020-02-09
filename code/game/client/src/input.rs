use winit::event::VirtualKeyCode;

use gfx::camera::CameraInput;
use os::input_sys::RawInput;

use crate::game_debug::GameDebugInput;

#[derive(Default, Copy, Clone, Debug)]
pub struct Input {
  pub game_debug: GameDebugInput,
  pub camera: CameraInput,
}

impl Input {
  pub fn from_raw(input: RawInput) -> Self {
    let game_debug = GameDebugInput {
      grid_linear_velocity_x_inc: input.is_key_down(VirtualKeyCode::PageDown),
      grid_linear_velocity_x_dec: input.is_key_down(VirtualKeyCode::Delete),
      grid_linear_velocity_y_inc: input.is_key_down(VirtualKeyCode::Home),
      grid_linear_velocity_y_dec: input.is_key_down(VirtualKeyCode::End),
      grid_angular_velocity_inc: input.is_key_down(VirtualKeyCode::PageUp),
      grid_angular_velocity_dec: input.is_key_down(VirtualKeyCode::Insert),
      grid_randomize: input.is_key_pressed(VirtualKeyCode::R),
      grid_reset: input.is_key_pressed(VirtualKeyCode::Return),

      activate_setup_1: input.is_key_pressed(VirtualKeyCode::Key1),
      activate_setup_2: input.is_key_pressed(VirtualKeyCode::Key2),
      activate_setup_3: input.is_key_pressed(VirtualKeyCode::Key3),
      activate_setup_4: input.is_key_pressed(VirtualKeyCode::Key4),
      activate_setup_5: input.is_key_pressed(VirtualKeyCode::Key5),
      activate_setup_6: input.is_key_pressed(VirtualKeyCode::Key6),
      activate_setup_7: input.is_key_pressed(VirtualKeyCode::Key7),
      activate_setup_8: input.is_key_pressed(VirtualKeyCode::Key8),
      activate_setup_9: input.is_key_pressed(VirtualKeyCode::Key9),
      activate_setup_0: input.is_key_pressed(VirtualKeyCode::Key0),

      print_metrics: input.is_key_pressed(VirtualKeyCode::M)
    };
    let camera = CameraInput {
      move_up: input.is_key_down(VirtualKeyCode::W),
      move_right: input.is_key_down(VirtualKeyCode::D),
      move_down: input.is_key_down(VirtualKeyCode::S),
      move_left: input.is_key_down(VirtualKeyCode::A),
      zoom_delta: input.mouse_wheel_delta.y as f32,
      drag: input.mouse_buttons.right,
      drag_pos: input.mouse_pos,
    };
    Input { game_debug, camera }
  }
}
