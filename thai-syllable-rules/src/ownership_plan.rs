use crate::atom::ThaiAtom;
use crate::character::ThaiScalar;
use crate::part::{ByteRange, Part, push_text};

/// Applies ownership decisions while preserving a contiguous UTF-8 partition.
///
/// Atom starts track source already assigned to a neighboring construction.
/// All output and transfers pass through this owner, including the public
/// invariant that a lone consonant cannot be emitted as a complete syllable.
/// Candidate selection and competing-attachment decisions remain in resolution.
pub(crate) struct OwnershipPlan<'a> {
    pub(crate) text: &'a str,
    pub(crate) source_offset: usize,
    pub(crate) atom_ranges: Vec<ByteRange>,
    pub(crate) atom_starts: Vec<usize>,
    pub(crate) parts: Vec<Part>,
}

impl<'a> OwnershipPlan<'a> {
    pub(crate) fn new(text: &'a str, source_offset: usize, atoms: &[ThaiAtom]) -> Self {
        Self {
            text,
            source_offset,
            atom_ranges: atoms
                .iter()
                .map(ThaiAtom::relative_range)
                .cloned()
                .collect(),
            atom_starts: atoms
                .iter()
                .map(|atom| atom.relative_range().start)
                .collect(),
            parts: Vec::new(),
        }
    }

    pub(crate) fn atom_range(&self, index: usize) -> crate::part::ByteRange {
        self.atom_starts[index]..self.atom_ranges[index].end
    }

    pub(crate) fn atom_absolute_range(&self, index: usize) -> crate::part::ByteRange {
        let range = self.atom_range(index);
        (self.source_offset + range.start)..(self.source_offset + range.end)
    }

    pub(crate) fn adopt_atom_prefix(&mut self, target: usize, source: usize) {
        let start = self.atom_starts[source];
        debug_assert!(source <= target);
        debug_assert_eq!(self.emitted_end(), self.source_offset + start);
        self.atom_starts[target] = start;
    }

    pub(crate) fn push_syllable(&mut self, range: ByteRange) {
        // Rule: every public syllable must contain at least two Thai
        // characters. Downgrade shorter candidates at the shared
        // public-output boundary so every resolution path follows the same
        // rule, including isolated and terminal runs. Examples: `ฤ` and `ฮ`
        // become `Undecided`, while `นะก` becomes `นะ | Undecided("ก")` and
        // `กา` remains a syllable.
        // Phinthu suppresses an inherent vowel; adding it to a lone base
        // supplies no nucleus and cannot upgrade that base to a syllable.
        if self.text[range.start - self.source_offset..range.end - self.source_offset]
            .chars()
            .count()
            < 2
            || self.single_base_character(&range).is_some()
            || self.text[range.start - self.source_offset..range.end - self.source_offset]
                .strip_suffix('ฺ')
                .is_some_and(|base| {
                    let mut characters = base.chars();
                    characters
                        .next()
                        .is_some_and(|character| ThaiScalar(character).is_base())
                        && characters.next().is_none()
                })
        {
            self.push_undecided(range);
        } else {
            self.validate_next(&range);
            self.parts.push(Part::Syllable(range));
        }
    }

    pub(crate) fn single_base_character(&self, range: &ByteRange) -> Option<char> {
        let start = range.start.checked_sub(self.source_offset)?;
        let end = range.end.checked_sub(self.source_offset)?;
        let text = self.text.get(start..end)?;
        let mut characters = text.chars();
        let character = characters.next()?;
        (ThaiScalar(character).is_base() && characters.next().is_none()).then_some(character)
    }

    pub(crate) fn push_undecided(&mut self, range: crate::part::ByteRange) {
        self.validate_next(&range);
        self.parts.push(Part::Undecided(range));
    }

    pub(crate) fn push_text(&mut self, range: crate::part::ByteRange) {
        self.validate_next(&range);
        push_text(&mut self.parts, range);
    }

    pub(crate) fn last_is_syllable(&self) -> bool {
        self.parts.last().is_some_and(Part::is_syllable)
    }

    pub(crate) fn extend_last_syllable(&mut self, end: usize) -> bool {
        debug_assert!(end <= self.source_offset + self.text.len());
        debug_assert!(self.text.is_char_boundary(end - self.source_offset));
        debug_assert!(end >= self.emitted_end());
        let Some(Part::Syllable(range)) = self.parts.last_mut() else {
            return false;
        };
        range.end = end;
        true
    }

    pub(crate) fn emitted_end(&self) -> usize {
        self.parts
            .last()
            .map_or(self.source_offset, |part| part.byte_range().end)
    }

    pub(crate) fn validate_next(&self, range: &ByteRange) {
        debug_assert_eq!(
            range.start,
            self.emitted_end(),
            "ownership must be contiguous"
        );
        debug_assert!(range.start < range.end, "ownership must consume source");
        debug_assert!(range.end <= self.source_offset + self.text.len());
        debug_assert!(self.text.is_char_boundary(range.start - self.source_offset));
        debug_assert!(self.text.is_char_boundary(range.end - self.source_offset));
    }
}
