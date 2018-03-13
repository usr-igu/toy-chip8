extern crate sdl2;
use std::fs::File;
use std::io::Read;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

mod chip8;

fn main() {
    let rom_path = match std::env::args().nth(1) {
        Some(path) => path,
        None => panic!("invalid ROM path"),
    };

    let mut f = File::open(rom_path).unwrap();

    let mut buf = Vec::new();

    f.read_to_end(&mut buf).unwrap();

    let mut chip = chip8::new();

    chip.load_rom(&buf);

    let window_width = 800;
    let window_height = 600;
    let block_size = 10u32;

    let background_color = Color::RGB(186, 255, 201);
    let block_color = Color::RGB(255, 179, 186);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("chip8 interpreter", window_width, window_height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut timers_past = std::time::Instant::now();
    let mut cpu_past = std::time::Instant::now();

    let timers_tickrate =
        std::time::Duration::from_millis(f64::floor((1.0 / 60.0) * 1000.0) as u64); // 60hz

    let cpu_tickrate = std::time::Duration::from_millis(f64::floor((1.0 / 500.0) * 1000.0) as u64);

    // canvas.set_draw_color(background_color);
    // canvas.clear();
    // canvas.present();
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
                        chip.key_down(0x1);
                    }
                    Keycode::Num2 => {
                        chip.key_down(0x2);
                    }
                    Keycode::Num3 => {
                        chip.key_down(0x3);
                    }
                    Keycode::Num4 => {
                        chip.key_down(0xC);
                    }
                    Keycode::Q => {
                        chip.key_down(0x4);
                    }
                    Keycode::W => {
                        chip.key_down(0x5);
                    }
                    Keycode::E => {
                        chip.key_down(0x6);
                    }
                    Keycode::R => {
                        chip.key_down(0xD);
                    }
                    Keycode::A => {
                        chip.key_down(0x7);
                    }
                    Keycode::S => {
                        chip.key_down(0x8);
                    }
                    Keycode::D => {
                        chip.key_down(0x9);
                    }
                    Keycode::F => {
                        chip.key_down(0xE);
                    }
                    Keycode::Z => {
                        chip.key_down(0xA);
                    }
                    Keycode::X => {
                        chip.key_down(0x0);
                    }
                    Keycode::C => {
                        chip.key_down(0xB);
                    }
                    Keycode::V => {
                        chip.key_down(0xF);
                    }
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Num1 => {
                        chip.key_up(0x1);
                    }
                    Keycode::Num2 => {
                        chip.key_up(0x2);
                    }
                    Keycode::Num3 => {
                        chip.key_up(0x3);
                    }
                    Keycode::Num4 => {
                        chip.key_up(0xC);
                    }
                    Keycode::Q => {
                        chip.key_up(0x4);
                    }
                    Keycode::W => {
                        chip.key_up(0x5);
                    }
                    Keycode::E => {
                        chip.key_up(0x6);
                    }
                    Keycode::R => {
                        chip.key_up(0xD);
                    }
                    Keycode::A => {
                        chip.key_up(0x7);
                    }
                    Keycode::S => {
                        chip.key_up(0x8);
                    }
                    Keycode::D => {
                        chip.key_up(0x9);
                    }
                    Keycode::F => {
                        chip.key_up(0xE);
                    }
                    Keycode::Z => {
                        chip.key_up(0xA);
                    }
                    Keycode::X => {
                        chip.key_up(0x0);
                    }
                    Keycode::C => {
                        chip.key_up(0xB);
                    }
                    Keycode::V => {
                        chip.key_up(0xF);
                    }
                    _ => {}
                },
                Event::Quit { .. } => break 'game_loop,
                _ => {}
            }
        }

        let timers_now = std::time::Instant::now();
        let cpu_now = std::time::Instant::now();

        let timers_ticks = timers_now - timers_past;
        let cpu_ticks = cpu_now - cpu_past;

        if cpu_ticks > cpu_tickrate {
            chip.cpu_tick();
            cpu_past = cpu_now;
        }

        if timers_ticks > timers_tickrate {
            chip.timers_tick();
            timers_past = timers_now;
        }

        if chip.draw_flag {
            canvas.set_draw_color(background_color);
            canvas.clear();
            canvas.set_draw_color(block_color);
            for j in 0..32 {
                for i in 0..64 {
                    if chip.gfx[i + (j * 64)] != 0 {
                        canvas
                            .fill_rect(Rect::new(
                                (i * window_width as usize / 64) as i32, // - block_size as i32,
                                (j * window_height as usize / 32) as i32, //+ block_size as i32,
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
}
