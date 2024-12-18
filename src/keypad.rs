use sdl2::{event::Event, keyboard::Keycode, EventPump};

use crate::display::Display;

const KEY_NUMBERS: usize = 16;

pub struct Keypad<'a> {
    event_queue: &'a mut EventPump,
    keys: [bool; KEY_NUMBERS],
}

impl Keypad<'_> {
    pub fn new<'a>(event_queue: &'a mut EventPump) -> Keypad<'a> {
        Keypad {
            event_queue,
            keys: [false; KEY_NUMBERS],
        }
    }

    pub fn handle_events(&mut self) -> bool {
        while let Some(event) = self.event_queue.poll_event() {
            match event {
                Event::Quit { .. } => {
                    return false;
                }
                Event::KeyDown {
                    keycode: Some(k), ..
                } => {
                    self.key_pressed(k);
                }
                Event::KeyUp {
                    keycode: Some(k), ..
                } => {
                    self.key_released(k);
                }
                _ => {}
            }
        }

        true
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    fn key_pressed(&mut self, k: Keycode) -> Option<u8> {
        let pressed = self.keycode_to_u8(k);
        if let Some(c) = pressed {
            self.keys[c as usize] = true;
        }

        pressed
    }

    fn key_released(&mut self, k: Keycode) {
        let released = self.keycode_to_u8(k);
        if let Some(c) = released {
            self.keys[c as usize] = false;
        }
    }

    pub fn block_read(&mut self, display: &mut Display) -> Option<u8> {
        loop {
            let event = self.event_queue.wait_event();
            match event {
                Event::Quit { .. } => {
                    return None;
                }
                Event::KeyDown {
                    keycode: Some(k), ..
                } => {
                    if let Some(key) = self.key_pressed(k) {
                        return Some(key);
                    };
                }
                Event::Window { .. } => {
                    display.render();
                }
                _ => {}
            }
        }
    }

    fn keycode_to_u8(&self, k: Keycode) -> Option<u8> {
        match k {
            Keycode::Num2 => Some(0x1),
            Keycode::Num3 => Some(0x2),
            Keycode::Num4 => Some(0x3),
            Keycode::Num5 => Some(0xC),
            Keycode::W => Some(0x4),
            Keycode::E => Some(0x5),
            Keycode::R => Some(0x6),
            Keycode::T => Some(0xD),
            Keycode::S => Some(0x7),
            Keycode::D => Some(0x8),
            Keycode::F => Some(0x9),
            Keycode::G => Some(0xE),
            Keycode::X => Some(0xA),
            Keycode::C => Some(0x0),
            Keycode::V => Some(0xB),
            Keycode::B => Some(0xF),
            _ => None,
        }
    }
}
