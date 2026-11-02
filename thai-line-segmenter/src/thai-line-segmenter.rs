#![forbid(unsafe_code)]
#![warn(missing_docs)]
//! Thai line segmenter with script-aware punctuation attachment and zero-width joiner insertion.
//!
//! Browsers and renderers without a Thai dictionary break Thai lines in the wrong places, or
//! over-split words with their own dictionary. [`LineSegmenter`] prepares text for them by
//!
//! 1. segmenting Thai with the neural [`thai_word_segmenter::WordSegmenter`],
//! 2. attaching punctuation prefixes and suffixes to the adjacent segment,
//! 3. inserting zero-width joiners ([`ZERO_WIDTH_JOINER`]) inside words wherever the standard ICU
//!    Thai break rules would split them, and
//! 4. inserting zero-width spaces ([`ZERO_WIDTH_SPACE`]) at the break opportunities between
//!    words.
//!
//! Unlike the other segmenters it inserts characters, so its output is not a partition of the
//! input; removing the zero-width characters gives the input back.
//!
//! # Examples
//!
//! ```
//! use thai_line_segmenter::{LineSegmenter, ZERO_WIDTH_SPACE};
//!
//! let segmenter = LineSegmenter::new()?;
//! let marked = segmenter.normalize_line_boundaries("ในปฐมกาลพระวาทะ")?;
//! assert!(marked.contains(ZERO_WIDTH_SPACE));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Edge cases, invariants and panics
//!
//! - Empty input gives an empty string, and text without Thai is returned unchanged except for
//!   the punctuation rules above.
//! - Zero-width characters in the input are dropped and special spaces become ordinary spaces
//!   before segmentation; a space joined to attached punctuation becomes a non-breaking space.
//! - Characters of the Thai block outside the accepted alphabet, such as `๚`, are kept.
//! - No function panics for any input; property-based tests check that removing the inserted
//!   characters restores the input.
//!
//! # Threads
//!
//! A [`LineSegmenter`] is `Send + Sync`. Building one loads a word model, which takes about
//! 40 ms, so build one and share it.

mod error;
/// Longest-match dictionary for ICU Thai break patterns.
pub mod icu_dictionary;
