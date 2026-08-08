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
