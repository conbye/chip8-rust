// Standard Libraries
use std::io;
use std::fs;
use std::time::Duration;
use std::sync::mpsc;
use std::thread;

// Special libraries needed for this project
use chip8_rust::*;

mod window;

const FONT_PATH: &str = "./chip48font.txt";
const FONT_HEIGHT: usize = 5;
const KEYBOARD_SIZE: usize = 16;
const DEFAULT_HZ: u64 = 60;
const INT_ASCII: u8 = 0x50;


const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCALE: u32 = 15;


fn main() {

	println!("Initializing emulator...");
	let mut emu = Emulator::init();

	println!("Enter Location of Font to Load (Default if not given): ");
	let mut path = String::new();
	io::stdin().read_line(&mut path)
		.expect("failed to read line");

	let font: Vec<u8>;
	if path.trim().is_empty()  {
		font = parse_font(FONT_PATH);
	} else  {
		font = parse_font(path.as_str());
	}
	emu.load_font(&font);

	println!("Initializing timer...");
	let tick_length = Duration::from_millis(1000 / DEFAULT_HZ);

	let mut sdl_handler = window::SdlHandler::init(SCREEN_WIDTH, SCREEN_HEIGHT, SCALE);

	sdl_handler.display_buff(&emu);

	thread::scope(|s| {

		let (tx, rx) = mpsc::channel();

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
			let exit = rx.recv().unwrap();
			if exit { break; }
		});
		'comp_and_display: loop {
			// Poll events and check for an exit command
			let exit = sdl_handler.poll_events(&mut emu);

			sdl_handler.display_buff(&emu);

			tx.send(exit).unwrap();
			if exit { break 'comp_and_display; }
		}
		handler.join();
	});
}

fn parse_font(font_path: &str) -> Vec<u8> {

	let mut font: Vec<u8> = Vec::new();
	font.reserve(KEYBOARD_SIZE * FONT_HEIGHT);

	let font_str = fs::read_to_string(font_path)
		.expect("Can't find our font at path!");

	let font_lines = font_str.lines();
	

	for (i, line) in font_lines.enumerate() {
		if i % (FONT_HEIGHT + 1) == 0 {
			continue;
		}
		let line_str: String = String::from(line);
		let byte: u8 = isize::from_str_radix(&line_str[2..], 2).unwrap() as u8;
		font.push(byte);
	}
	font
}