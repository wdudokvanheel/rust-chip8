use std::{process, u8, usize};

use getrandom::getrandom;

pub struct Chip8 {
    memory: [u8; 4096],
    registers: [u8; 16],
    index_register: u16,
    program_counter: u16,
    stack: Vec<u16>,
    pub display: [[bool; 64]; 32],
    input: [bool; 16],
    delay_timer: u8,
    sound_timer: u8,
    quirk_config: QuirkConfig,
    total_cycles: u32,
    blocking_on_draw: bool,
    blocking_input: Option<u8>,
}

struct Opcode {
    instruction: u16,
    opcode: u8,
    x: u8,
    y: u8,
    n: u8,
    nn: u8,
    nnn: u16,
}

#[derive(Copy, Clone)]
struct QuirkConfig {
    memory_index_register_increase: bool,
    source_vy_bitshift: bool,
}

pub struct Chip8Rom {
    pub name: String,
    pub data: Vec<u8>,
    pub quirks: QuirkConfig,
}

impl Chip8Rom {
    pub fn new(name: &str, data: Vec<u8>) -> Self {
        Chip8Rom {
            name: name.to_string(),
            data,
            quirks: QuirkConfig::new(),
        }
    }

    pub fn to_device(&self) -> Chip8 {
        let mut chip8 = Chip8::new();
        chip8.quirk_config = self.quirks;
        chip8.set_rom(&self.data);
        return chip8;
    }
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            memory: [0; 4096],
            registers: [0; 16],
            index_register: 0,
            program_counter: 0x200,
            stack: vec!(),
            display: [[false; 64]; 32],
            input: [false; 16],
            quirk_config: QuirkConfig::new(),
            total_cycles: 0,
            delay_timer: 0,
            sound_timer: 0,
            blocking_on_draw: false,
            blocking_input: None,
        }
    }

    pub fn cycle(&mut self) {
        if self.blocking_on_draw {
            return;
        }

        self.total_cycles += 1;

        let instruction = self.fetch_instruction();
        let opcode = Opcode::from_instruction(instruction);

        self.program_counter += 2;

        match opcode {
            Opcode { opcode: 0x1, nnn, .. } => self.set_program_counter(nnn),
            Opcode { opcode: 0x2, nnn, .. } => self.jump_sub(nnn),
            Opcode { opcode: 0x3, x, nn, .. } => self.value_conditional_skip(x, nn, false),
            Opcode { opcode: 0x4, x, nn, .. } => self.value_conditional_skip(x, nn, true),
            Opcode { opcode: 0x5, x, y, .. } => self.register_conditional_skip(x, y, false),
            Opcode { opcode: 0x6, x, nn, .. } => self.set_v_register(x, nn),
            Opcode { opcode: 0x7, x, nn, .. } => self.add_v_register(x, nn),
            Opcode { opcode: 0x8, n: 0x0, x, y, .. } => self.register_copy(x, y),
            Opcode { opcode: 0x8, n: 0x1, x, y, .. } => self.register_or(x, y),
            Opcode { opcode: 0x8, n: 0x2, x, y, .. } => self.register_and(x, y),
            Opcode { opcode: 0x8, n: 0x3, x, y, .. } => self.register_xor(x, y),
            Opcode { opcode: 0x8, n: 0x4, x, y, .. } => self.register_add(x, y),
            Opcode { opcode: 0x8, n: 0x5, x, y, .. } => self.register_sub(x, y, false),
            Opcode { opcode: 0x8, n: 0x6, x, y, .. } => self.register_shift(x, y, false),
            Opcode { opcode: 0x8, n: 0x7, x, y, .. } => self.register_sub(x, y, true),
            Opcode { opcode: 0x8, n: 0xE, x, y, .. } => self.register_shift(x, y, true),
            Opcode { opcode: 0x9, x, y, .. } => self.register_conditional_skip(x, y, true),
            Opcode { opcode: 0xA, nnn, .. } => self.set_index_register(nnn),
            Opcode { opcode: 0xB, nnn, .. } => self.jump_offset(nnn),
            Opcode { opcode: 0xC, x, nn, .. } => self.set_register_random(x, nn),
            Opcode { opcode: 0xD, x, y, n, .. } => self.draw_sprite(x, y, n),
            Opcode { opcode: 0xE, nn: 0x9E, x, .. } => self.input_conditional_skip(x, false),
            Opcode { opcode: 0xE, nn: 0xA1, x, .. } => self.input_conditional_skip(x, true),
            Opcode { opcode: 0xF, nn: 0x07, x, .. } => self.get_delay_timer(x),
            Opcode { opcode: 0xF, nn: 0x0A, x, .. } => self.wait_for_input(x),
            Opcode { opcode: 0xF, nn: 0x15, x, .. } => self.set_delay_timer(x),
            Opcode { opcode: 0xF, nn: 0x18, x, .. } => self.set_sound_timer(x),
            Opcode { opcode: 0xF, nn: 0x1E, x, .. } => self.add_index_register(x),
            Opcode { opcode: 0xF, nn: 0x29, x, .. } => self.index_to_font_char(x),
            Opcode { opcode: 0xF, nn: 0x33, x, .. } => self.convert_to_bcd(x),
            Opcode { opcode: 0xF, nn: 0x55, x, .. } => self.register_to_memory(x),
            Opcode { opcode: 0xF, nn: 0x65, x, .. } => self.memory_to_register(x),
            Opcode { instruction: 0x00E0, .. } => self.clear_screen(),
            Opcode { instruction: 0x00EE, .. } => self.return_sub(),
            Opcode { instruction, .. } => {
                println!("Instruction not supported: {:04X}", instruction);
                process::exit(0x0100);
            }
        }
    }

    pub fn update(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        self.blocking_on_draw = false;
    }

    fn set_delay_timer(&mut self, source_register: u8) {
        self.delay_timer = self.registers[source_register as usize];
    }

    fn set_sound_timer(&mut self, source_register: u8) {
        self.sound_timer = self.registers[source_register as usize];
    }

    fn get_delay_timer(&mut self, target_register: u8) {
        self.registers[target_register as usize] = self.delay_timer;
    }

    fn add_index_register(&mut self, target_register: u8) {
        // TODO: Add amiga style VF register handling
        self.index_register = self.index_register.wrapping_add(self.registers[target_register as usize].into());
    }

    fn wait_for_input(&mut self, target_register: u8) {
        match self.blocking_input {
            None => {
                let first_input = self.input.iter().enumerate().find(|&(_, &value)| value);

                match first_input {
                    Some((index, _)) => self.blocking_input = Some(index as u8),
                    None => self.program_counter -= 2,
                }
            }
            Some(input) => {
                if self.input[input as usize] == false {
                    self.registers[target_register as usize] = input;
                    self.blocking_input = None;
                } else {
                    self.program_counter -= 2;
                }
            }
        }
    }

    fn set_register_random(&mut self, target_register: u8, mod_and: u8) {
        let mut buf = [0u8; 1];
        getrandom(&mut buf).expect("Random number");

        self.registers[target_register as usize] = buf[0] & mod_and;
    }

    fn input_conditional_skip(&mut self, source_register: u8, inverse: bool) {
        let input = self.registers[source_register as usize];
        if self.input[input as usize] ^ inverse {
            self.program_counter += 2;
        }
    }

    fn register_to_memory(&mut self, target_register: u8) {
        for i in 0..=target_register {
            self.memory[(self.index_register + i as u16) as usize] = self.registers[i as usize];
        }
        if self.quirk_config.memory_index_register_increase {
            self.index_register += (target_register as u16) + 1;
        }
    }

    fn memory_to_register(&mut self, target_register: u8) {
        for i in 0..=target_register {
            self.registers[i as usize] = self.memory[(self.index_register + i as u16) as usize];
        }

        if self.quirk_config.memory_index_register_increase {
            self.index_register += (target_register as u16) + 1;
        }
    }

    pub fn set_input(&mut self, input: u8, pressed: bool) {
        self.input[input as usize] = pressed;
    }

    fn convert_to_bcd(&mut self, target_register: u8) {
        let value = self.registers[target_register as usize];

        let hundreds = value / 100;
        let tens = (value / 10) % 10;
        let ones = value % 10;

        self.memory[self.index_register as usize] = hundreds;
        self.memory[self.index_register as usize + 1] = tens;
        self.memory[self.index_register as usize + 2] = ones;
    }

    fn register_copy(&mut self, target_register: u8, source_register: u8) {
        self.registers[target_register as usize] = self.registers[source_register as usize];
    }

    fn register_or(&mut self, target_register: u8, source_register: u8) {
        self.registers[0xF] = 0;
        self.registers[target_register as usize] = self.registers[target_register as usize] | self.registers[source_register as usize];
    }

    fn register_xor(&mut self, target_register: u8, source_register: u8) {
        self.registers[0xF] = 0;
        self.registers[target_register as usize] = self.registers[target_register as usize] ^ self.registers[source_register as usize];
    }

    fn register_and(&mut self, target_register: u8, source_register: u8) {
        self.registers[0xF] = 0;
        self.registers[target_register as usize] = self.registers[target_register as usize] & self.registers[source_register as usize];
    }

    fn register_shift(&mut self, target_register: u8, source_register: u8, inverse: bool) {
        let bit_out: u8;

        if self.quirk_config.source_vy_bitshift {
            self.registers[target_register as usize] = self.registers[source_register as usize];
        }

        if inverse {
            bit_out = (self.registers[target_register as usize] >> 7) & 1;
            self.registers[target_register as usize] <<= 1;
        } else {
            bit_out = self.registers[target_register as usize] & 1;
            self.registers[target_register as usize] >>= 1;
        }
        self.registers[0xF] = bit_out;
    }

    fn register_add(&mut self, target_register: u8, source_register: u8) {
        let x = self.registers[target_register as usize];
        let y = self.registers[source_register as usize];

        self.registers[target_register as usize] = x.wrapping_add(y);

        if y > (255 - x) {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn register_sub(&mut self, target_register: u8, source_register: u8, swap: bool) {
        let mut x = self.registers[target_register as usize];
        let mut y = self.registers[source_register as usize];

        if swap {
            std::mem::swap(&mut x, &mut y);
        }

        self.registers[target_register as usize] = x.wrapping_sub(y);

        if x >= y {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }


    fn return_sub(&mut self) {
        if let Some(position) = self.stack.pop() {
            self.set_program_counter(position);
        }
    }

    fn jump_sub(&mut self, position: u16) {
        self.stack.push(self.program_counter);
        self.set_program_counter(position);
    }

    fn jump_offset(&mut self, position: u16) {
        self.set_program_counter(position + self.registers[0] as u16);
    }

    fn register_conditional_skip(&mut self, register_a: u8, register_b: u8, inverse: bool) {
        let vx = self.registers[register_a as usize];
        let vy = self.registers[register_b as usize];

        if (vx == vy) ^ inverse {
            self.program_counter += 2;
        }
    }

    fn value_conditional_skip(&mut self, register: u8, value: u8, inverse: bool) {
        let vx = self.registers[register as usize];

        if (vx == value) ^ inverse {
            self.program_counter += 2;
        }
    }

    fn draw_sprite(&mut self, x: u8, y: u8, height: u8) {
        let x = self.registers[x as usize] % 64;
        let mut y = self.registers[y as usize] % 32;
        self.registers[0xF] = 0;

        for i in 0..height {
            let address = self.index_register + (i as u16);
            let sprite = self.memory[address as usize];

            let mut x = x;

            if y >= 32 {
                break;
            }

            for b in (0..8).rev() {
                let bit = (sprite >> b) & 1;
                let bit_bool = bit != 0;

                if x >= 64 {
                    break;
                }

                if self.display[y as usize][x as usize] && bit_bool {
                    self.registers[0xF] = 1;
                }

                self.display[y as usize][x as usize] ^= bit_bool;

                x += 1;
            }
            y += 1;
        }
        self.blocking_on_draw = true;
    }

    fn set_program_counter(&mut self, value: u16) {
        // println!("Setting program counter to: {:04X}", value);
        self.program_counter = value;
    }

    fn set_v_register(&mut self, register: u8, value: u8) {
        if register > 0xF {
            println!("Invalid V register: {:01X}", register);
            return;
        }

        self.registers[register as usize] = value;
    }

    fn add_v_register(&mut self, register: u8, value: u8) {
        //println!("Adding value to register v{:01X}: {:02X}", register, value);
        if register > 0xF {
            println!("Invalid V register: {:01X}", register);
            return;
        }

        self.registers[register as usize] = self.registers[register as usize].wrapping_add(value);
    }

    fn set_index_register(&mut self, value: u16) {
        self.index_register = value;
    }

    fn clear_screen(&mut self) {
        self.display = [[false; 64]; 32];
    }

    fn fetch_instruction(&self) -> u16 {
        let pc = self.program_counter as usize;
        let high_byte = self.memory[pc] as u16;
        let low_byte = self.memory[pc + 1] as u16;
        (high_byte << 8) | low_byte
    }

    pub fn set_rom(&mut self, rom: &Vec<u8>) {
        let end = std::cmp::min(rom.len(), self.memory.len() - 512);

        for (index, &byte) in rom.iter().enumerate().take(end) {
            self.memory[512 + index] = byte;
        }

        let font = get_font_chars();

        for (index, &byte) in font.iter().enumerate() {
            self.memory[0x050 + index as usize] = byte;
        }
    }

    fn index_to_font_char(&mut self, target_register: u8) {
        let char = self.registers[target_register as usize];
        self.index_register = 0x050 + (char as u16 * 5);
    }
}

impl Opcode {
    fn from_instruction(instruction: u16) -> Self {
        let opcode = ((instruction >> 12) & 0xF) as u8;
        let x = ((instruction >> 8) & 0xF) as u8;
        let y = ((instruction >> 4) & 0xF) as u8;
        let n = (instruction & 0xF) as u8;
        let nn = (instruction & 0xFF) as u8;
        let nnn = instruction & 0xFFF;

        Opcode { instruction, opcode, x, y, n, nn, nnn }
    }
}

impl QuirkConfig {
    fn new() -> Self {
        QuirkConfig {
            memory_index_register_increase: false,
            source_vy_bitshift: false,
        }
    }
}

pub fn get_font_chars() -> Vec<u8> {
    vec![
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80,  // F
    ]
}
