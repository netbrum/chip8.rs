use sdl2::{keyboard::Keycode, EventPump, Sdl};

pub struct Keyboard {
    pub keys: [bool; 16],
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard { keys: [false; 16] }
    }

    fn key_to_hex(key: &Keycode) -> Option<u8> {
        return match key {
            Keycode::Num1 => Some(0x0),
            Keycode::Num2 => Some(0x1),
            Keycode::Num3 => Some(0x2),
            Keycode::Num4 => Some(0x3),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0x7),
            Keycode::A => Some(0x8),
            Keycode::S => Some(0x9),
            Keycode::D => Some(0xA),
            Keycode::F => Some(0xB),
            Keycode::Z => Some(0xC),
            Keycode::X => Some(0xD),
            Keycode::C => Some(0xE),
            Keycode::V => Some(0xF),
            _ => None,
        };
    }

    pub fn poll(&mut self, event_pump: &EventPump) {
        let keys: Vec<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let mut emulator_keys = [false; 16];

        for key in keys {
            if let Some(hex) = Keyboard::key_to_hex(&key) {
                emulator_keys[hex as usize] = true;
            }
        }

        self.keys = emulator_keys;
    }
}
