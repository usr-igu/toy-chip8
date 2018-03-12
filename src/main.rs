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
            self.memory[512 + i] = rom[i];
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
