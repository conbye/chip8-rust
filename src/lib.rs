use rand::Rng;
use std::cmp;

const BYTE_SIZE: usize = 8;

const MEM_SIZE_BYTES: usize = 4096;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

const NUM_REGISTERS: usize = 16;
const VF_IDX: usize = 15;

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
			($instr >> bits) as u8
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
	delay_timer: u8,
	sound_timer: u8,
	r: [u8; NUM_REGISTERS],
	stack: Vec<u16>,
	stack_limit: usize
}

enum Instruction {
	CLEAR_OR_END_SUBROUTINE, // 0x0
	JUMP, // 0x1
	INIT_SUBROUTINE, // 0x2
	SKIP_VAL_EQ, // 0x3
	SKIP_VAL_NEQ, // 0x4
	SKIP_REG_EQ, // 0x5
	SET, // 0x6
	ADD, // 0x7
	ARITHMETIC, // 0x8
	SKIP_REG_NEQ, // 0x9
	SET_INDEX, // 0xA
	JUMP_OFF, // 0xB
	RAND, // 0xC
	DISPLAY, // 0xD
	SKIP_IF_KEY, // 0xE
	OTHER // 0xF
}

impl Emulator {
	fn init() -> Self {
		Self {
			pc: INIT_PC,
			ram: [0; MEM_SIZE_BYTES],
			display: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
			ir: INIT_IR,
			delay_timer: INIT_DELAY,
			sound_timer: INIT_SOUND,
			r: [0; NUM_REGISTERS],
			stack: Vec::new(),
			stack_limit: INIT_LIMIT,
		}
	}

	fn fetch(&mut self) -> u16 {
		let pc_addr = self.pc as usize;
		self.pc += 1;
		((self.ram[pc_addr] as u16) << 8) | (self.ram[pc_addr + 1] as u16)
	}

	fn decode_and_execute(&mut self, instr: u16) {
		let start: Instruction = extract_nibble!(instr, 3);

		match start {
			Instruction::CLEAR_OR_END_SUBROUTINE => {
				let val = extract_val!(instr, 0);
				if val == 0xEE {
					self.end_subroutiune();
				} else if val == 0xE0 {
					self.clear_screen();
				} else {
					panic!("Invalid Command! {instr}");
				}
			}
			Instruction::JUMP => {
				let addr = extract_addr!(instr);
				self.jump(addr);
			}
			Instruction::INIT_SUBROUTINE => {
				let addr = extract_addr!(instr);
				self.init_subroutine(addr);
			}
			Instruction::SKIP_VAL_EQ => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.skip_cond_n(true, x, val);
			}
			Instruction::SKIP_VAL_NEQ => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.skip_cond_n(false, x, val);
			}
			Instruction::SKIP_REG_EQ => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.skip_cond_n(true, x, val);
			}
			Instruction::SKIP_REG_NEQ => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.skip_cond_n(false, x, val);
			}
			Instruction::SET => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.set(x, val);
			}
			Instruction::ADD => {
				let x = extract_nibble!(instr, 2) as usize;
				let val = extract_val!(instr, 0);
				self.add(x, val);
			}
			Instruction::ARITHMETIC => {
				let x = extract_nibble!(instr, 2) as usize;
				let y = extract_nibble!(instr, 1) as usize;
				let arith_type = extract_nibble!(instr, 0);
			}
			Instruction::SET_INDEX => {
				let addr = extract_addr!(instr);
				self.set_index_register(addr);
			}
			Instruction::JUMP_OFF => {
				let addr = extract_addr!(instr);
				self.jump_with_offset(addr);
			}
			Instruction::RAND => {
				let x = extract_nibble!(instr, 2) as usize;
				let mask = extract_val!(instr, 0);
				self.rand(x, mask);
			}
			Instruction::DISPLAY => {

			}
			Instruction::SKIP_IF_KEY => {

			}
			Instruction::OTHER => {

			}
		}
	}

	fn clear_screen(&mut self) {
		for row in self.display {
			for mut pixel in row { 
				pixel = false; 
			}
		}
	}

	fn jump(&mut self, addr: u16) {
		self.pc = addr;
	}

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

	fn skip_cond_n(&mut self, eq: bool, x: usize, val: u8) {
		if eq && (self.r[x] == val) {
			self.pc += 2;
		} else if !eq && (self.r[x] != val) {
			self.pc += 2;
		}

	}

	fn skip_cond_r(&mut self, eq: bool, x: usize, y: usize) {
		if eq && (self.r[x] == self.r[y]) {
			self.pc += 2;
		} else if !eq && (self.r[x] != self.r[y]) {
			self.pc += 2;
		}
	}

	fn set(&mut self, x: usize, val: u8) {
		self.r[x] = val;
	}

	fn add(&mut self, x: usize, val: u8) {
		self.r[x] += val;
	}

	fn arith_set(&mut self, x: usize, y: usize) {
		self.r[x] = self.r[y];
	}

	fn set_index_register(&mut self, addr: u16) {
		self.ir = addr;
	}

	fn jump_with_offset(&mut self, addr: u16) {
		self.pc = addr + (self.r[0] as u16);
	}

	fn rand(&mut self, x: usize, mask: u8) {
		let mut rng = rand::rng();
		let mut num = rng.gen_range(0..256);
		self.r[x] = mask & (num as u8);
	}

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
}