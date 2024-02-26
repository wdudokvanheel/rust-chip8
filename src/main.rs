use std::usize;

fn main() {
    println!("Chip 8 Emu");
    
    let rom = load_rom();

    let mut chip = Chip8::new();
    chip.set_rom(rom);
    for _ in 0 .. 32 {
        chip.cycle();
        //chip.debug_print();
    }
    print_display(chip.display);
}

fn load_rom() -> Vec<u8>{
    let rom = include_bytes!("roms/ibm.ch8");
    rom.to_vec()
}


struct Chip8{
    memory: [u8; 4096],
    registers: [u8; 16], 
    index_register: u16,
    program_counter: u16,
    display: [[bool; 64]; 32],
}

impl Chip8{
    fn new() -> Self {
        Chip8 {
            memory: [0; 4096],
            registers: [0; 16],
            index_register: 0,
            program_counter: 0x200,
            display: [[false; 64]; 32],
        }
    }

    fn cycle(&mut self){
        let instruction = self.fetch_instruction();
        println!("\nNext instruction: {:04X}", instruction);
        let opcode = Opcode::from_instruction(instruction);

        self.program_counter+=2;

        match opcode  {
            Opcode { opcode: 0xD, x, y, n, ..} => self.draw_sprite(x, y, n),
            Opcode { opcode: 0x1, nnn, ..} => self.set_program_counter(nnn),
            Opcode { opcode: 0x7, x, nn, ..} => self.add_v_register(x, nn),
            Opcode { opcode: 0x6, x, nn, ..} => self.set_v_register(x, nn),
            Opcode { opcode: 0xA, nnn, .. } => self.set_index_register(nnn),
            Opcode { instruction: 0x00E0, ..} => self.clear_screen(),
            Opcode { opcode, .. } => println!("Opcode not supported: {:01X}", opcode),
        }
    }

    fn draw_sprite(&mut self, x: u8, y: u8, height: u8){
        let mut x = self.registers[x as usize] % 64;
        let mut y = self.registers[y as usize] % 32;
        self.registers[0xF] = 0;

        for i in 0 .. height  {
            let address = self.index_register + (i as u16);
            let sprite = self.memory[address as usize];

            for b in (0..8).rev() {
                let bit = (sprite >> b) & 1; 
                let bit_bool = bit != 0;

                if bit_bool && x < 64 && y < 32 {
                    self.display[(y % 32) as usize][(x % 64) as usize] = true;
                    
                }
                else if bit_bool{
                    println!("out of bounds: {},{}", x, y);
                }
                x+=1;
            }
            y+=1;
            x-=8;
        } 

        println!("Drawing sprite at {},{} with height of {}", x, y, height+1);

    }

    fn set_program_counter(&mut self, value: u16){
        println!("Setting program counter to: {:04X}", value);
        self.program_counter = value;
    }

    fn set_v_register(&mut self, register: u8, value: u8){
        println!("Setting register v{:01X} to {:02X}", register, value);
            if register > 0xF {
                println!("Invalid V register: {:01X}", register);
                return;
            }

        self.registers[register as usize] = value;
    }


    fn add_v_register(&mut self, register: u8, value: u8){
        println!("Adding value to register v{:01X}: {:02X}", register, value);
            if register > 0xF {
                println!("Invalid V register: {:01X}", register);
                return;
            }

        self.registers[register as usize] += value;
    }

    fn set_index_register(&mut self, value: u16){
        println!("Setting index register to {:04X}", value);
        self.index_register = value;
    }
   
    fn clear_screen(&mut self){
        // TODO: Clear screen
        println!("Clear display")
    }

    fn fetch_instruction(&self) -> u16 {
        let pc = self.program_counter as usize;
        let high_byte = self.memory[pc] as u16; 
        let low_byte = self.memory[pc + 1] as u16; 
        (high_byte << 8) | low_byte 
    }

    fn set_rom(&mut self, rom: Vec<u8>){
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

struct Opcode {
    instruction: u16,
    opcode: u8,
    x: u8,
    y: u8,
    n: u8,
    nn: u8,
    nnn: u16,
}

impl Opcode {
    fn from_instruction(instruction: u16) -> Self {
        let opcode = ((instruction >> 12) & 0xF) as u8;
        let x = ((instruction >> 8) & 0xF) as u8;
        let y = ((instruction >> 4) & 0xF) as u8;
        let n = (instruction & 0xF) as u8;
        let nn = (instruction & 0xFF) as u8;
        let nnn = instruction & 0xFFF;
/*
        println!("Opcode: {:01X}", opcode);
        println!("x: {:01X}", x);
        println!("y: {:01X}", y);
        println!("n: {:01X}", n);
        println!("nn: {:02X}", nn);
        println!("nnn: {:04X}", nnn);
        */

        Opcode { instruction, opcode, x, y, n, nn, nnn }
    }
}

fn print_display(display: [[bool; 64]; 32]) {
    for row in display.iter() {
        for &pixel in row.iter() {
            if pixel {
                print!("█"); 
            } else {
                print!(" "); 
            }
        }
        println!(); 
    }
}

