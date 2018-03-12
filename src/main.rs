struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16], // registers
    i: u16,      // address register
    pc: u16,     // program counter
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
            v: [0; 16], // registers
            i: 0,       // address register
            pc: 0x200,  // chip 8 programs start at position 512
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            keyboard: [0; 16],
            gfx: [0; 64 * 32],
        }
    }

    fn cycle(&mut self) {
        
    }
}

fn main() {
    let mut chip8 = Chip8::new();
}
