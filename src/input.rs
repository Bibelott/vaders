use winit::{event::ElementState, keyboard::KeyCode};

static mut KEYS: [bool; 256] = [false; 256];

pub fn register_key_state(key: KeyCode, state: ElementState) {
    unsafe {
        KEYS[key as usize] = match state {
            ElementState::Pressed => true,
            ElementState::Released => false,
        };
    }
}

pub fn is_key_pressed(key: KeyCode) -> bool {
    unsafe { KEYS[key as usize] }
}

