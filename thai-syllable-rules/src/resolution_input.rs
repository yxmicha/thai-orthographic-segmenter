use crate::atom_sequence::AtomSequence;
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

    /// Adds structural candidates without deciding public ownership.
    pub(crate) fn prepare_candidates(&mut self) {
        // Rule: a terminal bare run may need a synthetic right-hand candidate
        // so the preceding run can expose ambiguity instead of swallowing it.
        // Example: in `กะตกลม`, the terminal `ลม` candidate gives the bare
        // material after `กะ` a right-hand anchor. Without that candidate, the
        // resolver would have no structural reason to expose the choice.
        if self.following_source.has_single_period() {
            // Rule: before one adjacent period, a bare singleton may be an
            // abbreviation tail, so keep the prior candidate and singleton
            // split. Example: `นะก.` becomes `นะ | ?ก | .`, and the same
            // suffix pass makes `ในจมก.` become `ใน | จม | ?ก | .`; a
            // repeated `นะก..` does not receive this exception because the
            // source scanner classifies the punctuation as ordinary `Text`.
            self.promote_bare_prefix_before_singleton();
        }
        self.promote_terminal_implicit_syllable();
    }

    /// Splits a valid leading implicit candidate from a final bare singleton.
    fn promote_bare_prefix_before_singleton(&mut self) {
        // Rule: before one period, split a valid leading implicit candidate from a
        // final bare singleton so the singleton can represent an abbreviation.
        // The candidate may be the whole Thai run or the bare suffix after an
        // earlier nucleus. Examples: `นะก.` becomes `นะ | ?ก | .`, `จมก.`
        // becomes `จม | ?ก | .`, and `ในจมก.` becomes `ใน | จม | ?ก | .`.
        // Repeated periods do not enter this helper.
        let text = self.text;
        let atoms = &mut self.atoms;
        if atoms.len() < 3 {
            return;
        }

        let tail_index = atoms.len() - 1;
        let Some(tail) = atoms.get(tail_index) else {
            return;
        };
        if tail.first_base().is_none()
            || tail.has_silent_mark()
            || text[tail.relative_range().clone()].chars().count() != 1
        {
            return;
        }

        let unresolved_start = atoms[..tail_index]
            .iter()
            .rposition(ThaiAtom::has_nucleus)
            .map_or(0, |index| index + 1);
        let prefix = &atoms[unresolved_start..tail_index];
        if prefix.iter().any(|atom| atom.first_base().is_none())
            || AtomSequence(prefix).implicit_syllable_sequence_start() != Some(prefix.len())
        {
            return;
        }

        let Some(leading) = ThaiAtom::synthetic_implicit(prefix) else {
            return;
        };
        let Some(tail) = atoms.pop() else {
            return;
        };
        atoms.truncate(unresolved_start);
        atoms.push(leading);
        atoms.push(tail);
    }

    /// Keeps a terminal implicit-vowel syllable available as a following candidate.
    fn promote_terminal_implicit_syllable(&mut self) {
        // Rule: keep a terminal implicit-vowel syllable available as the following
        // candidate when preceding consonants remain ambiguous.
        // Examples: in `กะตกลม`, a terminal `ลม` candidate gives the preceding
        // bare tail a right-hand anchor; without that candidate the tail would
        // incorrectly be absorbed into `กะ`. The promoted range remains contiguous
        // with the original terminal atoms.
        let Some(last_nucleus) = self
            .atoms
            .iter()
            .rposition(super::atom::ThaiAtom::has_nucleus)
        else {
            self.promote_bare_consonant_syllable_candidates();
            return;
        };
        let atoms = &mut self.atoms;
        let unresolved_start = last_nucleus + 1;
        let unresolved_count = atoms.len().saturating_sub(unresolved_start);
        if unresolved_count < 2 {
            return;
        }
        let implicit_length = if unresolved_count == 2 {
            // Rule: a short terminal tail is one conservative implicit candidate;
            // splitting it further would manufacture an orphan consonant without
            // a right-hand syllable to receive it.
            // Example: a two-atom terminal tail such as `จม` stays one candidate;
            // it does not become `จ | ม` merely to create more boundaries when
            // neither side has a following structural nucleus.
            unresolved_count
        } else if AtomSequence(&atoms[atoms.len() - 3..]).is_implicit_glide_syllable() {
            3
        } else {
            2
        };
        if unresolved_count < implicit_length {
            return;
        }

        let syllable_start = atoms.len() - implicit_length;
        let syllable_atoms = &atoms[syllable_start..];
        if syllable_atoms
            .iter()
            .any(|atom| atom.has_nucleus() || atom.first_base().is_none())
        {
            return;
        }

        let Some(syllable) = ThaiAtom::synthetic_implicit(syllable_atoms) else {
            return;
        };
        atoms.truncate(syllable_start);
        atoms.push(syllable);
    }

    /// Gives an all-bare terminal run structural left and right candidates.
    fn promote_bare_consonant_syllable_candidates(&mut self) {
        // Rule: an all-bare terminal run gets structural left and right candidates
        // only when both have the minimum implicit shape; the middle remains
        // undecided for the caller.
        // Examples: a four-atom run can expose two two-atom candidates with the
        // intervening attachment made explicit; a three-atom run is preserved
        // instead of yielding a standalone middle consonant. This keeps every
        // `Undecided` part between real candidates.
        let atoms = &mut self.atoms;
        let implicit_length = if atoms.len() >= 3
            && AtomSequence(&atoms[atoms.len() - 3..]).is_implicit_glide_syllable()
        {
            3
        } else {
            2
        };
        let ambiguous_length = implicit_length + 1;
        let reserved_suffix_length = implicit_length + ambiguous_length;
        let Some(leading_length) = atoms.len().checked_sub(reserved_suffix_length) else {
            return;
        };
        if leading_length < 2
            || atoms
                .iter()
                .any(|atom| atom.has_nucleus() || atom.first_base().is_none())
        {
            return;
        }

        let terminal_start = leading_length + ambiguous_length;
        let Some(leading) = ThaiAtom::synthetic_implicit(&atoms[..leading_length]) else {
            return;
        };
        let Some(terminal) = ThaiAtom::synthetic_implicit(&atoms[terminal_start..]) else {
            return;
        };
        let middle = atoms[leading_length..terminal_start].to_vec();

        atoms.clear();
        atoms.push(leading);
        atoms.extend(middle);
        atoms.push(terminal);
    }
}
