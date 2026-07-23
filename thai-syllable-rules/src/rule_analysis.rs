use crate::{RHanPattern, ReadingAlternative};

/// Recoverable readings alongside the flat, source-preserving rules projection.
///
/// # Examples
///
/// ```
/// use thai_syllable_rules::{RuleAnalysis, split};
///
/// let source = "เกรียน";
/// let analysis = RuleAnalysis::new(source);
/// // The rules project the word as one syllable, but the analysis keeps the other reading.
/// assert_eq!(split(source).len(), 1);
/// assert_eq!(analysis.reading_alternatives().len(), 2);
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RuleAnalysis {
    alternatives: Vec<ReadingAlternative>,
    r_han: Vec<RHanPattern>,
}
