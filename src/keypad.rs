use std::ops::Shl;

use sdl2::{event::Event, keyboard::Keycode, EventPump};

use crate::config::Config;

pub struct Keypad {
    event_queue: EventPump,
    keys: u16,
}

impl Keypad {
    pub fn new<'a>(event_queue: EventPump) -> Keypad {
        Keypad {
            event_queue,
            keys: 0,
        }
    }

    pub fn handle_events(&mut self, config: &mut Config) -> bool {
        while let Some(event) = self.event_queue.poll_event() {
            match event {
                Event::Quit { .. } => {
                    return false;
                }
                Event::KeyDown {
                    keycode: Some(k), ..
                } => {
                    self.key_pressed(k, config);
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
        self.keys.checked_shr((15 - key) as u32).unwrap_or(0) & 0x1 == 1
    }

    fn key_pressed(&mut self, k: Keycode, config: &mut Config) -> Option<u8> {
        let mut matched = true;
        match k {
            Keycode::Up => config.rate += 1,
            Keycode::Down => {
                if config.rate > 1 {
                    config.rate -= 1
                }
            }
            Keycode::Right => config.ipf += 1,
            Keycode::Left => {
                if config.ipf > 1 {
                    config.ipf -= 1
                }
            }
            _ => {
                matched = false;
            }
        }

        if matched {
            println!(
                "Rate: {}Hz. Instructions per frame: {}",
                config.rate, config.ipf
            );
        } else if let Some(c) = self.keycode_to_u8(k) {
            self.keys |= 0x1_u16.shl((15 - c) as u32);
            return Some(c);
        }

        None
    }

    fn key_released(&mut self, k: Keycode) -> Option<u8> {
        let released = self.keycode_to_u8(k);
        if let Some(c) = released {
            self.keys &= 0x0_u16.shl((15 - c) as u32);
        }

        released
    }

    pub fn block_read(&mut self, keep_running: &mut bool, config: &mut Config) -> Option<u8> {
        while let Some(event) = self.event_queue.poll_event() {
            match event {
                Event::Quit { .. } => {
                    *keep_running = false;
                    return None;
                }
                Event::KeyDown {
                    keycode: Some(k), ..
                } => {
                    self.key_pressed(k, config);
                }
                Event::KeyUp {
                    keycode: Some(k), ..
                } => {
                    if let Some(key) = self.key_released(k) {
                        return Some(key);
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn keycode_to_u8(&self, k: Keycode) -> Option<u8> {
        match k {
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
}
