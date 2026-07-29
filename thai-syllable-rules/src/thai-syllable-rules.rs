#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Conservative, rule-based Thai syllable candidates.
//!
//! This crate splits Thai text with spelling rules alone. It deliberately does not choose a final
//! segmentation when Thai consonant attachment is ambiguous: a [`Part::Undecided`] range keeps
//! that ambiguity visible to a later resolver, while [`Part::Syllable`] marks an orthographic
//! candidate or anchor and [`Part::Invalid`] keeps malformed local Thai units visible without
//! presenting them as syllables. Every part is a byte range of the input, and the parts cover it
//! exactly, so no character is lost or invented.
//!
//! # Examples
//!
//! ```
//! use thai_syllable_rules::{Part, split};
//!
//! let source = "กะตกลม";
//! let parts = split(source);
//! // `กะ` and `ลม` are syllables; the two consonants between them could attach either way.
//! assert_eq!(
//!     parts,
//!     [
//!         Part::Syllable(0..6),
//!         Part::Undecided(6..9),
//!         Part::Undecided(9..12),
//!         Part::Syllable(12..18),
//!     ]
//! );
//! assert_eq!(parts[0].text(source), "กะ");
//! ```
//!
//! [`RuleAnalysis`] reports the other readings the flat parts cannot show, and [`ThaiScalar`]
//! classifies single characters:
//!
//! ```
//! use thai_syllable_rules::{RuleAnalysis, ThaiScalar};
//!
//! assert!(ThaiScalar::new('เ').is_leading_vowel());
//! assert_eq!(RuleAnalysis::new("เกรียน").reading_alternatives().len(), 2);
//! ```
//!
//! # Edge cases, invariants and panics
//!
//! - Empty input gives no parts. Input without Thai letters is one [`Part::Text`].
//! - Malformed marks, hostless vowels and incomplete constructions become [`Part::Invalid`] or
//!   [`Part::Undecided`]; the parser never reports an error.
//! - Offsets are UTF-8 byte offsets on character boundaries, and the parts are contiguous and
//!   non-empty, so they cover the input exactly.
//! - No function panics for any input. The test suite checks this on arbitrary Unicode with
//!   property-based tests, and the Han-solo fixtures pin the linguistic behaviour.
//!
//! # Where this crate fits
//!
//! `thai-syllable-segmenter` resolves the undecided parts with a bundled model, and
//! `thai-word-segmenter` and `thai-line-segmenter` build words and line-break marks on top of
//! it. This crate has no model and no dependencies beyond a parser library.
//!
//! # Features
//!
//! - `serde`: implements `Serialize` and `Deserialize` for the plain data types.
//! - `test-support`: exposes internal stage snapshots for this repository's integration tests; it is not
//!   part of the stable API.

mod alternative_sites;
mod atom;
mod atom_sequence;
mod atomization_completion;
mod atomization_rime;
mod character;
mod character_sequence;
mod construction_scan;
mod ending_lookahead;
mod part;
mod r_han;
mod reading_alternative;
mod rule_analysis;

pub use r_han::RHanPattern;
pub use reading_alternative::ReadingAlternative;
