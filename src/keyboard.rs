use std::collections::HashMap;
use sdl3::keyboard;
use sdl3::keyboard::Keycode;

pub struct Keyboard {
	key_map: HashMap<Keycode, char>,
	cur_key: char,
	exit_key: Keycode
}

impl Keyboard {
	pub fn init(exit: Keycode) -> Self {
		Self {
			key_map: HashMap::from([
				(Keycode::_1, '1'), (Keycode::_2, '2'), (Keycode::_3, '3'), (Keycode::_4, 'G'),
				( Keycode::Q, '4'), ( Keycode::W, '5'), ( Keycode::E, '6'), ( Keycode::R, 'F'),
				( Keycode::A, '7'), ( Keycode::S, '8'), ( Keycode::D, '9'), ( Keycode::F, 'E'),
				( Keycode::Z, 'A'), ( Keycode::X, 'B'), ( Keycode::C, 'C'), ( Keycode::V, 'D'),
			]),
			cur_key: '\0',
			// key_map: HashMap::from([
			// 		('1', '1'), ('2', '2'), ('3', '3'), ('4', 'C'),
			// 		('q', '4'), ('w', '5'), ('e', '6'), ('r', 'D'),
			// 		('a', '7'), ('s', '8'), ('d', '9'), ('f', 'E'),
			// 		('z', 'A'), ('x', '0'), ('c', 'B'), ('v', 'D'),
			// 	]),
			exit_key: exit,
		}
	}

	pub fn interpret_keystroke(&mut self, key: &Keycode) {
		self.cur_key = '\0';
		if self.key_map.contains_key(key) {
			self.cur_key = self.key_map[key];
		} else if *key == self.exit_key {
			self.cur_key = '!';
		}
	}

	pub fn get_cur_key(&mut self) -> char {
		let temp = self.cur_key;
		self.cur_key = '\0';
		temp
	}
}