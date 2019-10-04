use crate::{Error, NextPartList, SentencePart, SentencePartPair};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use rand::Rng;

/// The markov chain. This contains the history of all the word combinations this chain has seen.
///
/// This chain can either be created by using `Default::default()`, or loaded from a file with `Memory::load`. The chain can be saved by calling `Memory::save`
///
/// To learn new sentences, call `Memory::learn(line: &str)`.
///
/// To get a sentence that starts with a given word, call `Memory::get(starting_word: &str)`
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Memory {
    words: HashMap<SentencePartPair, NextPartList>,
}

impl Memory {
    /// Loads a markov chain from a given file. This file should be a zip of a binary representation of a previously saved chain.
    pub fn load(file: impl AsRef<Path>) -> Result<Memory, Error> {
        let fs = File::open(file.as_ref()).map_err(Error::CouldNotOpenFile)?;
        let mut reader = zip::ZipArchive::new(fs).map_err(Error::CouldNotReadZip)?;
        let first_entry = reader.by_index(0).map_err(Error::CouldNotReadFirstFile)?;
        let result = bincode::deserialize_from(first_entry).map_err(Error::CouldNotDeserialize)?;
        Ok(result)
    }

    /// Save this chain to a file. This will serialize this memory with `bincode::serialize_into`, and save that into a zip file. As such, the file extension should be `.zip`
    pub fn save(&self, file: impl AsRef<Path>) -> Result<(), Error> {
        let fs = File::create(file).map_err(Error::CouldNotCreateFile)?;
        let mut writer = zip::ZipWriter::new(fs);
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        writer
            .start_file("memory.dat", options)
            .map_err(Error::CouldNotCreateZipEntry)?;
        bincode::serialize_into(&mut writer, self).map_err(Error::CouldNotSerialize)?;
        Ok(())
    }

    /// Learn the given line. This will append the word combinations to the internal memory model.
    pub fn learn(&mut self, line: &str) {
        // We split the line into chunks:
        // - __START__ + first word
        // - first word + second word
        // - ...
        // - last_word + __END__
        let mut previous_pair = SentencePartPair::default();

        let add_sequence = |memory: &mut Self, prev_pair, part| {
            memory
                .words
                .entry(prev_pair)
                .or_insert_with(NextPartList::default)
                .count_part(part);
        };

        for part in line
            .split_ascii_whitespace()
            .filter(|part| !part.trim().is_empty())
        {
            if previous_pair.is_valid_sentence() {
                // if the `previous` is a valid word segment, we add the current word to the list of follow-up words.
                let new_word = SentencePart::Word(part.to_owned());
                add_sequence(self, previous_pair.clone(), new_word);
            }
            previous_pair.shift(part);
        }
        // this should always be true, unless the caller provides an empty string
        if previous_pair.is_valid_sentence() {
            add_sequence(self, previous_pair, SentencePart::EndOfLine);
        }
    }

    /// Tries to produce a sentence starting with the given `starting_word`.
    ///
    /// No validation is given to the word, if the starting word is not a valid word (e.g. it's multiple words), this function will always return None.
    pub fn speak(&self, starting_word: &str) -> Option<String> {
        let mut len = 0;
        let mut rand = rand::thread_rng();
        let mut result = String::new();

        // We always start with __START__, starting_word
        let mut previous_pair =
            SentencePartPair::with_previous_word(starting_word.to_ascii_lowercase());

        // While the combination of the last 2 words is known
        while let Some(words) = self.words.get(&previous_pair) {
            // Try to get a random follow-up word
            let word = match words.get(&mut rand) {
                Some(SentencePart::Word(next_word)) => next_word,
                _ => break,
            };

            if !result.is_empty() {
                result += " ";
            }
            result += word;
            previous_pair.shift(word);

            len += 1;

            // We don't want to get in an infinite loop,
            // so we add 10% chance to break at the current word, for each 3 words we added
            let chance_to_break = (len / 3) * 10;
            if rand.gen_bool(chance_to_break as f64 / 100.0) {
                break;
            }
        }

        if result.is_empty() {
            None
        } else {
            // Make sure to prepend the requested `starting_word`
            Some(format!("{} {}", starting_word.to_ascii_lowercase(), result))
        }
    }
}
