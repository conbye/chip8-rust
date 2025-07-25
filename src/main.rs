use std::io;
use std::fs;
use std::time;

mod keyboard;

const FONT_PATH: str = "./chip48font.txt";
const FONT_HEIGHT: usize = 5;
const KEYBOARD_SIZE: usize = 16;

fn main() {
	println!("Initializing emulator...");
	let mut emu = Emulator::init();

	println!("Loading font...");
	let font_text = fs::read_to_string(FONT_PATH)
		.expect("Can't find our font!");

	loop {
		let (delay_up, sound_up) = emu.update_timer();
		emu.fetch();
		emu.decode_and_execute();
	};
	println!("Hello World!");
}

fn parse_font(font_path: &str) -> Vec<u8> {
	let font: Vec<u8> = Vec::new();
	font.reserve(KEYBOARD_SIZE * FONT_HEIGHT);
	let font_lines = fs::read_to_string(FONT_PATH)
		.unwrap()
		.lines();
	for (i, line) in font_lines.enumerate() {
		if i % (FONT_HEIGHT + 1) == 0 {
			continue;
		}
		let mut byte: u8 = 0;
		for (place, chr) in line.enumerate() {
			let bit = chr as u8;
			byte += (bit << place);
		}
		font.push(byte);
	}
	font
}

fn boot_emulator(&mut emu) {

}