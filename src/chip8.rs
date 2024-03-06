use std::process::{self, exit};

pub struct Chip8 {
    memory: [u8; 4096],
    registers: [u8; 16],
    index_register: u16,
    program_counter: u16,
    stack: Vec<u16>,
    pub display: [[bool; 64]; 32],
    delay_timer: u8,
    sound_timer: u8,
    legacy_instructions: bool,
    total_cycles: u32,
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

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            memory: [0; 4096],
            registers: [0; 16],
            index_register: 0,
            program_counter: 0x200,
            stack: vec!(),
            display: [[false; 64]; 32],
            legacy_instructions: false,
            total_cycles: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn cycle(&mut self) {
        self.total_cycles += 1;

        if(self.total_cycles > 23){
            //exit(1);
        }

        let instruction = self.fetch_instruction();
        let opcode = Opcode::from_instruction(instruction);

        self.program_counter += 2;
        if(instruction != 0x1542) {
            println!("Instruction {}: {:04x}", self.total_cycles, instruction);
        }
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
            Opcode { opcode: 0xD, x, y, n, .. } => self.draw_sprite(x, y, n),
            Opcode { opcode: 0xF, nn: 0x33, x, .. } => self.convert_to_bcd(x),
            Opcode { opcode: 0xF, nn: 0x55, x, .. } => self.register_to_memory(x),
            Opcode { opcode: 0xF, nn: 0x65, x, .. } => self.memory_to_register(x),
            Opcode { opcode: 0xF, nn: 0x1E, x, .. } => self.add_index_register(x),
            //Opcode { opcode: 0xF, nn: 0x15, x, .. } => self.set_delay_timer(x),
            Opcode { instruction: 0x00E0, .. } => self.clear_screen(),
            Opcode { instruction: 0x00EE, .. } => self.return_sub(),
            Opcode { instruction, .. } => {
                println!("Instruction not supported: {:04X}", instruction);
                process::exit(0x0100);
            }
        }
    }

    pub fn update(&mut self){
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn add_index_register(&mut self, target_register: u8) {
        // TODO: Add amiga style VF register handling
        self.index_register = self.index_register.wrapping_add(self.registers[target_register as usize].into());
    }

    fn register_to_memory(&mut self, target_register: u8) {
        // TODO: Add support for legacy
        println!("Writing memory to v0 to v{:01x}", target_register);
        for i in 0..=target_register {
            self.memory[(self.index_register + i as u16) as usize] = self.registers[i as usize];
        }
    }

    fn memory_to_register(&mut self, target_register: u8) {
        for i in 0..=target_register {
            self.registers[i as usize] = self.memory[(self.index_register + i as u16) as usize];
        }

        if self.legacy_instructions {
            self.index_register += target_register as u16;
        }
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
        self.registers[target_register as usize] = self.registers[target_register as usize] | self.registers[source_register as usize];
    }

    fn register_xor(&mut self, target_register: u8, source_register: u8) {
        self.registers[target_register as usize] = self.registers[target_register as usize] ^ self.registers[source_register as usize];
    }

    fn register_and(&mut self, target_register: u8, source_register: u8) {
        self.registers[target_register as usize] = self.registers[target_register as usize] & self.registers[source_register as usize];
    }

    fn register_shift(&mut self, target_register: u8, source_register: u8, inverse: bool) {
        let bit_out: u8;

        if self.legacy_instructions {
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

    fn register_conditional_skip(&mut self, register_a: u8, register_b: u8, inverse: bool) {
        let vx = self.registers[register_a as usize];
        let vy = self.registers[register_b as usize];

        if (vx == vy) ^ inverse {
            self.program_counter += 2;
        }
    }

    fn value_conditional_skip(&mut self, register: u8, value: u8, inverse: bool) {
        let vx = self.registers[register as usize];
        println!("Checking if equal v{:02x} = {:02x} :: {:02x}", register, value, vx);

        if (vx == value) ^ inverse {
            self.program_counter += 2;
        }
    }

    fn draw_sprite(&mut self, x: u8, y: u8, height: u8) {
        let mut x = self.registers[x as usize] % 64;
        let mut y = self.registers[y as usize] % 32;
        self.registers[0xF] = 0;

        for i in 0..height {
            let address = self.index_register + (i as u16);
            let sprite = self.memory[address as usize];

            for b in (0..8).rev() {
                let bit = (sprite >> b) & 1;
                let bit_bool = bit != 0;

                if bit_bool && x < 64 && y < 32 {
                    self.display[(y % 32) as usize][(x % 64) as usize] = true;
                } else if bit_bool {
                    println!("out of bounds: {},{}", x, y);
                }
                x += 1;
            }
            y += 1;
            x -= 8;
        }

        println!("Drawing sprite at {},{} with height of {}", x, y, height + 1);
    }

    fn set_program_counter(&mut self, value: u16) {
       // println!("Setting program counter to: {:04X}", value);
        self.program_counter = value;
    }

    fn set_v_register(&mut self, register: u8, value: u8) {
        println!("Setting register v{:01X} to {:02X}", register, value);
        if register > 0xF {
            println!("Invalid V register: {:01X}", register);
            return;
        }

        self.registers[register as usize] = value;
    }


    fn add_v_register(&mut self, register: u8, value: u8) {
        println!("Adding value to register v{:01X}: {:02X}", register, value);
        if register > 0xF {
            println!("Invalid V register: {:01X}", register);
            return;
        }

        self.registers[register as usize] = self.registers[register as usize].wrapping_add(value);
    }

    fn set_index_register(&mut self, value: u16) {
        println!("Setting index register to {:04X}", value);
        self.index_register = value;
    }

    fn clear_screen(&mut self) {
        // TODO: Clear screen
        println!("Clear display")
    }

    fn fetch_instruction(&self) -> u16 {
        let pc = self.program_counter as usize;
        let high_byte = self.memory[pc] as u16;
        let low_byte = self.memory[pc + 1] as u16;
        (high_byte << 8) | low_byte
    }

    pub fn set_rom(&mut self, rom: Vec<u8>) {
        let end = std::cmp::min(rom.len(), self.memory.len() - 512);

        for (index, &byte) in rom.iter().enumerate().take(end) {
            self.memory[512 + index] = byte;
        }
    }

    fn debug_print(&self) {
        println!("Chip8 Emulator State:");
        println!("Registers:");
        for (index, &value) in self.registers.iter().enumerate() {
            println!("V{:X}: {:02X}", index, value);
        }
        println!("Index Register: {:04X}", self.index_register);
        println!("Program Counter: {:04X}", self.program_counter);
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

pub fn load_rom() -> Vec<u8> {
    // let rom = include_bytes!("roms/ibm.ch8");
    // let rom = include_bytes!("roms/corax.plus.ch8");
     let rom = include_bytes!("roms/flags.ch8");
    // let rom = include_bytes!("roms/quirks.ch8");
    rom.to_vec()
}
