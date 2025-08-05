use std::cmp;
use std::sync::{Arc, Mutex};
use::std::fmt;

use rand::Rng;
use sdl3::keyboard::Keycode;
use crate::keyboard::Keyboard;

mod keyboard;

const BYTE_SIZE: usize = 8;

const MEM_SIZE_BYTES: usize = 512 * BYTE_SIZE;
const MAX_BYTE: u16 = 0xFF;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

const NUM_REGISTERS: usize = 16;
const VF_IDX: usize = 15;


const INIT_PC: u16 = 0;
const INIT_IR: u16 = 0x0000;
const INIT_FONT_ADDRESS: u16 = 0x0050;
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


pub struct Emulator {
	pc: u16,
	ram: [u8; MEM_SIZE_BYTES],
	display: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
	ir: u16,
	font_loc: u16,
	delay_timer: Arc<Mutex<u8>>,
	sound_timer: Arc<Mutex<u8>>,
	r: [u8; NUM_REGISTERS],
	stack: Vec<u16>,
	stack_limit: usize,
	keyboard: Keyboard,
}

impl fmt::Debug for Emulator {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Chip8 Emulator State -> \n\
			PC: 0x{:X}\n\
			INSTRUCTION: 0x{:X}\n\
			REGISTERS: \n\
				V0: {}, V1: {}, V2: {}, V3: {}, V4: {}, V5: {}, V6: {}, V7: {}, \n\
				V8: {}, V9: {}, VA: {}, VB: {}, VC: {}, VD: {}, VE: {}, VF: {}",
		self.pc,
	   {
		   let pc_addr = self.pc as usize;
		   ((self.ram[pc_addr] as u16) << BYTE_SIZE) | (self.ram[pc_addr + 1] as u16)
	   },
		self.r[0], self.r[1], self.r[2], self.r[3], self.r[4], self.r[5], self.r[6], self.r[7],
	    self.r[8], self.r[9], self.r[10], self.r[11], self.r[12], self.r[13], self.r[14], self.r[15])
	}
}

impl Emulator {
	pub fn init() -> Self {
		Self {
			pc: INIT_PC,
			ram: [0; MEM_SIZE_BYTES],
			display: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
			ir: INIT_IR,
			font_loc: INIT_FONT_ADDRESS,
			delay_timer: Arc::new(Mutex::new(INIT_DELAY)),
			sound_timer: Arc::new(Mutex::new(INIT_SOUND)),
			r: [0; NUM_REGISTERS],
			stack: Vec::new(),
			stack_limit: INIT_LIMIT,
			keyboard: Keyboard::init(Keycode::Escape),
		}
	}
	pub fn print_registers(&self) {
		println!("REGISTERS:");
		for i in 0..NUM_REGISTERS {
			println!("   V{}: 0x{:X}", i, self.r[i]);
		}
	}

	pub fn load_extern_data(& mut self, data: &Vec<u8>, addr: usize, is_rom: bool) {
		for i in 0..data.len() {
			self.ram[addr + i] = data[i];
		}
		if is_rom { self.pc = addr as u16; }
	}

	pub fn extract_timers(&mut self) -> (Arc<Mutex<u8>>, Arc<Mutex<u8>>) {
		(self.delay_timer.clone(), self.sound_timer.clone())
	}

	pub fn update_keystroke(&mut self, key: &Keycode) {
		self.keyboard.interpret_keystroke(key);
	}

	pub fn get_display(&self) -> &[[bool; SCREEN_WIDTH]; SCREEN_HEIGHT] {
		&self.display
	}

	pub fn run_next_instr(&mut self) {
		let instr = self.fetch();
		self.decode_and_execute(instr);
	}

	fn fetch(&mut self) -> u16 {
		let pc_addr = self.pc as usize;
		self.pc += 2;
		((self.ram[pc_addr] as u16) << BYTE_SIZE) | (self.ram[pc_addr + 1] as u16)
	}

	fn decode_and_execute(&mut self, instr: u16) {
		let start_key = extract_nibble!(instr, 3);

		match start_key {
			0x0 => {
				let val = extract_val!(instr, 0);
				if val == 0xEE {
					self.end_subroutine();
				} else if val == 0xE0 {
					self.clear_screen();
				} else {
					self.print_registers();
;					panic!("Invalid Command! 0x{:X}", instr);
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
				let y = extract_val!(instr, 1) as usize;
				self.skip_cond_r(true, x, y);
			}
			0x9 => {
				let x = extract_nibble!(instr, 2) as usize;
				let y = extract_val!(instr, 1) as usize;
				self.skip_cond_r(false, x, y);
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

					0x5 => { self.arith_sub(x, y); }
					0x7 => { self. arith_sub(y, x); }

					0x6 => { self.arith_shift_r(x, y); }
					0xE => { self.arith_shift_l(x, y); }

					_ => {
						self.print_registers();
						panic!("Invalid Arithmetic Instruction.");
					}
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
					0x07 => { self.get_delay_timer(x); }
					0x15 => { self.set_delay_timer(x); }
					0x18 => { self.set_sound_timer(x); }

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
			k => { panic!("Invalid Start Key {k} for instruction 0x{:X}", instr); }
		}
	}

	// 0x0
	fn clear_screen(&mut self) {
		for row in self.display.iter_mut() {
			for pixel in row { 
				*pixel = false; 
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
		self.jump(addr);
	}
	
	fn end_subroutine(&mut self) {
		let last_addr = self.stack.pop();
		if last_addr != None {
			self.jump(last_addr.unwrap());
		} else {
			panic!("ERROR: Returned from subroutine to empty stack");
		}
	}

	// 0x4
	fn skip_cond_n(&mut self, eq: bool, x: usize, val: u8) {
		let cond = if eq { self.r[x] == val } else { self.r[x] != val };

		if cond {self.pc += 2; }

	}

	// 0x5
	fn skip_cond_r(&mut self, eq: bool, x: usize, y: usize) {
		let cond = if eq { self.r[x] == self.r[y] } else { self.r[x] != self.r[y] };

		if cond {self.pc += 2; }
	}

	// 0x6
	fn set(&mut self, x: usize, val: u8) {
		self.r[x] = val;
	}

	// 0x7
	fn add(&mut self, x: usize, val: u8) {
		let sum = self.r[x] as u16 + val as u16;
		self.r[x] = (sum & MAX_BYTE) as u8;
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
		let sum: u16 = self.r[x] as u16 + self.r[y] as u16;
		self.r[VF_IDX] = if sum > MAX_BYTE {1} else {0};
		self.r[x] = (sum & MAX_BYTE) as u8;
	}

	fn arith_sub(&mut self, x: usize, y: usize) {
		if self.r[x] < self.r[y] {
			self.r[x] = self.r[y] - self.r[x];
			self.r[VF_IDX] = 0;
		} else {
			self.r[x] = self.r[x] - self.r[y];
			self.r[VF_IDX] = 1;
		}
	}

	fn arith_shift_r(&mut self, x: usize, y: usize) {
		self.r[VF_IDX] = if (self.r[x]) & 0x01 > 0 {1} else {0};
		self.r[x] = self.r[x] >> 0x1;
	}

	fn arith_shift_l(&mut self, x: usize, y: usize) {
		self.r[VF_IDX] = if (self.r[x]) & 0x80 > 0 {1} else {0};
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
		let mut rng = rand::rng();
		let num = rng.random_range(0..=(MAX_BYTE as i32));
		self.r[x] = mask & (num as u8);
	}
	
	// 0xD
	fn display(&mut self, x: usize, y: usize, sprite_height: u8) {

		// Calculate sprite size and placement
		let sprite_height = sprite_height as usize;
		let x_coord = (self.r[x] % SCREEN_WIDTH as u8) as usize;
		let y_coord = (self.r[y] % SCREEN_HEIGHT as u8) as usize;
		let sprite_height = cmp::min(sprite_height, SCREEN_HEIGHT - y_coord);
		let sprite_len = cmp::min(BYTE_SIZE, SCREEN_WIDTH - x_coord);

		// Access needed portion of sprite in memory
		let idx = self.ir as usize;
		let sprite = &self.ram[idx..idx + sprite_height];

		// Update display according to sprite bits
		for i in (0..sprite_height) {
			let row = sprite[i];
			let x_end = x_coord + sprite_len - 1;
			for bit in (0..sprite_len).rev() {
				if ((row >> bit) & 0x1) != 0 {
					self.display[y_coord + i][x_end - bit] = !self.display[y_coord + i][x_end - bit];
					if !self.display[y_coord + i][x_end - bit] {
						self.r[VF_IDX] = 0x1;
					}
				}
			}
		}
	}

	// 0xE
	fn skip_if_key(&mut self, x: usize, pressed: bool) {
		let chr = self.r[x] as char;

		let keystroke = self.keyboard.get_cur_key();

		let skip = if pressed {keystroke == chr} else {keystroke != chr};

		if skip {
			self.pc += 2;
		}
	}

	// 0xF
	fn get_delay_timer(&mut self, x: usize) {
		let atomic_delay_timer = self.delay_timer.lock().unwrap();
		self.r[x] = *atomic_delay_timer;
	}

	fn set_delay_timer(&mut self, x: usize) {
		let mut atomic_delay_timer = self.delay_timer.lock().unwrap();
		*atomic_delay_timer = self.r[x];
	}

	fn set_sound_timer(&mut self, x: usize) {
		let mut atomic_sound_timer = self.delay_timer.lock().unwrap();
		*atomic_sound_timer = self.r[x];
	}

	fn add_to_idx(&mut self, x: usize) {
		self.ir = self.r[x] as u16;
	}

	fn get_key(&mut self, x: usize) {

		let keystroke = self.keyboard.get_cur_key();
		
		if keystroke == '\0' {
			self.pc -= 2;
		} else {
			self.r[x] = keystroke as u8;
		}
	}

	fn font_char(&mut self, x: usize) {
		println!("using font char");
		let font_addr = self.font_loc;
		let mut chr_code = self.r[x] as u16;
		if chr_code >= 'A' as u16 {
			chr_code = chr_code - 0x7;
		}
		self.ir = font_addr + chr_code;
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
		for xi in 0..=x {
			self.ram[ir + xi] = self.r[xi];
		}
	}

	fn load(&mut self, x: usize) {
		let ir = self.ir as usize;
		for xi in 0..=x {
			self.r[xi] = self.ram[ir + xi];
		}
	}
}