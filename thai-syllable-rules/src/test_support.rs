//! Hidden integration-test seams for the parser stages.
//!
//! This module is intentionally not part of the normal public API. It gives
//! repository integration tests stable snapshots of the internal parser
//! stages without exposing parser implementation types or test fixtures through
//! the normal library API. The atom module keeps its one read-only semantic
//! trace behind the same `test` feature gate.


/// The semantic onset shape captured by atomization.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnsetSnapshot {
    /// No onset was found.
    None,
    /// A base consonant onset.
    Base(char),
    /// A preposed-vowel onset and its optional base.
    LeadingVowel {
        /// The base following the preposed vowel, if present.
        base: Option<char>,
    },
    /// An independent vowel onset.
    IndependentVowel(char),
    /// A dependent mark at the start of malformed source material.
    DependentMark,
}
