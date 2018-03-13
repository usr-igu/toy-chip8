extern crate sdl2;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

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
            *x = if self.phase <= 0.5 {
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
        None => panic!("invalid ROM path"),
    };

    let mut f = File::open(rom_path).unwrap();

    let mut buf = Vec::new();

    f.read_to_end(&mut buf).unwrap();

    let mut chip = chip8::new();

    chip.load_rom(&buf);

    let window_width = 64 * 10;
    let window_height = 32 * 10;
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

    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(2),
        samples: None,
    };
    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| SoundWave {
            phase_inc: 50.0 / spec.freq as f32,
            phase: 0.5,
            volume: 0.15,
        })
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let timers_tickrate =
        std::time::Duration::from_millis(f64::floor((1.0 / 60.0) * 1000.0) as u64); // 60hz

    let cpu_tickrate = std::time::Duration::from_millis(f64::floor((1.0 / 500.0) * 1000.0) as u64);

    let mut timers_past = std::time::Instant::now();
    let mut cpu_past = std::time::Instant::now();

    let mut keys = HashMap::new();
    keys.insert(Keycode::Num1, 0x1);
    keys.insert(Keycode::Num2, 0x2);
    keys.insert(Keycode::Num3, 0x3);
    keys.insert(Keycode::Num4, 0xC);
    keys.insert(Keycode::Q, 0x4);
    keys.insert(Keycode::W, 0x5);
    keys.insert(Keycode::E, 0x6);
    keys.insert(Keycode::R, 0xD);
    keys.insert(Keycode::A, 0x7);
    keys.insert(Keycode::S, 0x8);
    keys.insert(Keycode::D, 0x9);
    keys.insert(Keycode::F, 0xE);
    keys.insert(Keycode::Z, 0xA);
    keys.insert(Keycode::X, 0x0);
    keys.insert(Keycode::C, 0xB);
    keys.insert(Keycode::V, 0xF);

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
