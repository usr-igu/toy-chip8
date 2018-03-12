extern crate rand;

use rand::Rng;

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
    key_press: bool,
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
            key_press: false,
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
                    // 0x00 => panic!("unknown opcode {:X}", opcode), // TODO(fuzzyqu)
                    //00E0 Clear the screen
                    0xE0 => for i in 0..self.gfx.len() {
                        self.gfx[i] = 0;
                    },
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
                let nnn = opcode & 0x0FFF;
                self.pc = nnn;
            }
            //2NNN Execute subroutine starting at address NNN
            0x2000 => {
                let nnn = opcode & 0x0FFF;
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
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
                    self.v[0xF] = 0;
                    let x = (opcode & 0x0F00) >> 8;
                    if (self.v[x as usize]) & 0x1 == 1 {
                        self.v[0xF] = 1;
                    }
                    self.v[x as usize] >>= 1;
                    // TODO(fuzzy): Verificar se é necessário modificar VY.
                    // let x = (opcode & 0x0F00) >> 8;
                    // let y = (opcode & 0x00F0) >> 4;
                    // self.v[0x0F] = self.v[y as usize] & 0x1;
                    // self.v[y as usize] >>= 1;
                    // self.v[x as usize] = self.v[y as usize];
                    // self.v[x as usize] >>= 1;
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
                    self.v[0xF] = 0;
                    let x = (opcode & 0x0F00) >> 8;
                    if (self.v[x as usize] >> 7) & 0x1 == 1 {
                        self.v[0xF] = 1;
                    }
                    self.v[x as usize] <<= 1;
                    // let y = (opcode & 0x00F0) >> 4;
                    // self.v[0xF] = self.v[y as usize] >> 7;
                    // self.v[y as usize] <<= 1;
                    // self.v[x as usize] = self.v[y as usize];
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
            //BNNN Jump to address NNN + V0
            0xB000 => {
                let nnn = opcode & 0x0FFF;
                self.pc = nnn + self.v[0x0] as u16;
            }
            //CXNN Set VX to a random number with a mask of NN
            0xC000 => {
                let x = (opcode & 0x0F00) >> 8;
                let nn = opcode & 0x00FF;
                self.v[x as usize] = rand::thread_rng().gen::<u8>() & nn as u8;
            }
            //DXYN Draw a sprite at position VX, VY with N bytes of sprite data
            //starting at the address stored in I
            //Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
            0xD000 => {
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                let n = opcode & 0x000F;
                self.v[0xF] = 0;
                for j in 0..n {
                    let p = self.memory[(self.i + j as u16) as usize];
                    for i in 0..8 {
                        if (p & (128 >> i)) != 0 {
                            let index = self.v[x as usize] as u16 + i
                                + (self.v[y as usize] as u16 + j) * 64;
                            if self.gfx[index as usize] == 1 {
                                // bit flipped
                                self.v[0xF] = 1;
                            }
                            self.gfx[index as usize] ^= 1;
                        }
                    }
                }
            }
            0xE000 => match opcode & 0x00FF {
                //EX9E Skip the following instruction if the key corresponding to
                //the hex value currently stored in register VX is pressed
                0x9E => {
                    let x = (opcode & 0x0F00) >> 8;
                    if self.keyboard[self.v[x as usize] as usize] != 0 {
                        self.pc += 2;
                    }
                }
                //EXA1 Skip the following instruction if the key corresponding
                //to the hex value currently stored in register VX is not pressed
                0xA1 => {
                    let x = (opcode & 0x0F00) >> 8;
                    if self.keyboard[self.v[x as usize] as usize] == 0 {
                        self.pc += 2;
                    }
                }
                _ => panic!("unknown opcode {:X}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                // FX07 Store the current value of the delay timer in register VX
                0x07 => {
                    let x = (opcode & 0x0F00) >> 8;
                    self.v[x as usize] = self.delay_timer;
                }
                //FX0A Wait for a keypress and store the result in register VX
                0x0A => {
                    let x = (opcode & 0x0F00) >> 8;
                    self.key_press = false;
                    for i in 0..self.keyboard.len() {
                        if self.keyboard[i] != 0 {
                            self.v[x as usize] = i as u8;
                            self.key_press = true;
                            break;
                        }
                    }
                    if !self.key_press {
                        return;
                    }
                }
                //FX15 Set the delay timer to the value of register VX
                0x15 => {
                    let x = (opcode & 0x0F00) >> 8;
                    self.delay_timer = self.v[x as usize];
                }
                //FX18 Set the sound timer to the value of register VX
                0x18 => {
                    let x = (opcode & 0x0F00) >> 8;
                    self.sound_timer = self.v[x as usize];
                }
                //FX1E Add the value stored in register VX to register I
                0x1E => {
                    let x = (opcode & 0x0F00) >> 8;
                    self.i += self.v[x as usize] as u16;
                }
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
                //FX55 Store the values of registers V0 to VX inclusive in memory starting at address I
                //I is set to I + X + 1 after operation
                0x55 => {
                    let x = (opcode & 0x0F00) >> 8;
                    for i in 0..x as usize + 1 {
                        self.memory[self.i as usize + i] = self.v[i];
                        // self.i += 1;
                    }
                    // self.i = self.i + x + 1;
                }
                //FX65 Fill registers V0 to VX inclusive with the values
                //stored in memory starting at address I
                //I is set to I + X + 1 after operation
                0x65 => {
                    let x = (opcode & 0x0F00) >> 8;
                    for i in 0..x as usize + 1 {
                        self.v[i] = self.memory[self.i as usize + i];
                        // self.i += 1;
                    }
                    // self.i = self.i + x + 1;
                }
                _ => panic!("unknown opcode {:X}", opcode),
            },
            _ => panic!("unknown opcode {:X}", opcode),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        self.pc += 2; // go to the next instruction
    }
}

extern crate sdl2;
use std::fs::File;
use std::io::Read;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

fn main() {
    let mut f = File::open("test.ch8").unwrap();
    // let mut f = File::open("GAMES/INVADERS").unwrap();
    // let mut f = File::open("GAMES/PONG").unwrap();

    let mut buf = Vec::new();

    f.read_to_end(&mut buf).unwrap();

    let mut chip8 = Chip8::new();

    chip8.load_rom(&buf);

    let window_width = 800;
    let window_height = 600;
    let block_size = 20u32;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("chip8", window_width, window_height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(186, 255, 201));
    canvas.clear();
    canvas.present();
    'game_loop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Escape => {
                        break 'game_loop;
                    }
                    Keycode::Num1 => {
                        chip8.keyboard[0x1] = 1;
                        println!("key down 1 (2)")
                    }
                    Keycode::Num2 => {
                        chip8.keyboard[0x2] = 1;
                        println!("key down 2 (2)")
                    }
                    Keycode::Num3 => {
                        chip8.keyboard[0x3] = 1;
                        println!("key down 3 (3)")
                    }
                    Keycode::Num4 => {
                        chip8.keyboard[0xC] = 1;
                        println!("key down 4 (C)")
                    }
                    Keycode::Q => {
                        chip8.keyboard[0x4] = 1;
                        println!("key down Q (4)")
                    }
                    Keycode::W => {
                        chip8.keyboard[0x5] = 1;
                        println!("key down W (5)")
                    }
                    Keycode::E => {
                        chip8.keyboard[0x6] = 1;
                        println!("key down E (6)")
                    }
                    Keycode::R => {
                        chip8.keyboard[0xD] = 1;
                        println!("key down R (D)")
                    }
                    Keycode::A => {
                        chip8.keyboard[0x7] = 1;
                        println!("key down A (7)")
                    }
                    Keycode::S => {
                        chip8.keyboard[0x8] = 1;
                        println!("key down S (8)")
                    }
                    Keycode::D => {
                        chip8.keyboard[0x9] = 1;
                        println!("key down D (9)")
                    }
                    Keycode::F => {
                        chip8.keyboard[0xE] = 1;
                        println!("key down F (E)")
                    }
                    Keycode::Z => {
                        chip8.keyboard[0xA] = 1;
                        println!("key down Z (A)")
                    }
                    Keycode::X => {
                        chip8.keyboard[0x0] = 1;
                        println!("key down X (0)")
                    }
                    Keycode::C => {
                        chip8.keyboard[0xB] = 1;
                        println!("key down C (B)")
                    }
                    Keycode::V => {
                        chip8.keyboard[0xF] = 1;
                        println!("key down V (F)")
                    }
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Num1 => {
                        chip8.keyboard[0x1] = 0;
                        println!("key up 1 (2)")
                    }
                    Keycode::Num2 => {
                        chip8.keyboard[0x2] = 0;
                        println!("key up 2 (2)")
                    }
                    Keycode::Num3 => {
                        chip8.keyboard[0x3] = 0;
                        println!("key up 3 (3)")
                    }
                    Keycode::Num4 => {
                        chip8.keyboard[0xC] = 0;
                        println!("key up 4 (C)")
                    }
                    Keycode::Q => {
                        chip8.keyboard[0x4] = 0;
                        println!("key up Q (4)")
                    }
                    Keycode::W => {
                        chip8.keyboard[0x5] = 0;
                        println!("key up W (5)")
                    }
                    Keycode::E => {
                        chip8.keyboard[0x6] = 0;
                        println!("key up E (6)")
                    }
                    Keycode::R => {
                        chip8.keyboard[0xD] = 0;
                        println!("key up R (D)")
                    }
                    Keycode::A => {
                        chip8.keyboard[0x7] = 0;
                        println!("key up A (7)")
                    }
                    Keycode::S => {
                        chip8.keyboard[0x8] = 0;
                        println!("key up S (8)")
                    }
                    Keycode::D => {
                        chip8.keyboard[0x9] = 0;
                        println!("key up D (9)")
                    }
                    Keycode::F => {
                        chip8.keyboard[0xE] = 0;
                        println!("key up F (E)")
                    }
                    Keycode::Z => {
                        chip8.keyboard[0xA] = 0;
                        println!("key up Z (A)")
                    }
                    Keycode::X => {
                        chip8.keyboard[0x0] = 0;
                        println!("key up X (0)")
                    }
                    Keycode::C => {
                        chip8.keyboard[0xB] = 0;
                        println!("key up C (B)")
                    }
                    Keycode::V => {
                        chip8.keyboard[0xF] = 0;
                        println!("key down V (F)")
                    }
                    _ => {}
                },
                Event::Quit { .. } => break 'game_loop,
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(186, 255, 201));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 179, 186));
        chip8.cycle();
        for j in 0..32 {
            for i in 0..64 {
                if chip8.gfx[i + (j * 64)] != 0 {
                    canvas
                        .fill_rect(Rect::new(
                            (i * window_width as usize / 64) as i32 - block_size as i32,
                            (j * window_height as usize / 32) as i32 + block_size as i32,
                            block_size,
                            block_size,
                        ))
                        .unwrap();
                }
            }
        }
        canvas.present();
    }
}
