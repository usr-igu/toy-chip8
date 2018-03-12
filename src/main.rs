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
        let mut chip8 = Chip8 {
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
        };
        let chip8_fontset = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, //0
            0x20, 0x60, 0x20, 0x20, 0x70, //1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
            0x90, 0x90, 0xF0, 0x10, 0x10, //4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
            0xF0, 0x10, 0x20, 0x40, 0x40, //7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
            0xF0, 0x90, 0xF0, 0x90, 0x90, //A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
            0xF0, 0x80, 0x80, 0x80, 0xF0, //C
            0xE0, 0x90, 0x90, 0x90, 0xE0, //D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
            0xF0, 0x80, 0xF0, 0x80, 0x80]; //F
        for i in 0..chip8_fontset.len() {
            chip8.memory[i] = chip8_fontset[i];
        }
        chip8
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
                    _ => panic!("unknown opcode {:X}", opcode),
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
                self.v[x as usize] = self.v[x as usize].wrapping_add(nn as u8);
            }
            0x8000 => match opcode & 0x000F {
                //8XY0 Store the value of register VY in register VX
                0x0 => {
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    self.v[x as usize] = self.v[y as usize];
                }
                //8XY1 Set VX to VX OR VY
                0x1 => {
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    self.v[x as usize] |= self.v[y as usize];
                }
                //8XY2 Set VX to VX AND VY
                0x2 => {
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    self.v[x as usize] &= self.v[y as usize];
                }
                //8XY3 Set VX to VX XOR VY
                0x3 => {
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    self.v[x as usize] ^= self.v[y as usize];
                }
                //8XY4 Add the value of register VY to register VX
                //Set VF to 01 if a carry occurs
                //Set VF to 00 if a carry does not occur
                0x04 => {
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    self.v[0xF] = 0;
                    let (sum, carry) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                    if carry {
                        self.v[0xF] = 1
                    };
                    self.v[x as usize] = sum;
                }
                //8XY5 Subtract the value of register VY from register VX
                //Set VF to 00 if a borrow occurs
                //Set VF to 01 if a borrow does not occur
                0x05 => {
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    self.v[0xF] = 1;
                    let (sub, borrow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                    if borrow {
                        self.v[0xF] = 0
                    };
                    self.v[x as usize] = sub;
                }
                //8XY6 Store the value of register VY shifted right one bit in register VX
                //Set register VF to the least significant bit prior to the shift
                0x06 => {
                    // TODO(fuzzy): Verificar se é necessário modificar VY.
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    self.v[0x0F] = self.v[y as usize] & 0x1;
                    self.v[y as usize] >>= 1;
                    self.v[x as usize] = self.v[y as usize];
                }
                //8XY7 Set register VX to the value of VY minus VX
                //Set VF to 00 if a borrow occurs
                //Set VF to 01 if a borrow does not occur
                0x07 => {
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    self.v[0xF] = 1;
                    let (sub, borrow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                    if borrow {
                        self.v[0xF] = 0
                    };
                    self.v[x as usize] = sub;
                }
                //8XYE Store the value of register VY shifted left one bit in register VX
                //Set register VF to the most significant bit prior to the shift
                0x0E => {
                    // TODO(fuzzy): Verificar se é necessário modificar VY.
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    self.v[0x0F] = self.v[y as usize] >> 7;
                    self.v[y as usize] <<= 1;
                    self.v[x as usize] = self.v[y as usize];
                }
                _ => panic!("unknown opcode {:X}", opcode),
            },
            //9XY0 Skip the following instruction if the value of register VX
            //is not equal to the value of register VY
            0x9000 => {
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
            }
            //ANNN Store memory address NNN in register I
            0xA000 => {
                let nnn = opcode & 0x0FFF;
                self.i = nnn;
            }
            //DXYN Draw a sprite at position VX, VY with N bytes of sprite data
            //starting at the address stored in I
            //Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
            0xD000 => {
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                let n = opcode & 0x000F;
                for j in 0..n {
                    let p = self.memory[(self.i + j as u16) as usize];
                    for i in 0..8 {
                        if (p & (128 >> i)) != 0 {
                            let index = self.v[x as usize] as u16 + i
                                + (self.v[y as usize] as u16 + j) * 64;
                            if self.gfx[index as usize] == 1 { // bit flipped
                                self.v[0xF] = 1; 
                            }
                            self.gfx[index as usize] ^= 1;
                        }
                    }
                }
            }
            0xF000 => match opcode & 0x00FF {
                // FX07 Store the current value of the delay timer in register VX
                0x07 => {
                    let x = (opcode & 0x0F00) >> 8;
                    self.v[x as usize] = self.delay_timer;
                }
                0x0A => unimplemented!(),
                //FX15 Set the delay timer to the value of register VX
                0x15 => {
                    let x = (opcode & 0x0F00) >> 8;
                    self.delay_timer = self.v[x as usize];
                }
                0x18 => unimplemented!(),
                0x1E => unimplemented!(),
                //FX29 Set I to the memory address of the sprite data corresponding to
                //the hexadecimal digit stored in register VX
                0x29 => {
                    let x = (opcode & 0x0F00) >> 8;
                    self.i = u16::from(self.v[x as usize]) * 0x05;
                }
                //FX33 	Store the binary-coded decimal equivalent of the value stored
                //in register VX at addresses I, I+1, and I+2
                0x33 => {
                    let x = (opcode & 0x0F00) >> 8;
                    self.memory[self.i as usize] = self.v[x as usize] / 100;
                    self.memory[(self.i + 1) as usize] = (self.v[x as usize] / 10) % 10;
                    self.memory[(self.i + 2) as usize] = (self.v[x as usize] % 100) % 10;
                }
                0x55 => unimplemented!(),
                //FX65 Fill registers V0 to VX inclusive with the values
                //stored in memory starting at address I
                //I is set to I + X + 1 after operation
                0x65 => {
                    let x = (opcode & 0x0F00) >> 8;
                    for i in 0..x as usize + 1 {
                        self.v[i] = self.memory[self.i as usize + i];
                    }
                    self.i = self.i + x + 1;
                }
                _ => panic!("unknown opcode {:X}", opcode),
            },
            _ => panic!("unknown opcode {:X}", opcode),
        }
        self.pc += 2; // go to the next instruction
    }
}

use std::process::Command;

fn main() {
    // let mut f = File::open("test.ch8").unwrap();
    let mut f = File::open("GAMES/BREAKOUT").unwrap();

    let mut buf = Vec::new();

    f.read_to_end(&mut buf).unwrap();

    let mut chip8 = Chip8::new();

    chip8.load_rom(&buf);

    loop {
        chip8.cycle();
        for j in 0..32 {
            for i in 0..64 {
                if chip8.gfx[i + (j * 64)] != 0 {
                    print!(".");
                } else {
                    print!(" ");
                }
            }
            println!();
        }
        // std::thread::sleep(std::time::Duration::from_millis(25));
        let output = Command::new("clear").output().unwrap();
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
}
