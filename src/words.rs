use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Helper struct to contain 2 parts
#[derive(Hash, Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
pub struct SentencePartPair {
    prev: SentencePart,
    prev_prev: SentencePart,
}

impl Default for SentencePartPair {
    fn default() -> Self {
        Self {
            prev: SentencePart::StartOfLine,
            prev_prev: SentencePart::StartOfLine,
        }
    }
}

impl SentencePartPair {
    /// Create a pair with the segments (__START__, s)
    pub fn with_previous_word(s: impl Into<String>) -> Self {
        Self {
            prev: SentencePart::Word(s.into()),
            prev_prev: SentencePart::StartOfLine,
        }
    }

    /// Checks to see if this pair is a valid sentence. In effect, it checks if the last SentencePart is a Word
    pub fn is_valid_sentence(&self) -> bool {
        self.prev.is_word()
    }

    /// Shift the pair, so that (`prev`, `prev_prev`) becomes (`word`, `prev`). The old `prev_prev` gets pushed off
    pub fn shift(&mut self, new_prev: impl Into<String>) {
        std::mem::swap(&mut self.prev, &mut self.prev_prev);
        self.prev = SentencePart::Word(new_prev.into());
    }
}

/// A sentence part, which can either be a StartOfLine, EndOfLine, or an actual word
#[derive(Hash, Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
pub enum SentencePart {
    StartOfLine,
    EndOfLine,
    Word(String),
}

impl SentencePart {
    /// Check if a given SentencePart is a SentencePart::Word
    pub fn is_word(&self) -> bool {
        match self {
            SentencePart::Word(_) => true,
            _ => false,
        }
    }
}

/// Wrapper around a `HashMap<SentencePart, usize>`. Used for count how many times a follow-up part occured.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct NextPartList {
    parts: HashMap<SentencePart, usize>,
}

impl NextPartList {
    /// Count a part towards this `NextPartList`. If the part does not exist, it will be added.
    pub fn count_part(&mut self, part: SentencePart) {
        *self.parts.entry(part).or_insert(0) += 1;
    }

    /// Get a random sentence part from this list, weighed towards the part that is mostly used.
    ///
    /// Given a list containing 2 parts, one at 9 usages, and one at 1 usages, this function has a 90% chance to return the first part and a 10% chance to return the second.
    pub fn get(&self, rng: &mut impl rand::Rng) -> Option<&SentencePart> {
        if self.parts.is_empty() {
            return None;
        }
        let total: usize = self.parts.values().sum();
        let mut index = rng.gen_range(0, total);
        for (part, count) in &self.parts {
            if *count > index {
                return Some(part);
            }
            index -= count;
        }
        None
    }
}
