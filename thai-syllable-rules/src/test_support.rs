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

/// The semantic nucleus shape captured by atomization.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NucleusSnapshot {
    /// No nucleus was found.
    None,
    /// Dependent vowel material without a consonant base.
    OrphanRime,
    /// An implicit vowel nucleus.
    Implicit,
    /// An explicit rime and its structural flags.
    Rime {
        /// Whether the rime contains an explicit vowel mark.
        explicit: bool,
        /// Whether an `อ` carrier participates in the rime.
        carrier: bool,
        /// Whether a `ือ` carrier was completed.
        completed_ue_carrier: bool,
        /// Whether a `ย` or `ว` glide participates in the rime.
        glide: bool,
    },
    /// An independent `ฤ` or `ฦ` nucleus.
    IndependentVowel,
}
