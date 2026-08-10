use crate::part::ByteRange;

/// One written รร nucleus and its optional following final consonant.
///
/// # Examples
///
/// ```
/// use thai_syllable_rules::RuleAnalysis;
///
/// // `สรรพ` writes รร as one nucleus: ส + รร, with พ as a possible final.
/// let analysis = RuleAnalysis::new("สรรพ");
/// let pattern = &analysis.r_han_patterns()[0];
/// assert_eq!(pattern.nucleus_range(), &(0..9));
/// assert_eq!(pattern.possible_final(), Some(&(9..12)));
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RHanPattern {
    pub(crate) nucleus_range: ByteRange,
    pub(crate) possible_final: Option<ByteRange>,
}

impl RHanPattern {
    /// Onset and both ร characters, in source bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::RuleAnalysis;
    ///
    /// let analysis = RuleAnalysis::new("สรรพ");
    /// assert_eq!(analysis.r_han_patterns()[0].nucleus_range(), &(0..9));
    /// ```
    #[must_use]
    pub const fn nucleus_range(&self) -> &ByteRange {
        &self.nucleus_range
    }
}
