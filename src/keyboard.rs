use std::collections::HashMap;
use getch_rs::{Getch, Key};

pub struct Keyboard {
	key_map: HashMap<char, char>, 
	exit_key: Key
}

impl Keyboard {
	pub fn init(exit: Key) -> Self {
		Self {
			key_map: HashMap::from([
					('1', '1'), ('2', '2'), ('3', '3'), ('4', 'C'),
					('q', '4'), ('w', '5'), ('e', '6'), ('r', 'D'),
					('a', '7'), ('s', '8'), ('d', '9'), ('f', 'E'),
					('z', 'A'), ('x', '0'), ('c', 'B'), ('v', 'D'),
				]),
			exit_key: exit,
		}
	}

	pub fn interpret_keystroke(&self) -> char {
		let g = Getch::new();
		match g.getch() {
			Ok(Key::Char(c)) => {
				if !self.key_map.contains_key(&c) {
					return self.key_map[&c];
				}
				return '\0';
			}
			Ok(Key::Esc) => {
				return "!";
			}
			Ok(key) => {
				return '\0';
			}
			Err(e) => {
				panic!("{}", e);
			}
		}
	}
}