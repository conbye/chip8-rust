use std::collections::HashMap;
use sdl3::keyboard::Keycode;

pub struct Keyboard {
	key_map: HashMap<Keycode, u8>,
	cur_key: u8,
	exit_key: Keycode
}

impl Keyboard {
	pub fn init(exit: Keycode) -> Self {
		Self {
			key_map: HashMap::from([
				(Keycode::_1, 0x1), (Keycode::_2, 0x2), (Keycode::_3, 0x3), (Keycode::_4, 0xF),
				( Keycode::Q, 0x4), ( Keycode::W, 0x5), ( Keycode::E, 0x6), ( Keycode::R, 0xE),
				( Keycode::A, 0x7), ( Keycode::S, 0x8), ( Keycode::D, 0x9), ( Keycode::F, 0xD),
				( Keycode::Z, 0xA), ( Keycode::X, 0x0), ( Keycode::C, 0xB), ( Keycode::V, 0xC),
			]),
			cur_key: 0x0,
			exit_key: exit,
		}
	}

	pub fn interpret_keystroke(&mut self, key: &Keycode) {
		// self.cur_key = 0x0;
		if self.key_map.contains_key(key) {
			self.cur_key = self.key_map[key];
		} else if *key == self.exit_key {
			self.cur_key = 0x1F;
		}
	}

	pub fn get_cur_key(&mut self) -> u8 {
		let temp = self.cur_key;
		self.cur_key = 0x0;
		temp
	}
}