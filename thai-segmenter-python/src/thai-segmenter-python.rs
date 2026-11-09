#![warn(missing_docs)]
//! Python bindings for the Thai syllable, word and line segmenters.
//!
//! The extension module `thai_orthographic_segmenter._native` exposes one tokenizer class per segmenter and
//! the rule-based parts of `thai-syllable-rules`. The `thai_orthographic_segmenter` package wraps them in
//! `syllable_tokenize`, `word_tokenize` and the other tokenizer functions.
//! Every call releases the Python interpreter lock while it segments.

mod error;
