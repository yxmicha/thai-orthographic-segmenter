use crate::part::ByteRange;

/// A possible pronunciation relationship that does not duplicate source ownership.
///
/// These are alternatives to be resolved with lexical/contextual evidence, not
/// assertions that the consonant is repeated or a weak syllable is pronounced.
///
/// # Examples
///
/// ```
/// use thai_syllable_rules::{ReadingAlternative, RuleAnalysis};
///
/// let analysis = RuleAnalysis::new("สรรพ");
/// assert_eq!(
///     analysis.reading_alternatives(),
///     [ReadingAlternative::SharedConsonant { source_range: 6..9, boundary: 9 }]
/// );
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReadingAlternative {
    /// A bare consonant may supply a weak initial syllable before another shape.
    WeakInitial {
        /// Source spelling of that consonant.
        source_range: ByteRange,
        /// Source boundary after the weak initial.
        boundary: usize,
    },
    /// A preceding coda may also supply an additional weak syllable after its
    /// source owner (กิจการ: กิด-จะ-กาน; สรรพคุณ: สับ-พะ-คุน). This reference
    /// does not assert that two source spans are only two spoken syllables.
    SharedConsonant {
        /// The single written consonant referenced by both readings.
        source_range: ByteRange,
        /// Boundary after the source owner; no source character is inserted.
        boundary: usize,
    },
    /// A preposed vowel can scope over the second onset after a weak initial.
    LeadingVowelScope {
        /// Written preposed vowel, retained at its original source position.
        vowel_range: ByteRange,
        /// Weak consonant between the vowel and its possible following onset.
        consonant_range: ByteRange,
        /// Following onset which may receive the preposed vowel.
        following_onset_range: ByteRange,
        /// Boundary after the weak consonant in the source partition.
        boundary: usize,
    },
}
