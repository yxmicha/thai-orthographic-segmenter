use std::ops::Range;
use thai_syllable_rules::{Part, RuleAnalysis, SyllablePattern, split};

use crate::{Error, lattice_config::LatticeConfig};
use thai_syllable_rules::ThaiScalar;

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

    /// Groups a Thai run's scalars into cells; candidate boundaries fall only between cells.
    fn cells(characters: &[char]) -> Vec<Cell> {
        let mut cells = Vec::new();
        let mut start = 0;
        while start < characters.len() {
            let first = characters[start];
            let mut end = start + 1;
            if ThaiScalar::new(first).is_leading_vowel()
                && characters
                    .get(end)
                    .is_some_and(|value| ThaiScalar::new(*value).is_base())
            {
                end += 1;
            }
            while end < characters.len() && ThaiScalar::new(characters[end]).is_dependent() {
                end += 1;
            }
            let kind = if ThaiScalar::new(first).is_base()
                || ThaiScalar::new(first).is_independent_vowel()
                || ThaiScalar::new(first).is_leading_vowel()
            {
                CandidateKind::Thai
            } else {
                CandidateKind::Unparsed
            };
            cells.push(Cell {
                scalars: start..end,
                kind,
            });
            start = end;
        }
        cells
    }

    /// Prior score of a Thai candidate: the syllable cost, less the costs of unsupported shapes,
    /// extra nuclei and a cut through a rule syllable that no reading alternative recovers.
    fn thai_prior(
        text: &str,
        bytes: &Range<usize>,
        analysis: &RuleAnalysis,
        parts: &[Part],
        config: &LatticeConfig,
    ) -> Result<f32, Error> {
        let pattern = SyllablePattern::inspect(text);
        let recognized = pattern.is_some_and(|pattern| pattern.is_candidate(text))
            || analysis.permits_weak_initial(bytes);
        let recovered_boundary = analysis
            .reading_alternatives()
            .iter()
            .any(|alternative| alternative.boundary() == bytes.end);
        let nuclei = pattern.map_or(0, |pattern| pattern.vowel_pattern_count());
        let cuts_anchor = parts.iter().any(|part| match part {
            Part::Syllable(range) => range.start < bytes.end && bytes.end < range.end,
            Part::Undecided(_) | Part::Invalid(_) | Part::Dot(_) | Part::Text(_) => false,
        });
        let extra = f32::from(
            u16::try_from(nuclei.saturating_sub(1))
                .map_err(|_| Error::Invariant("too many nuclei"))?,
        );
        let prior = 0.0_f32
            - config.syllable_cost
            - if recognized {
                0.0
            } else {
                config.unsupported_cost
            }
            - extra * config.extra_nucleus_cost
            - if cuts_anchor && !recovered_boundary {
                config.anchor_cut_cost
            } else {
                0.0
            };
        if !prior.is_finite() {
            return Err(Error::Invariant("evidence score overflow"));
        }
        Ok(prior)
    }

    /// Exact unchanged Thai run.
    pub(crate) fn source(&self) -> &'a str {
        self.source
    }

    /// Number of Unicode scalars in the Thai run.
    pub(crate) fn scalar_count(&self) -> usize {
        self.offsets.len() - 1
    }
}
