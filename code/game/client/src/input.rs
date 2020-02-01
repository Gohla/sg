use winit::event::VirtualKeyCode;

use gfx::camera::CameraInput;
use os::input_sys::RawInput;

use crate::game::{GameDebugInput, GameInput};

#[derive(Default, Copy, Clone, Debug)]
pub struct Input {
  pub game: GameInput,
  pub camera: CameraInput,
}

impl Input {
  pub fn from_raw(input: RawInput) -> Self {
    let game = GameInput {
      debug: GameDebugInput {
        grid_linear_velocity_x_inc: input.is_key_down(VirtualKeyCode::PageDown),
        grid_linear_velocity_x_dec: input.is_key_down(VirtualKeyCode::Delete),
        grid_linear_velocity_y_inc: input.is_key_down(VirtualKeyCode::Home),
        grid_linear_velocity_y_dec: input.is_key_down(VirtualKeyCode::End),
        grid_angular_velocity_inc: input.is_key_down(VirtualKeyCode::PageUp),
        grid_angular_velocity_dec: input.is_key_down(VirtualKeyCode::Insert),
        grid_randomize: input.is_key_down(VirtualKeyCode::R),
        grid_reset: input.is_key_down(VirtualKeyCode::Return),
      },
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
    Input { game, camera }
  }
}
