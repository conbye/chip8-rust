use chip8_rust::*;

// Simple Directmedia Library 3
extern crate sdl3;

use sdl3::pixels::Color;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::rect::Rect;
use sdl3::render::WindowCanvas;
use sdl3::Sdl;

pub struct SdlHandler {
    scale: u32,
    context: Sdl,
    canvas: WindowCanvas,
}

impl SdlHandler {
    pub fn init(w: usize, h: usize, s: u32) -> Self {
        let width = w as u32;
        let height = h as u32;
        let sdl_context = sdl3::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("CHIP-8 Emulator (Rust)", width * s, height * s)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas();
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();
        canvas.present();

        Self {
            scale: s,
            context: sdl_context,
            canvas,
        }
    }

    pub fn poll_events(&mut self, emu: &mut Emulator) -> bool {
        let mut event_pump = self.context.event_pump().unwrap();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => { return true; }
                Event::KeyDown{keycode, ..} => {
                    let key = keycode.unwrap();
                    emu.update_keystroke(&key);
                },
                _ => {}
            }
        }
        // dbg!(&emu);
        emu.run_next_instr();
        false
    }

    pub fn display_buff(&mut self, emu: &Emulator) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        let screen_buf = emu.get_display();
        // Now set draw color to white, iterate through each point and see if it should be drawn
        self.canvas.set_draw_color(Color::WHITE);
        for (i, row) in screen_buf.iter().enumerate() {
            for (j, pixel) in row.iter().enumerate() {
                if *pixel {
                    // Convert our i and j indices to u32 from usize
                    let x = j as u32;
                    let y = i as u32;

                    // Draw a rectangle at (x,y), scaled up by our SCALE value
                    let rect = Rect::new((x * self.scale) as i32, (y * self.scale) as i32, self.scale, self.scale);
                    self.canvas.fill_rect(rect).unwrap();
                }
            }
        }
        self.canvas.present();
    }
}