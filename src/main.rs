// Standard Libraries
use std::io;
use std::fs;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;

// Special libraries needed for this project
use chip8_rust::*;
use sdl2::event::Event;

const FONT_PATH: &str = "./chip48font.txt";
const FONT_HEIGHT: usize = 5;
const KEYBOARD_SIZE: usize = 16;
const DEFAULT_HZ: u64 = 60;


const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn main() {

	let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("CHIP-8 Emulator (Rust)", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

	let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();


	println!("Initializing emulator...");
	let mut emu = Emulator::init();

	println!("Enter Location of Font to Load (Default if not given): ");
	let mut path = String::from(FONT_PATH);
	io::stdin().read_line(&mut path)
		.expect("failed to read line");
	
	let font: Vec<u8> = parse_font(path.as_str());
	emu.load_font(&font);

	println!("Initializing timer...");
	let tick_length = Duration::from_millis(1000 / DEFAULT_HZ);

	thread::scope(|s| {

		let (d_timer, s_timer) = emu.extract_timers();
		let handler = s.spawn(move || loop {
			thread::sleep(tick_length); 
			let mut atomic_delay_timer = d_timer.lock().unwrap();
			let mut atomic_sound_timer = s_timer.lock().unwrap();

			if *atomic_delay_timer > 0 {
				*atomic_delay_timer -= 1;
			}

			if *atomic_sound_timer > 0 {
				*atomic_sound_timer -= 1;
				// println!('\x07');
			}
		});
		loop {
			emu.run_next_instr();
		}
	});
}

fn parse_font(font_path: &str) -> Vec<u8> {

	let mut font: Vec<u8> = Vec::new();
	font.reserve(KEYBOARD_SIZE * FONT_HEIGHT);

	let font_str = fs::read_to_string(FONT_PATH)
		.expect("Can't find our font at path: {font_path}!");

	let font_lines = font_str.lines();
	

	for (i, line) in font_lines.enumerate() {
		if i % (FONT_HEIGHT + 1) == 0 {
			continue;
		}
		let mut byte: u8 = 0;
		let line_str: String = String::from(line);
		for (place, chr) in line_str.chars().enumerate() {
			let bit = chr as u8;
			byte &= bit << place;
		}
		font.push(byte);
	}
	font
}

fn draw_screen(emu: &mut Emulator) {

}