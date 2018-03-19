extern crate env_logger;
#[macro_use]
extern crate log;

extern crate sdl2;

use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::iter::FromIterator;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::audio::{AudioCallback, AudioSpecDesired};

mod chip8;

struct SoundWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SoundWave {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.55 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn main() {
    let rom_path = match std::env::args().nth(1) {
        Some(path) => path,
        None => {
            println!("PLEASE GIVE ME A ROM!!");
            return;
        }
    };

    let mut f = File::open(rom_path).unwrap();

    let mut buf = Vec::new();

    f.read_to_end(&mut buf).unwrap();

    env_logger::init();

    let mut chip = chip8::new();

    chip.toogle_quirks();

    chip.load_rom(&buf);

    let window_width = 64 * 15;
    let window_height = 32 * 15;
    let block_size = 15u32;

    // Facebook colors, lol
    let background_color = Color::RGB(59, 89, 152);
    let block_color = Color::RGB(247, 247, 247);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("chip8 interpreter", window_width, window_height)
        .position_centered()
        .build()
        .unwrap();

    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(2),
        samples: None,
    };
    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| SoundWave {
            phase_inc: 60.0 / spec.freq as f32,
            phase: 0.5,
            volume: 0.10,
        })
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(background_color);
    canvas.clear();

    let timers_tickrate =
        std::time::Duration::from_millis(f64::floor((1.0 / 60.0) * 1000.0) as u64); // 60hz

    let cpu_tickrate = std::time::Duration::from_millis(f64::floor((1.0 / 500.0) * 1000.0) as u64);

    let mut timers_past = std::time::Instant::now();
    let mut cpu_past = std::time::Instant::now();

    let keys: HashMap<Keycode, u8> = HashMap::from_iter(vec![
        (Keycode::Num1, 0x1),
        (Keycode::Num2, 0x2),
        (Keycode::Num3, 0x3),
        (Keycode::Num4, 0xC),
        (Keycode::Q, 0x4),
        (Keycode::W, 0x5),
        (Keycode::E, 0x6),
        (Keycode::R, 0xD),
        (Keycode::A, 0x7),
        (Keycode::S, 0x8),
        (Keycode::D, 0x9),
        (Keycode::F, 0xE),
        (Keycode::Z, 0xA),
        (Keycode::X, 0x0),
        (Keycode::C, 0xB),
        (Keycode::V, 0xF),
    ]);

    'game_loop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if key == Keycode::Escape {
                        break 'game_loop;
                    }
                    if keys.contains_key(&key) {
                        chip.key_down(keys[&key])
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if keys.contains_key(&key) {
                        chip.key_up(keys[&key])
                    }
                }
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

        if chip.sound_flag {
            device.resume();
        } else {
            device.pause();
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
                                (i * window_width as usize / 64) as i32,
                                (j * window_height as usize / 32) as i32,
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
