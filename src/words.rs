use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Hash, Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
pub struct PrevWords {
    pub prev: Word,
    pub prev_prev: Word,
}

impl PrevWords {
    pub fn shift(&mut self, new_prev: Word) {
        let new_prev_prev = std::mem::replace(&mut self.prev, new_prev);
        self.prev_prev = new_prev_prev;
    }
}

#[derive(Hash, Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
pub enum Word {
    StartOfLine,
    EndOfLine,
    Word(String),
}

impl Word {
    pub fn is_word(&self) -> bool {
        match self {
            Word::Word(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct NextWordList {
    words: HashMap<Word, usize>,
}

impl NextWordList {
    pub fn add_word(&mut self, word: Word) {
        let entry = self.words.entry(word).or_default();
        *entry += 1;
    }

    pub fn get(&self, rng: &mut impl rand::Rng) -> Option<&Word> {
        if self.words.is_empty() {
            return None;
        }
        let total: usize = self.words.values().sum();
        let mut index = rng.gen_range(0, total);
        for (word, count) in &self.words {
            if *count > index {
                return Some(word);
            }
            index -= count;
        }
        None
    }
}
