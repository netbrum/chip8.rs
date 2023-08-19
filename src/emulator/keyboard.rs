use sdl2::{keyboard::Keycode, EventPump};

pub struct Keyboard {
    pub keys: [bool; 16],
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard { keys: [false; 16] }
    }

    fn key_to_hex(key: &Keycode) -> Option<u8> {
        return match key {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let keyboard = Keyboard::new();

        assert_eq!(keyboard.keys, [false; 16]);
    }

    #[test]
    fn key_matches() {
        let k = Keyboard::key_to_hex;
        assert_eq!(k(&Keycode::Num1).unwrap(), 0x1);
        assert_eq!(k(&Keycode::Num2).unwrap(), 0x2);
        assert_eq!(k(&Keycode::Num3).unwrap(), 0x3);
        assert_eq!(k(&Keycode::Num4).unwrap(), 0xC);
        assert_eq!(k(&Keycode::Q).unwrap(), 0x4);
        assert_eq!(k(&Keycode::W).unwrap(), 0x5);
        assert_eq!(k(&Keycode::E).unwrap(), 0x6);
        assert_eq!(k(&Keycode::R).unwrap(), 0xD);
        assert_eq!(k(&Keycode::A).unwrap(), 0x7);
        assert_eq!(k(&Keycode::S).unwrap(), 0x8);
        assert_eq!(k(&Keycode::D).unwrap(), 0x9);
        assert_eq!(k(&Keycode::F).unwrap(), 0xE);
        assert_eq!(k(&Keycode::Z).unwrap(), 0xA);
        assert_eq!(k(&Keycode::X).unwrap(), 0x0);
        assert_eq!(k(&Keycode::C).unwrap(), 0xB);
        assert_eq!(k(&Keycode::V).unwrap(), 0xF);
    }

    #[test]
    fn key_doesnt_matchb() {
        let k = Keyboard::key_to_hex;
        assert!(k(&Keycode::P).is_none());
    }
}
