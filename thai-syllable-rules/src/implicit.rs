use crate::part::ByteRange;

/// Structural ranges synthesized from a terminal bare run.
///
/// The candidate pass returns ranges rather than public parts. Ownership
/// resolution decides whether these ranges become syllables or remain part of
/// an unresolved run; public parts are emitted only after that decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImplicitCandidateRanges {
    pub(crate) first: ByteRange,
    pub(crate) second: Option<ByteRange>,
}
