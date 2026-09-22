/// Source treatment, independent of the model's confidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidateKind {
    /// Proposed Thai orthographic syllable.
    Thai,
    /// An orphan Thai mark or unsupported non-base cell, preserved verbatim.
    Unparsed,
}
