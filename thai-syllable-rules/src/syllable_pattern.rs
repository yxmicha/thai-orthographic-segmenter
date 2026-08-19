use crate::character::ThaiScalar;

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

impl SyllablePattern {
    /// Number of atom-level vowel nuclei in the candidate.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::SyllablePattern;
    ///
    /// assert_eq!(SyllablePattern::inspect("ลิ").map(|pattern| pattern.vowel_pattern_count()), Some(1));
    /// // A bare terminal pair is continuation material and has no nucleus.
    /// assert_eq!(SyllablePattern::inspect("ตร").map(|pattern| pattern.vowel_pattern_count()), Some(0));
    /// ```
    #[must_use]
    pub const fn vowel_pattern_count(&self) -> usize {
        self.vowel_pattern_count
    }

    /// Whether the candidate begins with a preposed vowel construction.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::SyllablePattern;
    ///
    /// let leading = SyllablePattern::inspect("เกรียน").expect("Thai candidate");
    /// let plain = SyllablePattern::inspect("กะ").expect("Thai candidate");
    /// assert!(leading.starts_with_leading_vowel());
    /// assert!(!plain.starts_with_leading_vowel());
    /// ```
    #[must_use]
    pub const fn starts_with_leading_vowel(&self) -> bool {
        self.starts_with_leading_vowel
    }
}

fn starts_with_silenced_cell(source: &str) -> bool {
    let mut characters = source.chars();
    if !characters
        .next()
        .is_some_and(|character| ThaiScalar(character).is_base())
    {
        return false;
    }
    characters
        .take_while(|character| {
            !ThaiScalar(*character).is_base() && !ThaiScalar(*character).is_leading_vowel()
        })
        .any(|character| ThaiScalar(character).is_garan_mark())
}

/// Whether consonant-only silent-mark atoms count as vowel patterns.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SilentConsonants {
    Counted,
    Excluded,
}
