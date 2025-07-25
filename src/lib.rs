use rand::Rng;
use std::cmp;
use getch_rs::{Getch, Key};
use std::collections::HashMap;

mod keyboard;

const BYTE_SIZE: usize = 8;

const MEM_SIZE_BYTES: usize = 512 * BYTE_SIZE;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

const NUM_REGISTERS: usize = 16;
const VF_IDX: usize = 15;

const DEAFULT_FONT_LOC: u16 = 0x050;

const INIT_PC: u16 = 0;
const INIT_IR: u16 = 0x0000;
const INIT_DELAY: u8 = 60;
const INIT_SOUND: u8 = 60;
const INIT_LIMIT: usize = 16;


// Define Macros
//	-> For Parsing Instructions

macro_rules! extract_nibble {
	($instr: expr, $offset: expr) => {
		{
			let bits = 4 * $offset;
			(($instr >> bits) & 0x0F) as u8
		}
	}
}

macro_rules! extract_addr {
	($instr: expr) => {
		{
			$instr & 0xFFF
		}
	}
}

macro_rules! extract_val {
	($instr: expr, $offset: expr) => {
		{
			let bits = 4 * $offset;
			(($instr >> bits) & 0xFF) as u8
		}
	}
}

struct Emulator {
	pc: u16,
	ram: [u8; MEM_SIZE_BYTES],
	display: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
	ir: u16,
	font_loc: u16,
	delay_timer: u8,
	sound_timer: u8,
	r: [u8; NUM_REGISTERS],
	stack: Vec<u16>,
	stack_limit: usize,
	keyboard: keyboard::Keyboard,
}

impl Emulator {
	pub fn init() -> Self {
		Self {
			pc: INIT_PC,
			ram: [0; MEM_SIZE_BYTES],
			display: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
			ir: INIT_IR,
			font_loc: DEAFULT_FONT_LOC,
			delay_timer: INIT_DELAY,
			sound_timer: INIT_SOUND,
			r: [0; NUM_REGISTERS],
			stack: Vec::new(),
			stack_limit: INIT_LIMIT,
			keyboard: keyboard::Keyboard::init(Key::Esc),
		}
	}

	pub fn update_timer(&mut self) -> (bool, bool) {
		let delay_done = self.delay_timer <= 0;
		if !delay_done {
			self.delay_timer -= 1;
		}
		let sound_done = self.sound_timer <= 0;
		if !sound_done {
			self.sound_timer -= 1;
		}
		(delay_done, sound_done)
	}

	pub fn fetch(&mut self) -> u16 {
		let pc_addr = self.pc as usize;
		self.pc += 1;
		((self.ram[pc_addr] as u16) << BYTE_SIZE) | (self.ram[pc_addr + 1] as u16)
	}

	pub fn decode_and_execute(&mut self, instr: u16) {
		let start_key = extract_nibble!(instr, 3);

		match start_key {
			0x0 => {
				let val = extract_val!(instr, 0);
				if val == 0xEE {
					self.end_subroutiune();
				} else if val == 0xE0 {
					self.clear_screen();
				} else {
					panic!("Invalid Command! {instr}");
				}
			}
			0x1 => {
				let addr = extract_addr!(instr);
				self.jump(addr);
			}
			0x2 => {
				let addr = extract_addr!(instr);
				self.init_subroutine(addr);
			}
			0x3 => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.skip_cond_n(true, x, val);
			}
			0x4 => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.skip_cond_n(false, x, val);
			}
			0x5 => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.skip_cond_n(true, x, val);
			}
			0x9 => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.skip_cond_n(false, x, val);
			}
			0x6 => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.set(x, val);
			}
			0x7 => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.add(x, val);
			}
			0x8 => {
				let x = extract_nibble!(instr, 2) as usize;
				let y = extract_nibble!(instr, 1) as usize;
				let arith_type = extract_nibble!(instr, 0);
				match arith_type {
					0x0 => { self.arith_set(x, y); }

					0x1 => { self.arith_or(x, y); }
					0x2 => { self.arith_and(x, y); }
					0x3 => { self.arith_xor(x, y); }

					0x4 => { self.arith_add(x, y); }

					0x5 => { self.arith_sub_a(x, y); }
					0x7 => { self. arith_sub_b(x, y); }

					0x6 => { self.arith_shift_r(x, y); }
					0xE => { self.arith_shift_l(x, y); }

					_ => { panic!("Invalid Arithmetic Instruction."); }
				}
			}
			0xA => {
				let addr = extract_addr!(instr);
				self.set_index_register(addr);
			}
			0xB => {
				let addr = extract_addr!(instr);
				self.jump_with_offset(addr);
			}
			0xC => {
				let x = extract_nibble!(instr, 2) as usize;
				let mask = extract_val!(instr, 0);
				self.rand(x, mask);
			}
			0xD => {
				let x = extract_nibble!(instr, 2) as usize;
				let y = extract_nibble!(instr, 1) as usize;
				let height = extract_nibble!(instr, 0);
				self.display(x, y, height);
			}
			0xE => {
				let x = extract_nibble!(instr, 2) as usize;
				let label = extract_val!(instr, 0);
				let pressed = label == 0x9E;
				self.skip_if_key(x, pressed);
			}
			0xF => {
				let x = extract_nibble!(instr, 2) as usize;
				let end_key = extract_val!(instr, 0);
				match end_key {

					// Timers
					0x07 => { self.get_delay(x); }
					0x15 => { self.set_delay(x); }
					0x18 => { self.set_sound(x); }

					// Add to index
					0x1E => { self.add_to_idx(x); }
					0x0A => { self.get_key(x); }
					0x29 => { self.font_char(x); }
					0x33 => { self.dec_conversion(x); }

					// Loading and storing data
					0x55 => { self.store(x); }
					0x65 => { self.load(x); }

					// Key doesn't correlate to given intruction
					y => { panic!("Invalid other key: {y}"); }
				}
			}
			k => { panic!("Invalid Start Key {k}"); }
		}
	}

	// 0x0
	fn clear_screen(&mut self) {
		for row in self.display {
			for mut pixel in row { 
				pixel = false; 
			}
		}
	}

	// 0x1
	fn jump(&mut self, addr: u16) {
		self.pc = addr;
	}

	// 0x2
	fn init_subroutine(&mut self, addr: u16) {
		self.stack.push(self.pc);
		self.pc = addr;
	}
	
	fn end_subroutiune(&mut self) {
		let last_addr = self.stack.pop();
		if last_addr != None {
			self.pc = last_addr.unwrap();
		} else {
			panic!("ERROR: Returned from subroutine to empty stack");
		}
	}

	// 0x4
	fn skip_cond_n(&mut self, eq: bool, x: usize, val: u8) {
		if eq && (self.r[x] == val) {
			self.pc += 2;
		} else if !eq && (self.r[x] != val) {
			self.pc += 2;
		}

	}

	// 0x5
	fn skip_cond_r(&mut self, eq: bool, x: usize, y: usize) {
		if eq && (self.r[x] == self.r[y]) {
			self.pc += 2;
		} else if !eq && (self.r[x] != self.r[y]) {
			self.pc += 2;
		}
	}

	// 0x6
	fn set(&mut self, x: usize, val: u8) {
		self.r[x] = val;
	}

	// 0x7
	fn add(&mut self, x: usize, val: u8) {
		self.r[x] += val;
	}

	// 0x8
	fn arith_set(&mut self, x: usize, y: usize) {
		self.r[x] = self.r[y];
	}

	fn arith_or(&mut self, x:usize, y: usize) {
		self.r[x] = self.r[x] | self.r[y];
	}
	
	fn arith_and(&mut self, x:usize, y: usize) {
		self.r[x] = self.r[x] & self.r[y];
	}

	fn arith_xor(&mut self, x: usize, y: usize) {
		self.r[x] = self.r[x] ^ self.r[y];
	}

	fn arith_add(&mut self, x: usize, y: usize) {
		self.r[x] = self.r[x] + self.r[y];
	}

	fn arith_sub_a(&mut self, x: usize, y: usize) {
		self.r[x] = self.r[x] - self.r[y];
	}

	fn arith_sub_b(&mut self, x: usize, y: usize) {
		self.r[x] = self.r[y] - self.r[x];
	}

	fn arith_shift_r(&mut self, x: usize, y: usize) {
		if (self.r[x]) & 0x01 > 0 {
			self.r[VF_IDX] = 0x01;
		} else {
			self.r[VF_IDX] = 0x00;
		}
		self.r[x] = self.r[x] >> 0x1;
	}

	fn arith_shift_l(&mut self, x: usize, y: usize) {
		if (self.r[x]) & 0x80 > 0 {
			self.r[VF_IDX] = 0x01;
		} else {
			self.r[VF_IDX] = 0x00;
		}
		self.r[x] = self.r[x] << 0x1;
	}

	// 0xA
	fn set_index_register(&mut self, addr: u16) {
		self.ir = addr;
	}

	// 0xB
	fn jump_with_offset(&mut self, addr: u16) {
		self.pc = addr + (self.r[0] as u16);
	}

	// 0xC
	fn rand(&mut self, x: usize, mask: u8) {
		let mut rng = rand::thread_rng();
		let mut num = rng.gen_range(0..=255);
		self.r[x] = mask & (num as u8);
	}
	
	// 0xD
	fn display(&mut self, x: usize, y: usize, sprite_height: u8) {

		// Calculate sprite size and placement
		let sprite_height = sprite_height as usize;
		let x_coord = (self.r[x] % 64) as usize;
		let y_coord = (self.r[y] % 32) as usize;
		let sprite_height = cmp::min(sprite_height, 32 - (y_coord + sprite_height)) as usize;
		let sprite_len = cmp::min(BYTE_SIZE, 64 - x_coord) as usize;

		// Access needed portion of sprite in memory
		let idx = self.ir as usize;
		let sprite = &self.ram[idx..idx + sprite_height];

		// Update display according to sprite bits
		for (i, row) in sprite.iter().enumerate() {
			for bit in (0..sprite_len).rev() {
				if ((row >> bit) & 0x1) != 0 {
					self.display[x_coord + sprite_len - bit][y_coord + i] = !self.display[x_coord + bit][y_coord + i];
				}
				if !self.display[x_coord + bit][y_coord + i] {
					self.r[VF_IDX] = 0x1;
				}
			}
		}
	}

	// 0xE
	fn skip_if_key(&mut self, x: usize, pressed: bool) {
		let chr = self.r[x] as char;

		let keystroke = self.keyboard.interpret_keystroke();

		let mut skip = keystroke == chr;

		if !pressed {
			skip = !skip;
		}

		if skip {
			self.pc += 2;
		}
	}

	// 0xF
	fn get_delay(&mut self, x: usize) {
		self.r[x] = self.delay_timer;
	}

	fn set_delay(&mut self, x: usize) {
		self.delay_timer = self.r[x];
	}

	fn set_sound(&mut self, x: usize) {
		self.sound_timer = self.r[x];
	}

	fn add_to_idx(&mut self, x: usize) {
		self.ir = self.r[x] as u16;
	}

	fn get_key(&mut self, x: usize) {

		let keystroke = self.keyboard.interpret_keystroke();
		
		if keystroke == '\0' {
			self.pc -= 2;
		} else {
			self.r[x] = keystroke as u8;
		}
	}

	fn font_char(&mut self, x: usize) {
		let font_addr = self.font_loc as u16;
		let char_offset = self.r[x] as u16;
		self.pc = font_addr + char_offset;
	}

	fn dec_conversion(&mut self, x: usize) {
		let ir = self.ir as usize;
		let mut num = self.r[x];
		for i in (0..3).rev() {
			self.ram[ir + i] = num % 10;
			num /= 10;
		}
	}

	fn store(&mut self, x: usize) {
		let ir = self.ir as usize;
		for xi in 0..(x + 1) {
			self.ram[ir + xi] = self.r[xi];
		}
	}

	fn load(&mut self, x: usize) {
		let ir = self.ir as usize;
		for xi in 0..(x + 1) {
			self.r[xi] = self.ram[ir + xi];
		}
	}
}