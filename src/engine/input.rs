use std::collections::HashSet;
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct InputState {
    pressed: HashSet<KeyCode>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
        }
    }

    pub fn update(&mut self, key_event: &KeyEvent) {
        if let PhysicalKey::Code(code) = key_event.physical_key {
            match key_event.state {
                ElementState::Pressed => {
                    self.pressed.insert(code);
                }
                ElementState::Released => {
                    self.pressed.remove(&code);
                }
            }
        }
    }

    pub fn is_pressed(&self, code: KeyCode) -> bool {
        self.pressed.contains(&code)
    }

    pub fn clear(&mut self) {
        self.pressed.clear();
    }
}
