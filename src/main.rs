// Standard Libraries
use std::io;
use std::fs;
use std::io::Write;
use std::time::Duration;
use std::sync::mpsc;
use std::thread;

// Special libraries needed for this project
use chip8_rust::*;

mod window;

const FONT_PATH: &str = "./chip48font.txt";
const ROM_PATH: &str = "./test_games/c8games/";
const FONT_HEIGHT: usize = 5;
const KEYBOARD_SIZE: usize = 16;
const DEFAULT_TIMER_HZ: u64 = 60;
const INSTRUCTIONS_PER_SEC: u64 = 700;

const FONT_ADDRESS: usize = 0x0050;
const ROM_START_ADDRESS: usize = 0x0200;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCALE: u32 = 15;


fn main() {

	println!("Initializing emulator...");
	let mut emu = Emulator::init();

	print!("Enter Location of Font to Load (Default if not given): ");
	io::stdout().flush().expect("Failed to flush stdout");
	let mut font_path = String::new();
	io::stdin().read_line(&mut font_path)
		.expect("failed to read line");

	if font_path.trim().is_empty()  {
		load_font(FONT_PATH, &mut emu);
	} else  {
		load_font(font_path.trim(), &mut emu);
	}

	print!("Enter ROM you wish to run: ");
	io::stdout().flush().expect("Failed to flush stdout");
	let mut rom_path = String::from(ROM_PATH);
	io::stdin().read_line(&mut rom_path)
		.expect("failed to read line");
	load_rom(rom_path.trim(), &mut emu);

	println!("Initializing timer...");
	let tick_length = Duration::from_millis(1000 / DEFAULT_TIMER_HZ);
	let till_next_instr = Duration::from_millis(1000 / INSTRUCTIONS_PER_SEC);

	let mut sdl_handler = window::SdlHandler::init(SCREEN_WIDTH, SCREEN_HEIGHT, SCALE);

	sdl_handler.display_buff(&emu);

	thread::scope(|s| {

		let (tx, rx) = mpsc::channel();

		let (d_timer, s_timer) = emu.extract_timers();
		s.spawn(move || 'timer: loop {
			thread::sleep(tick_length); 
			let mut atomic_delay_timer = d_timer.lock().unwrap();
			let mut atomic_sound_timer = s_timer.lock().unwrap();

			if *atomic_delay_timer > 0 {
				*atomic_delay_timer -= 1;
			}

			if *atomic_sound_timer > 0 {
				*atomic_sound_timer -= 1;
				print!("{}", '\x07');
			}
			let exit = rx.recv().unwrap();
			if exit { break 'timer; }
		});
		'comp_and_display: loop {
			// Poll events and check for an exit command
			let exit = sdl_handler.poll_events(&mut emu);

			sdl_handler.display_buff(&emu);

			tx.send(exit).unwrap();
			if exit { break 'comp_and_display; }
			thread::sleep(till_next_instr);
		}
	});
}

fn load_font(font_path: &str, emu :&mut Emulator) {

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
	emu.load_extern_data(&font, FONT_ADDRESS, false);
}

pub fn load_rom(rom_path: &str, emu :&mut Emulator) {
	let rom_meta: fs::Metadata;
	match fs::metadata(rom_path) {
		Ok(meta) => {
			rom_meta = meta;
		}
		Err(e) => {
			panic!("Error: {e} | <{}>", rom_path);
		}
	}
	if !rom_meta.is_file() {
		panic!("<{}>  is not a file!", rom_path);
	}
	let rom_data = fs::read(rom_path).unwrap();
	emu.load_extern_data(&rom_data, ROM_START_ADDRESS, true);
}