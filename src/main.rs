extern crate rand;

use std::env;
use rand::Rng;
use std::rc::Rc;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::collections::BTreeMap;
use std::cell::{RefCell, RefMut, Cell};
use std::io::{self, BufReader, BufRead};

trait MarkovChain {
	fn gen_sentence(&self) -> String;
}

struct WordsDB {
	words: BTreeMap<String, Rc<WordInfo>>,
	first_words: Vec<String>
}

impl WordsDB {
	fn new() -> WordsDB {
		WordsDB {
			words: BTreeMap::new(),
			first_words: vec![]
		}
	}
	#[allow(dead_code)]
	fn pretty_print_db(&self) {
		for (key, value) in self.words.iter() {
			println!("Word: {}", key);
			println!("Next words: {:?}", value.next_words.borrow());
			println!("Appearance: Begin: {}, Middle: {}, End: {}",
				value.position.0.get(), value.position.1.get(),
				value.position.2.get());
		}
	}
	fn prepare_db_for_work(&mut self) {
		for (key, value) in self.words.iter() {
			if value.position.0.get() > 0 {
				self.first_words.push(key.clone());
			}
		}
	}
}

impl MarkovChain for WordsDB {
	fn gen_sentence(&self) -> String {
		let mut sentence = String::new();
		let max_index = self.first_words.len();
		let first_word_index = rand::thread_rng().gen_range(0, max_index);
		let mut cur_word = self.first_words[first_word_index].clone();
		sentence += &cur_word;
		loop {
			let word_info = self.words.get(&cur_word)
				.expect("Cannot get word info");
			let max_index = word_info.next_words.borrow().len();
			if max_index == 0 {
				break;
			}
			let cur_word_index = rand::thread_rng().gen_range(0, max_index);
			cur_word = word_info.next_words.borrow()[cur_word_index].clone();
			sentence += " ";
			sentence += &cur_word;
		}
		return sentence;
	}
}

#[derive(Debug)]
struct WordInfo {
	next_words: RefCell<Vec<String>>,
	position: Rc<Position>
}

impl WordInfo {
	fn inc_pos(&self, pos: PositionName) {
		match pos {
			PositionName::FirstPos =>
				self.position.0.set(self.position.0.get() + 1),
			PositionName::MiddlePos =>
				self.position.1.set(self.position.1.get() + 1),
			PositionName::EndPos =>
				self.position.2.set(self.position.2.get() + 1)
		}
	}
}

enum PositionName {
	FirstPos,
	MiddlePos,
	EndPos
}

#[derive(Debug)]
struct Position(Cell<u32>, Cell<u32>, Cell<u32>);

fn main() {
	let mut filename:String = "".to_string();
	for (index, arg) in env::args().enumerate() {
		if index == 1 {
			filename = arg;
			break;
		}
	}
	if filename.len() == 0 {
		println!("Please enter file name");
		return;
	}
	let filename = Path::new(&filename);
	if let Some(ext) = filename.extension() {
		if ext == "txt" || ext == "md" {
			println!("Analysing text file: {:?}", filename);
		} else {
			println!("Unknown file extension, use only .txt or .md");
			return;
		}
	} else {
		println!("Unknown file extension, use only .txt or .md");
		return;
	}

	let file = match File::open(filename) {
		Ok(file) => file,
		Err(error) => {
			panic!("{:?}", error.description());
		}
	};
	let mut words_database = WordsDB::new();
	parse_file(&file, &mut words_database);
	words_database.prepare_db_for_work();
	let words_database = words_database;
	println!("Type EXIT to exit the programm");
	loop {
		println!("Random generated sentence:");
		let sentence = words_database.gen_sentence();
		println!("{}", sentence);
		let mut input = String::new();
		let stdin = io::stdin();
		stdin.read_line(&mut input)
			.expect("Failed to read line");
		if input.trim() == "EXIT" {
			break;
		}
	}
}

fn parse_file(file: &File, words_db: &mut WordsDB) {
	let mut buf_reader = BufReader::new(file);
	let mut line = String::new();
	while buf_reader.read_line(&mut line)
		.expect("Error in reading file") > 0 {
		line = line.trim().to_string();
		let mut is_first = true;
		for (i, word) in line.split_whitespace().enumerate() {
			fn push_word(mut next_words: RefMut<Vec<String>>, word: String) {
				if !next_words.contains(&word) && word.len() > 0 {
					next_words.push(word);
				}
			}
			let word = word.to_string();
			let next_word = match line.split_whitespace().nth(i + 1) {
				Some(word) => word.to_string(),
				None => "".to_string()
			};
			let mut word_info = Rc::new(WordInfo {
				next_words: RefCell::new(Vec::new()),
				position: Rc::new(Position(Cell::new(0), 
					Cell::new(0), Cell::new(0)))
			});
			if !words_db.words.contains_key(&word) {
				words_db.words.insert(word.clone(), word_info.clone());
			} else {
				word_info = words_db.words.get(&word)
					.expect("Cannot get word").clone();
			}
			if !is_first {
				if word.ends_with(".") || word.ends_with("?") 
				|| word.ends_with("!") {
					word_info.inc_pos(PositionName::EndPos);
					is_first = true;
				} else {
					push_word(word_info.next_words.borrow_mut(),
						next_word.clone());
					word_info.inc_pos(PositionName::MiddlePos);
				}
			} else if is_first {
				word_info.inc_pos(PositionName::FirstPos);
				is_first = false;
				push_word(word_info.next_words.borrow_mut(),
					next_word.clone());
				if word.ends_with(".") || word.ends_with("?") 
				|| word.ends_with("!") {
					word_info.inc_pos(PositionName::EndPos);
					is_first = true;
				}
			}
		}
		line.clear();
	}
}