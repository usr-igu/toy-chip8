extern crate rand;
use self::rand::Rng;

pub struct Cpu {
    memory: [u8; 4096],
    v: [u8; 16], // registers
    i: u16,      // address register
    pc: u16,     // program counter
    sp: u8,      // stack pointer
    stack: [u16; 16],
    pub delay_timer: u8,
    pub sound_timer: u8,
    keyboard: [u8; 16],
    pub gfx: [u8; 64 * 32],
    pub key_pressed: bool,
    pub draw_flag: bool,
    load_store_quirk: bool,
    shift_quirk: bool,
}

pub fn new() -> Cpu {
    let mut chip8 = Cpu {
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
        key_pressed: false,
        draw_flag: false,
        load_store_quirk: true,
        shift_quirk: true,
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
                                           // load fontset
    chip8.memory[..chip8_fontset.len()].clone_from_slice(&chip8_fontset[..]);
    chip8
}

impl Cpu {
    pub fn load_rom(&mut self, rom: &[u8]) {
        self.memory[512..(rom.len() + 512)].clone_from_slice(&rom[..]);
    }

    pub fn cpu_tick(&mut self) {
        let opcode = (u16::from(self.memory[self.pc as usize]) << 8)
            | u16::from(self.memory[self.pc as usize + 1]);

        self.pc += 2; // go to the next instruction

        self.draw_flag = false;

        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let n = opcode & 0x000F;
        let nn = opcode & 0x00FF;
        let nnn = opcode & 0x0FFF;

        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    //0000 Execute machine language subroutine at address NNN
                    // 0x00 => panic!("unknown opcode {:X}", opcode), // TODO(fuzzyqu)
                    //00E0 Clear the screen
                    0xE0 => {
                        for i in 0..self.gfx.len() {
                            self.gfx[i] = 0;
                        }
                        self.draw_flag = true;
                    }
                    //00EE Return from a subroutine
                    0xEE => {
                        self.sp -= 1; // go down the stack
                        self.pc = self.stack[self.sp as usize]; // return from the routine
                    }
                    _ => panic!("unknown opcode {:X}", opcode),
                }
            }
            //1NNN Jump to address NNN
            0x1000 => {
                self.pc = nnn;
            }
            //2NNN Execute subroutine starting at address NNN
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            //3XNN Skip the following instruction if the value of register VX equals NN
            0x3000 => {
                if self.v[x as usize] == nn as u8 {
                    self.pc += 2;
                }
            }
            //4XNN Skip the following instruction if the value of register VX is not equal to NN
            0x4000 => {
                if self.v[x as usize] != nn as u8 {
                    self.pc += 2;
                }
            }
            //5XY0 Skip the following instruction if the value of register VX is equal to the value of register VY
            0x5000 => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            }
            //6XNN Store number NN in register VX
            0x6000 => {
                self.v[x as usize] = nn as u8;
            }
            //7XNN Add the value NN to register VX
            0x7000 => {
                self.v[x as usize] = self.v[x as usize].wrapping_add(nn as u8);
            }
            0x8000 => match opcode & 0x000F {
                //8XY0 Store the value of register VY in register VX
                0x0 => {
                    self.v[x as usize] = self.v[y as usize];
                }
                //8XY1 Set VX to VX OR VY
                0x1 => {
                    self.v[x as usize] |= self.v[y as usize];
                }
                //8XY2 Set VX to VX AND VY
                0x2 => {
                    self.v[x as usize] &= self.v[y as usize];
                }
                //8XY3 Set VX to VX XOR VY
                0x3 => {
                    self.v[x as usize] ^= self.v[y as usize];
                }
                //8XY4 Add the value of register VY to register VX
                //Set VF to 01 if a carry occurs
                //Set VF to 00 if a carry does not occur
                0x04 => {
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
                    if !self.shift_quirk {
                        self.v[0xF] = self.v[y as usize] & 0x1;
                        self.v[y as usize] >>= 1;
                        self.v[x as usize] = self.v[y as usize];
                    } else {
                        self.v[0xF] = self.v[x as usize] & 0x1;
                        self.v[x as usize] >>= 1;
                    }
                }
                //8XY7 Set register VX to the value of VY minus VX
                //Set VF to 00 if a borrow occurs
                //Set VF to 01 if a borrow does not occur
                0x07 => {
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
                    if !self.shift_quirk {
                        if !self.shift_quirk {
                            self.v[0xF] = (self.v[y as usize] >> 7) & 0x1;
                            self.v[y as usize] <<= 1;
                            self.v[x as usize] = self.v[y as usize];
                        }
                    } else {
                        self.v[0xF] = (self.v[x as usize] >> 7) & 0x1;
                        self.v[x as usize] <<= 1;
                    }
                }
                _ => panic!("unknown opcode {:X}", opcode),
            },
            //9XY0 Skip the following instruction if the value of register VX
            //is not equal to the value of register VY
            0x9000 => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
            }
            //ANNN Store memory address NNN in register I
            0xA000 => {
                self.i = nnn;
            }
            //BNNN Jump to address NNN + V0
            0xB000 => {
                self.pc = nnn + u16::from(self.v[0x0]);
            }
            //CXNN Set VX to a random number with a mask of NN
            0xC000 => {
                self.v[x as usize] = rand::thread_rng().gen::<u8>() & nn as u8;
            }
            //DXYN Draw a sprite at position VX, VY with N bytes of sprite data
            //starting at the address stored in I
            //Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
            0xD000 => {
                self.v[0xF] = 0;
                for j in 0..n {
                    let p = self.memory[(self.i + j as u16) as usize];
                    for i in 0..8 {
                        if (p & (128 >> i)) != 0 {
                            let index = (u16::from(self.v[x as usize]) + i
                                + (u16::from(self.v[y as usize]) + j) * 64)
                                % 2048;
                            if self.gfx[index as usize] == 1 {
                                // bit flipped
                                self.v[0xF] = 1;
                            }
                            self.gfx[index as usize] ^= 1;
                        }
                    }
                }
                self.draw_flag = true;
            }
            0xE000 => match opcode & 0x00FF {
                //EX9E Skip the following instruction if the key corresponding to
                //the hex value currently stored in register VX is pressed
                0x9E => {
                    if self.keyboard[self.v[x as usize] as usize] != 0 {
                        self.pc += 2;
                    }
                }
                //EXA1 Skip the following instruction if the key corresponding
                //to the hex value currently stored in register VX is not pressed
                0xA1 => {
                    if self.keyboard[self.v[x as usize] as usize] == 0 {
                        self.pc += 2;
                    }
                }
                _ => panic!("unknown opcode {:X}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                // FX07 Store the current value of the delay timer in register VX
                0x07 => {
                    self.v[x as usize] = self.delay_timer;
                }
                //FX0A Wait for a keypress and store the result in register VX
                0x0A => {
                    self.key_pressed = false;
                    for i in 0..self.keyboard.len() {
                        if self.keyboard[i] != 0 {
                            self.v[x as usize] = i as u8;
                            self.key_pressed = true;
                            break;
                        }
                    }
                    if !self.key_pressed {
                        return;
                    }
                }
                //FX15 Set the delay timer to the value of register VX
                0x15 => {
                    self.delay_timer = self.v[x as usize];
                }
                //FX18 Set the sound timer to the value of register VX
                0x18 => {
                    self.sound_timer = self.v[x as usize];
                }
                //FX1E Add the value stored in register VX to register I
                0x1E => {
                    self.i += u16::from(self.v[x as usize]);
                }
                //FX29 Set I to the memory address of the sprite data corresponding to
                //the hexadecimal digit stored in register VX
                0x29 => {
                    self.i = u16::from(self.v[x as usize]) * 0x5;
                }
                //FX33 	Store the binary-coded decimal equivalent of the value stored
                //in register VX at addresses I, I+1, and I+2
                0x33 => {
                    self.memory[self.i as usize] = self.v[x as usize] / 100;
                    self.memory[(self.i + 1) as usize] = (self.v[x as usize] / 10) % 10;
                    self.memory[(self.i + 2) as usize] = (self.v[x as usize] % 100) % 10;
                }
                //FX55 Store the values of registers V0 to VX inclusive in memory starting at address I
                //I is set to I + X + 1 after operation
                0x55 => {
                    for i in 0..x as usize + 1 {
                        self.memory[self.i as usize + i] = self.v[i];
                    }
                    if !self.load_store_quirk {
                        self.i = self.i + x + 1;
                    }
                }
                //FX65 Fill registers V0 to VX inclusive with the values
                //stored in memory starting at address I
                //I is set to I + X + 1 after operation
                0x65 => {
                    for i in 0..x as usize + 1 {
                        self.v[i] = self.memory[self.i as usize + i];
                    }
                    if !self.load_store_quirk {
                        self.i = self.i + x + 1;
                    }
                }
                _ => panic!("unknown opcode {:X}", opcode),
            },
            _ => panic!("unknown opcode {:X}", opcode),
        }
    }

    pub fn key_down(&mut self, key: u8) {
        self.keyboard[key as usize] = 1;
    }

    pub fn key_up(&mut self, key: u8) {
        self.keyboard[key as usize] = 0;
    }

    pub fn timers_tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}
