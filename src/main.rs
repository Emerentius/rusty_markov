use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::path::Path;

fn main() {
    const MEMORY_FILE: &str = "memory.zip";
    let memory = if Path::new(MEMORY_FILE).exists() {
        println!("Loading memory from file");
        let memory = Memory::load(MEMORY_FILE).expect("Could not read memory");
        println!("Done");
        memory
    } else {
        println!("Starting with a clean slate");
        let mut memory = Memory::default();
        println!("Learning...");
        let logs = File::open("logs.txt").expect("Could not open logs");
        let logs_reader = BufReader::new(logs);
        let mut count = 0;
        let start = time::precise_time_s();
        for line in logs_reader.lines() {
            if let Some(text) = line.as_ref().ok().and_then(|l| l.split('>').nth(1)) {
                count += 1;
                if count % 10000 == 0 {
                    println!("Line {}", count);
                }
                memory.learn(text);
            }
        }
        let end = time::precise_time_s();
        let time_diff = end - start;
        println!(
            "Learned {} lines in {:.2} s ({:.2} lines/sec)",
            count,
            time_diff,
            count as f64 / time_diff
        );
        println!("Saving memory");
        memory.save(MEMORY_FILE).expect("Could not save memory");
        memory
    };

    for word in &["hello", "Hmm", "butt", "sluttier", "nice"] {
        let line = memory.speak(word);
        println!("{:?} -> {:?}", word, line);
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Memory {
    words: HashMap<PrevWords, NextWordList>,
}

#[derive(Debug)]
pub enum MemoryError {
    CouldNotOpenFile(IoError),
    CouldNotCreateFile(IoError),

    CouldNotReadZip(zip::result::ZipError),
    CouldNotReadFirstFile(zip::result::ZipError),
    CouldNotCreateZipEntry(zip::result::ZipError),

    CouldNotDeserialize(bincode::Error),
    CouldNotSerialize(bincode::Error),
}

impl Memory {
    pub fn load(file: impl AsRef<Path>) -> Result<Memory, MemoryError> {
        let fs = File::open(file.as_ref()).map_err(MemoryError::CouldNotOpenFile)?;
        let mut reader = zip::ZipArchive::new(fs).map_err(MemoryError::CouldNotReadZip)?;
        let first_entry = reader
            .by_index(0)
            .map_err(MemoryError::CouldNotReadFirstFile)?;
        let result =
            bincode::deserialize_from(first_entry).map_err(MemoryError::CouldNotDeserialize)?;
        Ok(result)
    }

    pub fn save(&self, file: impl AsRef<Path>) -> Result<(), MemoryError> {
        let fs = File::create(file).map_err(MemoryError::CouldNotCreateFile)?;
        let mut writer = zip::ZipWriter::new(fs);
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        writer
            .start_file("memory.dat", options)
            .map_err(MemoryError::CouldNotCreateZipEntry)?;
        bincode::serialize_into(&mut writer, self).map_err(MemoryError::CouldNotSerialize)?;
        Ok(())
    }

    pub fn learn(&mut self, line: &str) {
        let mut previous = PrevWords {
            prev: Word::StartOfLine,
            prev_prev: Word::StartOfLine,
        };
        for part in line.split_ascii_whitespace() {
            let word = Word::Word(part.to_ascii_lowercase());
            if previous.prev.is_word() {
                let entry = self
                    .words
                    .entry(previous.clone())
                    .or_insert_with(Default::default);
                entry.add_word(word.clone());
            }
            previous.shift(word);
        }
        if previous.prev.is_word() {
            let entry = self.words.entry(previous).or_insert_with(Default::default);
            entry.add_word(Word::EndOfLine);
        }
    }

    pub fn speak(&self, starting_word: &str) -> Option<String> {
        let mut len = 0;
        let mut rand = rand::rngs::ThreadRng::default();
        let mut prev = PrevWords {
            prev_prev: Word::StartOfLine,
            prev: Word::Word(starting_word.to_ascii_lowercase()),
        };
        let mut result = String::new();

        while let Some(words) = self.words.get(&prev) {
            let next_word = match words.get(&mut rand) {
                Some(next_word) => next_word,
                None => {
                    break;
                }
            };

            if let Word::Word(word) = next_word {
                if !result.is_empty() {
                    result += " ";
                }
                result += word;
                prev.shift(Word::Word(word.to_owned()));

                len += 1;

                let chance_to_break = (len / 3) * 10; // 10% per 3 words

                use rand::Rng;
                if rand.gen_range(0, 100) < chance_to_break {
                    break;
                }
            } else {
                break;
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(format!("{} {}", starting_word.to_ascii_lowercase(), result))
        }
    }
}

#[derive(Hash, Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
pub struct PrevWords {
    prev: Word,
    prev_prev: Word,
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
