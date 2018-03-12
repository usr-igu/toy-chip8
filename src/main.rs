use std::fs::File;
use std::io::Read;

struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16], // registers
    i: u16,      // address register
    pc: u16,     // program counter
    sp: u8,      // stack pointer
    stack: [u16; 16],
    delay_timer: u8,
    sound_timer: u8,
    keyboard: [u8; 16],
    gfx: [u8; 64 * 32],
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200, // chip 8 programs start at position 512
            sp: 0,
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            keyboard: [0; 16],
            gfx: [0; 64 * 32],
        }
    }

    fn load_rom(&mut self, rom: &[u8]) {
        for i in 0..rom.len() {
            self.memory[0x200 + i] = rom[i];
        }
    }

    fn cycle(&mut self) {
        let opcode = (u16::from(self.memory[self.pc as usize]) << 8)
            | u16::from(self.memory[self.pc as usize + 1]);

        println!("opcode: {:X}", opcode);

        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    //0000 Execute machine language subroutine at address NNN
                    0x00 => panic!("0x00"), // TODO(fuzzyqu)
                    //00E0 Clear the screen
                    0xE0 => for i in 0..self.gfx.len() {
                        self.gfx[i] = 0;
                    },
                    //00EE Return from a subroutine
                    0xEE => {
                        self.pc = self.stack[self.sp as usize]; // return from the routine
                        self.sp -= 1; // go down the stack
                    }
                    _ => panic!("unknown opcode {:x}", opcode),
                }
            }
            //1NNN Jump to address NNN
            0x1000 => {
                let nnn = opcode & 0x0FFF;
                self.pc = nnn;
            }
            //2NNN Execute subroutine starting at address NNN
            0x2000 => {
                let nnn = opcode & 0x0FFF;
                self.stack[self.sp as usize] = self.pc;
                self.pc = nnn;
            }
            //3XNN Skip the following instruction if the value of register VX equals NN
            0x3000 => {
                let x = (opcode & 0x0F00) >> 8;
                let nn = opcode & 0x00FF;
                if self.v[x as usize] == nn as u8 {
                    self.pc += 2;
                }
            }
            //4XNN Skip the following instruction if the value of register VX is not equal to NN
            0x4000 => {
                let x = (opcode & 0x0F00) >> 8;
                let nn = opcode & 0x00FF;
                if self.v[x as usize] != nn as u8 {
                    self.pc += 2;
                }
            }
            //5XY0 Skip the following instruction if the value of register VX is equal to the value of register VY
            0x5000 => {
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            }
            //6XNN Store number NN in register VX
            0x6000 => {
                let x = (opcode & 0x0F00) >> 8;
                let nn = opcode & 0x00FF;
                self.v[x as usize] = nn as u8;
            }
            //7XNN Add the value NN to register VX
            0x7000 => {
                let x = (opcode & 0x0F00) >> 8;
                let nn = opcode & 0x00FF;
                self.v[x as usize] += nn as u8;
            }
            _ => panic!("unknown opcode {:x}", opcode),
        }
        self.pc += 2; // go to the next instruction
    }
}

fn main() {
    let mut f = File::open("test.ch8").unwrap();
    let mut buf = Vec::new();

    f.read_to_end(&mut buf).unwrap();

    let mut chip8 = Chip8::new();

    chip8.load_rom(&buf);

    loop {
        chip8.cycle();
    }
}
