//! Hidden integration-test seams for the parser stages.
//!
//! This module is intentionally not part of the normal public API. It gives
//! repository integration tests stable snapshots of the internal parser
//! stages without exposing parser implementation types or test fixtures through
//! the normal library API. The atom module keeps its one read-only semantic
//! trace behind the same `test` feature gate.

use crate::character_sequence::CharacterSequence;
use std::ops::Range;

use winnow::Parser;

use crate::atom::{EndingShape, NucleusShape, OnsetShape, SilentMarkShape, ThaiAtom};
use crate::character::thai_characters;
use crate::resolution_input::{FollowingSource, ThaiRun};

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

/// The semantic ending shape captured by atomization.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndingSnapshot {
    /// The atom does not have a hard structural ending.
    Open,
    /// The atom has a hard structural ending.
    Hard,
}

/// The semantic silent-mark shape captured by atomization.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SilentMarkSnapshot {
    /// No garan mark is present.
    None,
    /// A garan mark is attached without an opaque suffix classification.
    Attached,
    /// A garan-bearing opaque suffix and its shape flags.
    Opaque {
        /// Whether the suffix contains a vowel carrier.
        has_carrier: bool,
        /// Whether the suffix starts with an initial cluster.
        has_initial_cluster: bool,
    },
}

/// Semantic shape metadata for one atom.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticAtomSnapshot {
    /// The atom's onset shape.
    pub onset: OnsetSnapshot,
    /// The atom's nucleus shape.
    pub nucleus: NucleusSnapshot,
    /// The atom's ending shape.
    pub ending: EndingSnapshot,
    /// The atom's silent-mark shape.
    pub silent_mark: SilentMarkSnapshot,
}

/// A read-only snapshot of an atom emitted by Thai atomization.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomSnapshot {
    /// The atom's source-relative UTF-8 range.
    pub range: Range<usize>,
    /// Whether the atom's local ending is structurally closed.
    pub has_hard_ending: bool,
    /// Whether the atom contains a recognized nucleus.
    pub has_nucleus: bool,
    /// The semantic shape classifications used by resolution.
    pub semantic: SemanticAtomSnapshot,
}

impl From<&ThaiAtom> for SemanticAtomSnapshot {
    fn from(atom: &ThaiAtom) -> Self {
        let (onset_shape, nucleus_shape, ending_shape, silent_mark_shape) =
            (atom.onset, atom.nucleus, atom.ending, atom.silent_mark);
        let onset = match onset_shape {
            OnsetShape::None => OnsetSnapshot::None,
            OnsetShape::Base(base) => OnsetSnapshot::Base(base),
            OnsetShape::LeadingVowel { base } => OnsetSnapshot::LeadingVowel { base },
            OnsetShape::IndependentVowel(vowel) => OnsetSnapshot::IndependentVowel(vowel),
            OnsetShape::DependentMark => OnsetSnapshot::DependentMark,
        };
        let nucleus = match nucleus_shape {
            NucleusShape::None => NucleusSnapshot::None,
            NucleusShape::OrphanRime => NucleusSnapshot::OrphanRime,
            NucleusShape::Implicit => NucleusSnapshot::Implicit,
            NucleusShape::Rime {
                explicit,
                carrier,
                completed_ue_carrier,
                glide,
            } => NucleusSnapshot::Rime {
                explicit,
                carrier,
                completed_ue_carrier,
                glide,
            },
            NucleusShape::IndependentVowel => NucleusSnapshot::IndependentVowel,
        };
        let ending = match ending_shape {
            EndingShape::Open => EndingSnapshot::Open,
            EndingShape::Hard => EndingSnapshot::Hard,
        };
        let silent_mark = match silent_mark_shape {
            SilentMarkShape::None => SilentMarkSnapshot::None,
            SilentMarkShape::Attached => SilentMarkSnapshot::Attached,
            SilentMarkShape::Opaque {
                has_carrier,
                has_initial_cluster,
            } => SilentMarkSnapshot::Opaque {
                has_carrier,
                has_initial_cluster,
            },
        };
        Self {
            onset,
            nucleus,
            ending,
            silent_mark,
        }
    }
}

impl From<&ThaiAtom> for AtomSnapshot {
    fn from(atom: &ThaiAtom) -> Self {
        Self {
            range: atom.relative_range().clone(),
            has_hard_ending: atom.has_hard_ending(),
            has_nucleus: atom.has_nucleus(),
            semantic: SemanticAtomSnapshot::from(atom),
        }
    }
}

impl AtomSnapshot {
    fn from_atoms(atoms: &[ThaiAtom]) -> Vec<Self> {
        atoms.iter().map(Self::from).collect()
    }
}

/// Parses a complete Thai run and returns the atomization snapshot.
#[doc(hidden)]
pub fn parsed_atoms(source: &str) -> Option<Vec<AtomSnapshot>> {
    let mut input = source;
    let characters = thai_characters.parse_next(&mut input).ok()?;
    if !input.is_empty() {
        return None;
    }
    Some(AtomSnapshot::from_atoms(
        &CharacterSequence(&characters).thai_atoms(),
    ))
}

/// A read-only snapshot of a prepared Thai run and its source context.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedRunSnapshot {
    /// The run's absolute source offset.
    pub source_offset: usize,
    /// Whether one adjacent period follows the run.
    pub followed_by_single_period: bool,
    /// The prepared atom sequence.
    pub atoms: Vec<AtomSnapshot>,
}

impl<'a> ThaiRun<'a> {
    fn from_test_source(
        source: &'a str,
        source_offset: usize,
        followed_by_single_period: bool,
    ) -> Option<Self> {
        let following_source = if followed_by_single_period {
            FollowingSource::SinglePeriod
        } else {
            FollowingSource::None
        };
        let mut run = ThaiRun::parse(source, source_offset, following_source)?;
        run.prepare_candidates();
        Some(run)
    }
}
