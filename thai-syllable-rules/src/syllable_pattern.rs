use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
use crate::{Part, character::thai_characters, source::split};

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

    /// Inspects one complete Thai candidate's local vowel-pattern structure.
    ///
    /// Returns `None` for empty, non-Thai, or otherwise unparseable source. This
    /// inspects scanner nuclei without validating mark legality or completeness;
    /// it is not a count of spoken syllables or an exhaustive parse forest. A
    /// candidate such as `ลิ` reports one pattern, while a bare terminal pair such
    /// as `ตร` reports zero because it can still serve as continuation material.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::SyllablePattern;
    ///
    /// assert!(SyllablePattern::inspect("กะ").is_some());
    /// assert!(SyllablePattern::inspect("abc").is_none());
    /// ```
    #[must_use]
    pub fn inspect(source: &str) -> Option<Self> {
        Self::inspect_with(source, SilentConsonants::Counted)
    }

    fn inspect_with(source: &str, silent_consonants: SilentConsonants) -> Option<Self> {
        let mut input = source;
        let characters = thai_characters(&mut input).ok()?;
        if !input.is_empty() {
            return None;
        }
        let atoms = CharacterSequence(&characters).thai_atoms();
        let r_han = CharacterSequence(&characters).r_han_patterns();
        Some(Self {
            vowel_pattern_count: atoms
                .iter()
                .filter(|atom| {
                    atom.has_nucleus()
                        && !(silent_consonants == SilentConsonants::Excluded
                            && atom.has_silent_mark()
                            && !atom.has_explicit_rime()
                            && !atom.starts_with_leading_vowel()
                            && !atom.has_carrier()
                            && !atom.has_glide())
                        && !starts_with_silenced_cell(&source[atom.relative_range().clone()])
                        && !matches!(
                            crate::completeness::Completeness::of_atom(atom, source),
                            crate::completeness::Completeness::RequiresHost
                        )
                        && !r_han.iter().any(|pattern| {
                            let range = atom.relative_range();
                            range.start < pattern.nucleus_range.end
                                && pattern.nucleus_range.start < range.end
                        })
                })
                .count()
                + r_han.len(),
            starts_with_leading_vowel: atoms
                .first()
                .is_some_and(super::atom::ThaiAtom::starts_with_leading_vowel),
        })
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
