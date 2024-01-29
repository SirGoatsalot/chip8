use std::env;
use std::fs::File;
use std::io::Read;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use chip8_core::*;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Correct usage is cargo run path/to/file");
        return;
    }

    // Set-up SDL
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = Emu::new();
    let mut rom = File::open(&args[1]).expect("Unable to open file.");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit{..} => {
                    break 'gameloop;
                },
                Event::KeyDown { timestamp: _, window_id: _, keycode, scancode: _, keymod: _, repeat: _ } => {
                    match keycode {
                        Some(code) => set_key(&mut chip8, code, true),
                        None => ()
                    }
                },
                Event::KeyUp { timestamp: _, window_id: _, keycode, scancode: _, keymod: _, repeat: _ } => {
                    match keycode {
                        Some(code) => set_key(&mut chip8, code, false),
                        None => ()
                    }
                }
                _ => ()
            }
        }
        
        chip8.tick();
        draw_screen(&mut chip8, &mut canvas);
    }
}

fn draw_screen(emu: &mut chip8_core::Emu, canvas: &mut Canvas<Window>) {
    // Clear canvas as black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();
    // Set draw color to white, draw pixel if it needs to be drawn
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
        // Convert our 1D array's index into a 2D (x,y) position
        let x = (i % SCREEN_WIDTH) as u32;
        let y = (i / SCREEN_WIDTH) as u32;
        // Draw a rectangle at (x,y), scaled up by our SCALE value
        let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
        canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}

fn set_key(emu: &mut chip8_core::Emu, keycode: Keycode, pressed: bool) {
    match keycode {
        Keycode::Kp0 => emu.keypress(0x0, pressed),
        Keycode::Kp1 => emu.keypress(0x1, pressed),
        Keycode::Kp2 => emu.keypress(0x2, pressed),
        Keycode::Kp3 => emu.keypress(0x3, pressed),
        Keycode::Kp4 => emu.keypress(0x4, pressed),
        Keycode::Kp5 => emu.keypress(0x5, pressed),
        Keycode::Kp6 => emu.keypress(0x6, pressed),
        Keycode::Kp7 => emu.keypress(0x7, pressed),
        Keycode::Kp8 => emu.keypress(0x8, pressed),
        Keycode::Kp9 => emu.keypress(0x9, pressed),
        Keycode::KpDivide => emu.keypress(0xA, pressed),
        Keycode::KpMultiply => emu.keypress(0xB, pressed),
        Keycode::KpMinus => emu.keypress(0xC, pressed),
        Keycode::KpPlus => emu.keypress(0xD, pressed),
        Keycode::KpEnter => emu.keypress(0xE, pressed),
        Keycode::KpPeriod => emu.keypress(0xC, pressed),
        _ => println!("Invalid Keypre!: {}", keycode)
    }
}