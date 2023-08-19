pub mod display;
mod keyboard;

use display::Display;
use keyboard::Keyboard;
use rand::Rng;

const STACK_SIZE: usize = 16;
const REGISTER_SIZE: usize = 16;

const MEMORY_SIZE: usize = 4096;
const START_ADDRESS: usize = 512;

const FONTSET_SIZE: usize = 16 * 5;

const FONTSET: [u8; FONTSET_SIZE] = [
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

        emulator.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        emulator
    }

    pub fn load_rom(&mut self, buffer: &[u8]) {
        self.memory[START_ADDRESS..(START_ADDRESS + buffer.len())].copy_from_slice(buffer);
    }
    pub fn tick(&mut self) {
        let opcode = self.fetch();
        self.execute(opcode);
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // beep
            }
            self.sound_timer -= 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.memory[self.program_counter] as u16;
        let lower_byte = self.memory[self.program_counter + 1] as u16;

        let opcode = (higher_byte << 8) | lower_byte;
        self.program_counter += 2;

        opcode
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
        let y = nibbles.2 as usize;

        let byte = (opcode & 0xFF) as u8;

        match nibbles {
            // 00E0 - CLS
            (0x0, 0x0, 0xE, 0x0) => self.display.clear(),
            // 00EE - RET
            (0x0, 0x0, 0xE, 0xE) => {
                self.program_counter = self.stack[self.stack_pointer];
                self.stack_pointer -= 1;
            }
            // 1nnn - JP addr
            (0x1, _, _, _) => {
                self.program_counter = addr;
            }
            // 2nnn - CALL addr
            (0x2, _, _, _) => {
                self.stack_pointer += 1;

                self.stack[self.stack_pointer] = self.program_counter;

                self.program_counter = addr;
            }
            // 3xkk - SE Vx, byte
            (0x3, _, _, _) => {
                if self.v_registers[x] == byte {
                    self.program_counter += 2;
                }
            }
            // 4xkk - SNE Vx, byte
            (0x4, _, _, _) => {
                if self.v_registers[x] != byte {
                    self.program_counter += 2;
                }
            }
            // 5xy0 - SE Vx, Vy
            (0x5, _, _, 0x0) => {
                if self.v_registers[x] == self.v_registers[y] {
                    self.program_counter += 2;
                }
            }
            // 6xkk - LD Vx, byte
            (0x6, _, _, _) => {
                self.v_registers[x] = byte;
            }
            // 7xkk - ADD Vx, byte
            (0x7, _, _, _) => {
                self.v_registers[x] = self.v_registers[x].wrapping_add(byte);
            }
            // 8xy0 - LD Vx, Vy
            (0x8, _, _, 0x0) => {
                self.v_registers[x] = self.v_registers[y];
            }
            // 8xy1 - OR Vx, Vy
            (0x8, _, _, 0x1) => {
                self.v_registers[x] |= self.v_registers[y];
            }
            // 8xy2 - AND Vx, Vy
            (0x8, _, _, 0x2) => {
                self.v_registers[x] &= self.v_registers[y];
            }
            // 8xy3 - XOR Vx, Vy
            (0x8, _, _, 0x3) => {
                self.v_registers[x] ^= self.v_registers[y];
            }
            // 8xy4 - ADD Vx, Vy
            (0x8, _, _, 0x4) => {
                let vx = self.v_registers[x] as u16;
                let vy = self.v_registers[y] as u16;
                let result = vx + vy;

                self.v_registers[x] = result as u8;
                self.v_registers[0xF] = if result > 0xFF { 1 } else { 0 };
            }
            // 8xy5 - SUB Vx, Vy
            (0x8, _, _, 0x5) => {
                self.v_registers[0xF] = if self.v_registers[x] > self.v_registers[y] {
                    1
                } else {
                    0
                };

                self.v_registers[x] = self.v_registers[x].wrapping_sub(self.v_registers[y]);
            }
            // 8xy6 - SHR Vx {, Vy}
            (0x8, _, _, 0x6) => {
                self.v_registers[0xF] = self.v_registers[x] & 1;
                self.v_registers[x] >>= 1;
            }
            // 8xy7 - SUBN Vx, Vy
            (0x8, _, _, 0x7) => {
                self.v_registers[0xF] = if self.v_registers[y] > self.v_registers[x] {
                    1
                } else {
                    0
                };

                self.v_registers[x] = self.v_registers[y].wrapping_sub(self.v_registers[x]);
            }
            // 8xyE - SHL Vx {, Vy}
            (0x8, _, _, 0xE) => {
                self.v_registers[0xF] = self.v_registers[x] >> 7;
                self.v_registers[x] <<= 1;
            }
            // 9xy0 - SNE Vx, Vy
            (0x9, _, _, 0x0) => {
                if self.v_registers[x] != self.v_registers[y] {
                    self.program_counter += 2;
                }
            }
            // Annn - LD I, addr
            (0xA, _, _, _) => {
                self.index_register = addr;
            }
            // Bnnn - JP V0, addr
            (0xB, _, _, _) => {
                self.program_counter = addr.wrapping_add(self.v_registers[0].into());
            }
            // Cxkk - RND Vx, byte
            (0xC, _, _, _) => {
                self.v_registers[x] = rand::thread_rng().gen_range(0..=255) & byte;
            }
            // Dxyn - DRW Vx, Vy, nibble
            (0xD, _, _, n) => {
                let sprite = &self.memory[self.index_register..(self.index_register + n as usize)];

                let x = self.v_registers[x] as usize;
                let y = self.v_registers[y] as usize;

                self.v_registers[0xF] = self.display.draw(display::Point { x, y }, sprite) as u8;
            }
            // Ex9E - SKP Vx
            (0xE, _, 0x9, 0xE) => {
                let key = self.v_registers[x];
                if self.keyboard.keys[key as usize] {
                    self.program_counter += 2;
                }
            }
            // ExA1 - SKNP Vx
            (0xE, _, 0xA, 0x1) => {
                let key = self.v_registers[x];
                if !self.keyboard.keys[key as usize] {
                    self.program_counter += 2;
                }
            }
            // Fx07 - LD Vx, DT
            (0xF, _, 0x0, 0x7) => {
                self.v_registers[x] = self.delay_timer;
            }
            // Fx0A - LD Vx, K
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
            // Fx15 - LD DT, Vx
            (0xF, _, 0x1, 0x5) => {
                self.delay_timer = self.v_registers[x];
            }
            // Fx18 - LD ST, Vx
            (0xF, _, 0x1, 0x8) => {
                self.sound_timer = self.v_registers[x];
            }
            // Fx1E - ADD I, Vx
            (0xF, _, 0x1, 0xE) => {
                self.index_register += self.v_registers[x] as usize;
            }
            // Fx29 - LD F, Vx
            (0xF, _, 0x2, 0x9) => {
                self.index_register = (self.v_registers[x] as usize) * 5;
            }
            // Fx33 - LD B, Vx
            (0xF, _, 0x3, 0x3) => {
                let value = self.v_registers[x] as f32;
                self.memory[self.index_register] = (value / 100.0 % 10.0).floor() as u8;
                self.memory[self.index_register + 1] = (value / 10.0 % 10.0).floor() as u8;
                self.memory[self.index_register + 2] = (value % 10.0) as u8
            }
            // Fx55 - LD [I], Vx
            (0xF, _, 0x5, 0x5) => {
                for (index, v) in self.v_registers[..=x].iter().enumerate() {
                    self.memory[self.index_register + index] = *v;
                }
            }
            // Fx65 - LD Vx, [I]
            (0xF, _, 0x6, 0x5) => {
                let memory_range = &self.memory[self.index_register..=(x + self.index_register)];

                for (index, v) in memory_range.iter().enumerate() {
                    self.v_registers[index] = *v;
                }
            }
            _ => unimplemented!("opcode {:b} {:?} {}", opcode, nibbles, self.program_counter),
        }
    }
}
