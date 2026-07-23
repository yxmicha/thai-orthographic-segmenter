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

impl RuleAnalysis {

    /// Returns unresolved pronunciation relationships, never a chosen reading.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::{ReadingAlternative, RuleAnalysis};
    ///
    /// let analysis = RuleAnalysis::new("เกรียน");
    /// assert!(matches!(
    ///     analysis.reading_alternatives()[0],
    ///     ReadingAlternative::WeakInitial { boundary: 6, .. }
    /// ));
    /// ```
    #[must_use]
    pub fn reading_alternatives(&self) -> &[ReadingAlternative] {
        &self.alternatives
    }
}
