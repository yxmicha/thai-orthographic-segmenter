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
