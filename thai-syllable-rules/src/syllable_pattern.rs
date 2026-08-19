/// Structural vowel-pattern facts for one rule candidate.
///
/// This describes local orthography only. It does not decide whether the
/// candidate should be separated from a neighboring candidate.
///
/// # Examples
///
/// ```
/// use thai_syllable_rules::SyllablePattern;
///
/// let pattern = SyllablePattern::inspect("เกรียน").expect("Thai candidate");
/// assert_eq!(pattern.vowel_pattern_count(), 1);
/// assert!(pattern.starts_with_leading_vowel());
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SyllablePattern {
    pub(crate) vowel_pattern_count: usize,
    pub(crate) starts_with_leading_vowel: bool,
}
