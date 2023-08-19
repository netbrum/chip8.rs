pub mod display;
mod keyboard;

use display::Display;
use keyboard::Keyboard;
use rand::Rng;

const STACK_SIZE: usize = 16;
const REGISTER_SIZE: usize = 16;

const MEMORY_SIZE: usize = 4096;
const START_ADDRESS: usize = 512;

const FONT_SIZE: usize = 16 * 5;

const FONTSET: [u8; FONT_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

pub struct Emulator {
    memory: [u8; MEMORY_SIZE],
    v_registers: [u8; REGISTER_SIZE],
    program_counter: usize,
    stack_pointer: usize,
    stack: [usize; STACK_SIZE],
    index_register: usize,
    sound_timer: u8,
    delay_timer: u8,
    pub display: Display,
    pub keyboard: Keyboard,
}

impl Emulator {
    pub fn new() -> Emulator {
        let mut emulator = Emulator {
            memory: [0; MEMORY_SIZE],
            v_registers: [0; REGISTER_SIZE],
            program_counter: START_ADDRESS,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            index_register: 0,
            sound_timer: 0,
            delay_timer: 0,
            display: Display::new(),
            keyboard: Keyboard::new(),
        };

        emulator.memory[..FONT_SIZE].copy_from_slice(&FONTSET);

        emulator
    }

    pub fn load_rom(&mut self, buffer: &[u8]) {
        self.memory[FONT_SIZE..(FONT_SIZE + buffer.len())].copy_from_slice(buffer);
    }

    fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn execute(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12,
            (opcode & 0xF00) >> 8,
            (opcode & 0xF0) >> 4,
            (opcode & 0xF),
        );

        let addr = (opcode & 0xFFF) as usize;

        let x = nibbles.1 as usize;
        let y = nibbles.3 as usize;

        let byte = (opcode & 0xFF) as u8;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.display.clear(),
            (0x0, 0x0, 0xE, 0xE) => {
                self.program_counter = self.stack[self.stack_pointer];
                self.stack_pointer -= 1;
            }
            (0x1, _, _, _) => {
                self.program_counter = addr;
            }
            (0x2, _, _, _) => {
                self.stack_pointer += 1;

                self.stack[self.stack_pointer] = self.program_counter;

                self.program_counter = addr;
            }
            (0x3, _, _, _) => {
                if self.v_registers[x] == byte {
                    self.program_counter += 2;
                }
            }
            (0x4, _, _, _) => {
                if self.v_registers[x] != byte {
                    self.program_counter += 2;
                }
            }
            (0x5, _, _, _) => {
                if self.v_registers[x] == self.v_registers[y] {
                    self.program_counter += 2;
                }
            }
            (0x6, _, _, _) => {
                self.v_registers[x] = byte;
            }
            (0x7, _, _, _) => {
                self.v_registers[x] += byte;
            }
            (0x8, _, _, 0x0) => {
                self.v_registers[x] = self.v_registers[y];
            }
            (0x8, _, _, 0x1) => {
                self.v_registers[x] |= self.v_registers[y];
            }
            (0x8, _, _, 0x2) => {
                self.v_registers[x] &= self.v_registers[y];
            }
            (0x8, _, _, 0x3) => {
                self.v_registers[x] ^= self.v_registers[y];
            }
            (0x8, _, _, 0x4) => {
                let vx = self.v_registers[x] as u16;
                let vy = self.v_registers[y] as u16;
                let result = vx + vy;

                self.v_registers[x] = result as u8;
                self.v_registers[0xF] = if result > 0xFF { 1 } else { 0 };
            }
            (0x8, _, _, 0x5) => {
                self.v_registers[0xF] = if self.v_registers[x] > self.v_registers[y] {
                    1
                } else {
                    0
                };

                self.v_registers[x] = self.v_registers[x].wrapping_sub(self.v_registers[y]);
            }
            (0x8, _, _, 0x6) => {
                self.v_registers[0xF] = self.v_registers[x] & 1;
                self.v_registers[x] >>= 1;
            }
            (0x8, _, _, 0x7) => {
                self.v_registers[0xF] = if self.v_registers[y] > self.v_registers[x] {
                    1
                } else {
                    0
                };

                self.v_registers[x] = self.v_registers[y].wrapping_sub(self.v_registers[x]);
            }
            (0x8, _, _, 0xE) => {
                self.v_registers[0xF] = self.v_registers[x] >> 7;
                self.v_registers[x] <<= 1;
            }
            (0x9, _, _, 0x0) => {
                if self.v_registers[x] != self.v_registers[y] {
                    self.program_counter += 2;
                }
            }
            (0xA, _, _, _) => {
                self.index_register = addr;
            }
            (0xB, _, _, _) => {
                self.program_counter = addr.wrapping_add(self.v_registers[0].into());
            }
            (0xC, _, _, _) => {
                self.v_registers[x] = rand::thread_rng().gen_range(0..=255) & byte;
            }
            (0xD, _, _, n) => {
                let sprite = &self.memory[self.index_register..(self.index_register + n as usize)];

                let x = self.v_registers[x] as usize;
                let y = self.v_registers[y] as usize;

                self.v_registers[0xF] = self.display.draw(display::Point { x, y }, sprite) as u8;
            }
            (0xE, _, 0x9, 0xE) => {
                let key = self.v_registers[x];
                if self.keyboard.keys[key as usize] {
                    self.program_counter += 2;
                }
            }
            (0xE, _, 0xA, 0x1) => {
                let key = self.v_registers[x];
                if !self.keyboard.keys[key as usize] {
                    self.program_counter += 2;
                }
            }
            (0xF, _, 0x0, 0x7) => {
                self.v_registers[x] = self.delay_timer;
            }
            (0xF, _, 0x0, 0xA) => {
                let mut pressed = false;

                for index in 0..self.keyboard.keys.len() {
                    if self.keyboard.keys[index] {
                        self.v_registers[x] = index as u8;
                        pressed = true;
                        break;
                    }
                }

                // redo opcode if a key wasn't pressed
                if !pressed {
                    self.program_counter -= 2;
                }
            }
            (0xF, _, 0x1, 0x5) => {
                self.delay_timer = self.v_registers[x];
            }
            (0xF, _, 0x1, 0x8) => {
                self.sound_timer = self.v_registers[x];
            }
            (0xF, _, 0x1, 0xE) => {
                self.index_register += self.v_registers[x] as usize;
            }
            (0xF, _, 0x2, 0x9) => {
                self.index_register = (self.v_registers[x] as usize) * 5;
            }
            (0xF, _, 0x3, 0x3) => {
                let value = self.v_registers[x] as f32;
                self.memory[self.index_register] = (value / 100.0 % 10.0).floor() as u8;
                self.memory[self.index_register + 1] = (value / 10.0 % 10.0).floor() as u8;
                self.memory[self.index_register + 2] = (value % 10.0) as u8
            }
            (0xF, _, 0x5, 0x5) => {
                let x = self.v_registers[x] as usize;

                for (index, v) in self.v_registers[..=x].iter().enumerate() {
                    self.memory[self.index_register + index] = *v;
                }
            }
            (0xF, _, 0x6, 0x5) => {
                let x = self.v_registers[x] as usize;

                for (index, v) in self.memory[self.index_register..=x].iter().enumerate() {
                    self.v_registers[index] = *v;
                }
            }
            _ => unimplemented!("opcode {:b}", opcode),
        }
    }
}
