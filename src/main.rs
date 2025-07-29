use std::io;
use std::fs;
use std::time::{Duration, Instant};
use std::thread;
use chip8_rust::Emulator;

const FONT_PATH: &str = "./chip48font.txt";
const FONT_HEIGHT: usize = 5;
const KEYBOARD_SIZE: usize = 16;
const DEFAULT_HZ: u64 = 60;

fn main() {
	println!("Initializing emulator...");
	let mut emu = Emulator::init();

	println!("Enter Location of Font to Load (Default if not given): ");
	let mut path = String::from(FONT_PATH);
	io::stdin().read_line(&mut path)
		.expect("failed to read line");
	
	let font: Vec<u8> = parse_font(path.as_str());
	emu.load_font(&font);

	println!("Initializing timer...");
	let time_between_ticks = Duration::from_millis(1000 / DEFAULT_HZ);

	let handler = thread::spawn(|| {
		loop {
			thread::sleep(time_between_ticks); 
			emu.update_timer();
		}
	});
	
	loop {
		let instr = emu.fetch();
		emu.decode_and_execute(instr);
	};
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