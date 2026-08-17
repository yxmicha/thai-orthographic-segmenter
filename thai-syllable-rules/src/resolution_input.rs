use crate::character_sequence::CharacterSequence;
use winnow::Parser;

use crate::atom::ThaiAtom;
use crate::character::{ThaiCharacter, thai_characters};

/// The source context immediately following a maximal Thai run.
///
/// This is deliberately richer than a boolean so source punctuation policy is
/// named at the source/Thai seam and cannot silently spread as an unrelated
/// flag through ownership resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FollowingSource {
    /// The Thai run is not immediately followed by one eligible period.
    None,
    /// The Thai run is immediately followed by exactly one ASCII period.
    SinglePeriod,
}

impl FollowingSource {
    pub(crate) fn has_single_period(self) -> bool {
        matches!(self, Self::SinglePeriod)
    }
}

/// All source-relative state for one maximal Thai run.
///
/// Parsing and candidate preparation deliberately keep the same owner. The
/// stages still have separate methods, but the text, offset, source context,
/// character metadata, and atom sequence cannot be accidentally detached from
/// one another between stages.
pub(crate) struct ThaiRun<'a> {
    pub(crate) text: &'a str,
    pub(crate) source_offset: usize,
    pub(crate) following_source: FollowingSource,
    pub(crate) characters: Vec<ThaiCharacter>,
    pub(crate) atoms: Vec<ThaiAtom>,
}

impl<'a> ThaiRun<'a> {
    /// Parses one maximal Thai source run into scalar metadata and local atoms.
    pub(crate) fn parse(
        text: &'a str,
        source_offset: usize,
        following_source: FollowingSource,
    ) -> Option<Self> {
        let mut parser = thai_characters;
        let characters = parser.parse(text).ok()?;
        let atoms = CharacterSequence(&characters).thai_atoms();

        Some(Self {
            text,
            source_offset,
            following_source,
            characters,
            atoms,
        })
    }
}
