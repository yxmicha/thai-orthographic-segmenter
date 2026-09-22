use std::ops::Range;

/// Source treatment, independent of the model's confidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidateKind {
    /// Proposed Thai orthographic syllable.
    Thai,
    /// An orphan Thai mark or unsupported non-base cell, preserved verbatim.
    Unparsed,
}

/// One candidate edge with immutable source coordinates and prior score.
#[derive(Clone, Debug)]
pub(crate) struct Candidate {
    pub(crate) bytes: Range<usize>,
    pub(crate) scalars: Range<usize>,
    pub(crate) kind: CandidateKind,
    pub(crate) prior: f32,
}
