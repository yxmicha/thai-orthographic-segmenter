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

    /// Counts substantial constructions for the one-pattern dataset constraint.
    ///
    /// Like [`Self::inspect`], but consonant-only silent-mark atoms do
    /// not introduce a vowel pattern. For example, the opaque `มพ์` continuation
    /// in `พิมพ์` is not another vowel. Bare consonants likewise add no pattern.
    /// Compound vowels and r-han remain single constructions. This is a structural
    /// upper-bound check, not proof of completeness or a phonetic syllable count.
    /// The original scanner count is preserved for existing model score semantics.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::SyllablePattern;
    ///
    /// assert_eq!(SyllablePattern::inspect_substantial("กะ").map(|pattern| pattern.vowel_pattern_count()), Some(1));
    /// assert!(SyllablePattern::inspect_substantial("abc").is_none());
    /// ```
    #[must_use]
    pub fn inspect_substantial(source: &str) -> Option<Self> {
        Self::inspect_with(source, SilentConsonants::Excluded)
    }

    /// Whether `source`, whose pattern this is, is a complete orthographic candidate.
    ///
    /// `self` must be the result of [`Self::inspect`] for the same `source`, so the pattern is
    /// not recomputed. Unlike the minimum-shape query [`is_one_syllable_or_more`], this rejects
    /// ordinary material following a hard-closed rime. Context-licensed weak initials are
    /// handled by `RuleAnalysis`.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::SyllablePattern;
    ///
    /// let source = "กะ";
    /// let pattern = SyllablePattern::inspect(source).expect("Thai candidate");
    /// assert!(pattern.is_candidate(source));
    /// // Ambiguous consonants between two syllables are not one candidate.
    /// assert!(!SyllablePattern::inspect("กะตกลม").expect("Thai").is_candidate("กะตกลม"));
    /// ```
    #[must_use]
    pub fn is_candidate(self, source: &str) -> bool {
        // The pattern count is the cheap check; `is_one_syllable_or_more` runs a full split.
        if self.vowel_pattern_count > 1 || !is_one_syllable_or_more(source) {
            return false;
        }
        let mut input = source;
        let Ok(characters) = thai_characters(&mut input) else {
            return false;
        };
        let atoms = CharacterSequence(&characters).thai_atoms();
        if atoms
            .first()
            .is_some_and(|atom| starts_with_silenced_cell(&source[atom.relative_range().clone()]))
        {
            return false;
        }
        for (index, atom) in atoms.iter().enumerate() {
            if atom.has_hard_ending()
                && !matches!(
                    crate::completeness::Completeness::of_atom(atom, source),
                    crate::completeness::Completeness::RequiresHost
                )
                && atoms[index + 1..].iter().any(|following| {
                    following.first_base().is_some()
                        && !matches!(
                            crate::completeness::Completeness::of_atom(following, source),
                            crate::completeness::Completeness::RequiresHost
                        )
                })
            {
                return false;
            }
        }
        true
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

/// Returns whether `source` contains at least one complete syllable pattern.
///
/// A nucleus-bearing atom counts only when its spelling contains at least two
/// Thai characters. A pair of adjacent consonant atoms also counts: Thai's
/// implicit-vowel spelling can be represented by two consonants even when the
/// atom scanner keeps their ownership unresolved. This deliberately answers a
/// minimum-shape question, not whether the source is exactly one syllable;
/// callers that need the number of patterns should use
/// [`SyllablePattern::inspect`].
///
/// # Examples
///
/// ```
/// use thai_syllable_rules::is_one_syllable_or_more;
///
/// assert!(is_one_syllable_or_more("กะ"));
/// assert!(!is_one_syllable_or_more("abc"));
/// ```
#[must_use]
pub fn is_one_syllable_or_more(source: &str) -> bool {
    let parts = split(source);
    if parts.is_empty()
        || parts
            .iter()
            .any(|part| matches!(part, Part::Invalid(_) | Part::Dot(_) | Part::Text(_)))
    {
        return false;
    }
    parts
        .iter()
        .any(|part| matches!(part, Part::Syllable(_)) && part.text(source).chars().count() >= 2)
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
