extern crate termion;

use std::io::{stdin, stdout, Write};
use termion::clear;
use termion::cursor;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use std::cmp::{min, max};

#[derive(Debug, PartialEq, Clone)]
enum Token {
	Number(f64),
	Plus,
	Minus,
	Asterisk,
	Slash,
	LParen,
	RParen,
}

struct Lexer {
	input: Vec<char>,
	position: usize,
}

impl Lexer {
	fn new(input: Vec<char>) -> Lexer {
		Lexer { input, position: 0 }
	}

	fn token(&mut self) -> Option<Token> {
		use std::iter::FromIterator;
		while self.curr().is_some() && self.curr().unwrap().is_whitespace() {
			self.next();
		}

		let curr = self.curr()?;
		let token = if Self::is_number(curr) {
			let mut number = vec![*curr];
			while self.peek().is_some() && Self::is_number(self.peek().unwrap()) {
				self.next();
				number.push(*self.curr().unwrap());
			}
			String::from_iter(number)
				.parse::<f64>()
				.ok()
				.and_then(|n| Some(Token::Number(n)))
		} else {
			match curr {
				&'+' => Some(Token::Plus),
				&'-' => Some(Token::Minus),
				&'*' => Some(Token::Asterisk),
				&'/' => Some(Token::Slash),
				&'(' => Some(Token::LParen),
				&')' => Some(Token::RParen),
				_ => None,
			}
		};
		self.next();
		return token;
	}
	fn next(&mut self) {
		self.position += 1;
	}
	fn curr(&mut self) -> Option<&char> {
		self.input.get(self.position)
	}
	fn peek(&mut self) -> Option<&char> {
		self.input.get(self.position + 1)
	}
	fn is_number(c: &char) -> bool {
		c.is_ascii_digit() || c == &'.'
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cursor {
	row: usize,
	column: usize,
}

struct Dentaku {
	buffer: Vec<Vec<char>>,
	cursor: Cursor,
	row_offset: usize,
}

impl Default for Dentaku {
	fn default() -> Self {
		Self{
			buffer: vec![Vec::new()],
			cursor: Cursor { row: 0, column: 0 },
			row_offset: 0,
		}
	}
}

impl Dentaku {
	fn terminal_size() -> (usize, usize) {
		let (cols, rows) = termion::terminal_size().unwrap();
		(rows as usize, cols as usize)
	}
	fn draw<T: Write>(&self, out: &mut T) {
		let (rows, cols) = Self::terminal_size();

		write!(out, "{}", clear::All);
		write!(out, "{}", cursor::Goto(1, 1));

		let mut row = 0;
		let mut col = 0;

		let mut cursor: Option<Cursor> = None;

		'outer: for i in self.row_offset..self.buffer.len() {
			for j in 0..=self.buffer[i].len() {
				if self.cursor == (Cursor { row: i, column: j }) {
					cursor = Some(Cursor {
						row: row,
						column: col,
					});
				}

				if let Some(c) = self.buffer[i].get(j) {
					write!(out, "{}", c);
					col += 1;
					if col >= cols {
						row += 1;
						col = 0;
						if row >= rows {
							break 'outer;
						} else {
							write!(out, "\r\n");
						}
					}
				}
			}
			row += 1;
			col = 0;
			if row >= rows {
				break;
			} else {
				write!(out, "\r\n");
			}
		}

		if let Some(cursor) = cursor {
			write!(out, "{}", cursor::Goto(self.cursor.column  as u16 + 1, self.cursor.row as u16 + 1));
		}
		out.flush().unwrap();
	}
	fn scroll(&mut self) {
		let (rows, _) = Self::terminal_size();
		self.row_offset = min(self.row_offset, self.cursor.row);
		if self.cursor.row + 1 >= rows {
			self.row_offset = max(self.row_offset, self.cursor.row + 1 - rows);
		}
	}
	fn insert(&mut self, c: char) {
		if c == '\n' {
			//// 構文解析器
			//let mut command = Lexer::new(self.buffer[self.cursor.row].to_vec());
			//self.buffer.insert(self.cursor.row + 1, command.token().ok().chars().collect());
			//self.buffer.insert(self.cursor.row + 2, "".chars().collect());
			//self.cursor.column = 0;
			//self.cursor.row += 2;
			//// 構文解析器fin

			let command = &self.buffer[self.cursor.row];
			let mut ans: i32 = 0;
			let mut num: i32 = 0;
			let mut fugo = '+';
			let mut numcheck = false;
			let mut whitecheck = false;
			let mut fugocheck = false;
			let mut dame = false;
			for ch in command {
				if ch == &' ' {
					whitecheck = true;
					continue;
				}
				if ch.is_digit(10) {
					if numcheck & whitecheck {
						dame = true;
						break;
					}
					num = 10 * num + (ch.to_digit(10).unwrap() as i32);
					numcheck = true;
					whitecheck = false;
					fugocheck = false;
					continue;
				}
				if ch == &'+' {
					if fugocheck {
						dame = true;
						break;
					}
					whitecheck = false;
					numcheck = false;
					fugocheck = true;
					if fugo == '+' {
						ans += num;
						num = 0;
					} else if fugo == '-' {
						ans -= num;
						num = 0;
					}
					fugo = '+';
					continue;
				}
				if ch == &'-' {
					if fugocheck {
						dame = true;
						break;
					}
					whitecheck = false;
					numcheck = false;
					fugocheck = true;
					if fugo == '+' {
						ans += num;
						num = 0;
					} else if fugo == '-' {
						ans -= num;
						num = 0;
					}
					fugo = '-';
					continue;
				}
				else {
					dame = true;
					continue;
				}
			}
			if fugo == '+' {
				ans += num;
			}
			else if fugo == '-' {
				ans -= num;
			}
			if fugocheck == true {
				dame = true;
			}
			if dame {
				self.buffer.insert(self.cursor.row + 1, "I cant calcurate".chars().collect());
			} else {
				let ansc: Vec<char> = ans.to_string().chars().collect();
				self.buffer.insert(self.cursor.row + 1, ansc);
			}
			self.buffer.insert(self.cursor.row + 2, "".chars().collect());
			self.cursor.column = 0;
			self.cursor.row += 2;
		} else if !c.is_control() {
			self.buffer[self.cursor.row].insert(self.cursor.column, c);
			self.cursor_right();
		}
	}
	fn back_space(&mut self) {
		if self.cursor.column > 0 {
			let mut later = self.buffer[self.cursor.row].split_off(self.cursor.column);
			self.buffer[self.cursor.row].pop();
			self.buffer[self.cursor.row].append(&mut later);
			self.cursor.column -= 1;
		}
	}
	fn cursor_left(&mut self) {
		if self.cursor.column > 1 {
			self.cursor.column -= 1;
		}
	}
	fn cursor_right(&mut self) {
		self.cursor.column = min(self.cursor.column + 1, self.buffer[self.cursor.row].len());
	}
}

fn main() {
	let mut state = Dentaku::default();

	let stdin = stdin();

	let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
	state.draw(&mut stdout);

	for evt in stdin.events() {
		match evt.unwrap() {
			Event::Key(Key::Ctrl('c')) => {
				return;
			},
			Event::Key(Key::Left) => {
				state.cursor_left();
			},
			Event::Key(Key::Right) => {
				state.cursor_right();
			},
			Event::Key(Key::Char(c)) => {
				state.insert(c);
			},
			Event::Key(Key::Backspace) => {
				state.back_space();
			},
			_ => {
			},
		}
		state.scroll();
		state.draw(&mut stdout);
	}
}
