#![deny(missing_docs)]

//! This is a markov chain build with almost no previous experience with building a markov chain, on a lazy friday. This is not production-grade code. Use at own risk etc etc.
//!
//! The main entry point of this is `Memory`. Please see that class for more information. You can look at `main.rs` to see an implementation.

mod error;
mod memory;
mod words;

pub use self::error::Error;
pub use self::memory::Memory;
pub(crate) use self::words::{NextPartList, SentencePart, SentencePartPair};
