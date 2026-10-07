use std::ops::Range;
use thai_syllable_rules::{Part, RuleAnalysis, SyllablePattern, split};

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

/// A written base with its dependent marks, or a leading vowel with the following base and marks.
struct Cell {
    scalars: Range<usize>,
    kind: CandidateKind,
}

/// The best partial path found so far to one lattice position.
#[derive(Clone, Copy)]
struct PathEnd {
    score: f64,
    /// Scalar position where the path's last candidate starts.
    start: usize,
    /// Index of the path's last candidate.
    candidate: usize,
}

/// Relaxed lattice over one Thai run; only source cell boundaries are hard.
#[derive(Clone, Debug)]
pub(crate) struct RelaxedLattice<'a> {
    source: &'a str,
    pub(crate) offsets: Vec<usize>,
    pub(crate) candidates: Vec<Candidate>,
    pub(crate) ending_at: Vec<Vec<usize>>,
    pub(crate) parts: Vec<Part>,
}

impl<'a> RelaxedLattice<'a> {

    /// Exact unchanged Thai run.
    pub(crate) fn source(&self) -> &'a str {
        self.source
    }

    /// Number of Unicode scalars in the Thai run.
    pub(crate) fn scalar_count(&self) -> usize {
        self.offsets.len() - 1
    }
}
