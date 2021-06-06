use crate::cpu::{KeyState, KEY_SIZE};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

enum KeyMapping {
    Literal,
    QWERTY,
}

const MAPPING: KeyMapping = KeyMapping::QWERTY;

pub struct InputDriver {
    events: sdl2::EventPump,
}

impl InputDriver {
    pub fn new(context: &sdl2::Sdl) -> Self {
        Self {
            events: context.event_pump().unwrap(),
        }
    }

    pub fn poll(&mut self) -> Result<[KeyState; KEY_SIZE], ()> {
        for event in self.events.poll_iter() {
            match event {
                Event::Quit { .. } => return Err(()),
                Event::KeyDown {
                    keycode: Some(Keycode::Escape), ..
                } => return Err(()),
                _ => (),
            }
        }

        let mut chip8_keys = [KeyState::NotPressed; KEY_SIZE];

        let keys: Vec<_> = self
            .events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        for key in keys {
            if let Some(idx) = mapping(key) {
                chip8_keys[idx] = KeyState::Pressed;
            }
        }

        Ok(chip8_keys)
    }
}

fn mapping(key: Keycode) -> Option<usize> {
    match MAPPING {
        KeyMapping::Literal => mapping_literal(key),
        KeyMapping::QWERTY => mapping_qwerty(key),
    }
}

fn mapping_literal(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num0 | Keycode::Kp0 => Some(0x0),
        Keycode::Num1 | Keycode::Kp1 => Some(0x1),
        Keycode::Num2 | Keycode::Kp2 => Some(0x2),
        Keycode::Num3 | Keycode::Kp3 => Some(0x3),
        Keycode::Num4 | Keycode::Kp4 => Some(0x4),
        Keycode::Num5 | Keycode::Kp5 => Some(0x5),
        Keycode::Num6 | Keycode::Kp6 => Some(0x6),
        Keycode::Num7 | Keycode::Kp7 => Some(0x7),
        Keycode::Num8 | Keycode::Kp8 => Some(0x8),
        Keycode::Num9 | Keycode::Kp9 => Some(0x9),
        Keycode::A => Some(0xA),
        Keycode::B => Some(0xB),
        Keycode::C => Some(0xC),
        Keycode::D => Some(0xD),
        Keycode::E => Some(0xE),
        Keycode::F => Some(0xF),
        _ => None,
    }
}

fn mapping_qwerty(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
