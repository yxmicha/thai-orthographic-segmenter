use crate::atom::ThaiAtom;
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
}
