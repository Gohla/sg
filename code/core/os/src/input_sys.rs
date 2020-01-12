use std::collections::HashSet;
use std::sync::mpsc::Receiver;

use winit::event::{ElementState as WinitElementState, KeyboardInput, VirtualKeyCode};

use math::screen::{PhysicalDelta, PhysicalPosition};

use crate::event_sys::{ElementState, MouseButton, OsInputEvent};

pub struct OsInputSys {
  input_event_rx: Receiver<OsInputEvent>,
  prev_state: Option<RawInput>,
}

impl OsInputSys {
  pub fn new(input_event_rx: Receiver<OsInputEvent>) -> OsInputSys {
    return OsInputSys {
      input_event_rx,
      prev_state: None,
    };
  }

  pub fn update(&mut self) -> RawInput {
    let mut input_state = if let Some(ref prev_state) = self.prev_state {
      let mut prev_state = prev_state.clone();
      prev_state.clear_deltas();
      prev_state
    } else {
      RawInput::default()
    };

    for event in self.input_event_rx.try_iter() {
      match event {
        OsInputEvent::MouseInput { button, state } => {
          match button {
            MouseButton::Left => input_state.mouse_buttons.left = state == ElementState::Pressed,
            MouseButton::Right => input_state.mouse_buttons.right = state == ElementState::Pressed,
            MouseButton::Middle => input_state.mouse_buttons.middle = state == ElementState::Pressed,
            _ => {}
          };
        }
        OsInputEvent::MouseMoved(position) => {
          input_state.mouse_pos = position;
        }
        OsInputEvent::MouseWheelMoved { x_delta, y_delta } => {
          input_state.mouse_wheel_delta.x += x_delta;
          input_state.mouse_wheel_delta.y += y_delta;
        }
        OsInputEvent::KeyboardInput(KeyboardInput { virtual_keycode, state, .. }) => {
          if let Some(virtual_keycode) = virtual_keycode {
            match state {
              WinitElementState::Pressed => {
                input_state.keyboard_buttons.insert(virtual_keycode);
                input_state.keyboard_buttons_pressed.insert(virtual_keycode);
              }
              WinitElementState::Released => {
                input_state.keyboard_buttons.remove(&virtual_keycode);
                input_state.keyboard_buttons_released.insert(virtual_keycode);
              }
            };
          }
        }
        OsInputEvent::CharacterInput(c) => {
          input_state.characters.push(c);
        }
      }
    }

    input_state.mouse_pos_delta = match self.prev_state {
      Some(ref prev_state) => PhysicalDelta::new(input_state.mouse_pos.x - prev_state.mouse_pos.x, input_state.mouse_pos.y - prev_state.mouse_pos.y),
      None => PhysicalDelta::default(),
    };

    self.prev_state = Some(input_state.clone());
    return input_state;
  }
}


#[derive(Clone, Debug, Default)]
pub struct RawInput {
  pub mouse_buttons: MouseButtons,
  pub mouse_pos: PhysicalPosition,
  pub mouse_pos_delta: PhysicalDelta,
  pub mouse_wheel_delta: MouseWheelDelta,
  pub keyboard_buttons: HashSet<VirtualKeyCode>,
  pub keyboard_buttons_pressed: HashSet<VirtualKeyCode>,
  pub keyboard_buttons_released: HashSet<VirtualKeyCode>,
  pub characters: Vec<char>,
}

impl RawInput {
  pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
    self.keyboard_buttons.contains(&key)
  }
  pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
    self.keyboard_buttons_pressed.contains(&key)
  }
  pub fn is_key_released(&self, key: VirtualKeyCode) -> bool {
    self.keyboard_buttons_released.contains(&key)
  }


  pub fn remove_mouse_input(&mut self) {
    self.mouse_buttons.left = false;
    self.mouse_buttons.right = false;
    self.mouse_buttons.middle = false;
    self.mouse_pos_delta = PhysicalDelta::default();
    self.mouse_wheel_delta = MouseWheelDelta::default();
  }

  pub fn remove_keyboard_input(&mut self) {
    self.keyboard_buttons.clear();
    self.keyboard_buttons_pressed.clear();
    self.keyboard_buttons_released.clear();
    self.characters.clear();
  }


  fn clear_deltas(&mut self) {
    self.mouse_pos_delta = PhysicalDelta::default();
    self.mouse_wheel_delta = MouseWheelDelta::default();
    self.keyboard_buttons_pressed.clear();
    self.keyboard_buttons_released.clear();
    self.characters.clear();
  }
}


#[derive(Clone, Copy, Debug, Default)]
pub struct MouseButtons {
  pub left: bool,
  pub right: bool,
  pub middle: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MouseWheelDelta {
  pub x: f64,
  pub y: f64,
}

impl MouseWheelDelta {
  pub fn new(x: f64, y: f64) -> MouseWheelDelta { MouseWheelDelta { x, y } }
}
