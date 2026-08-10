use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
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

    /// A possible written coda; its ownership is not chosen here.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::RuleAnalysis;
    ///
    /// let analysis = RuleAnalysis::new("สรรพ");
    /// assert_eq!(analysis.r_han_patterns()[0].possible_final(), Some(&(9..12)));
    /// ```
    #[must_use]
    pub const fn possible_final(&self) -> Option<&ByteRange> {
        self.possible_final.as_ref()
    }
}

impl CharacterSequence<'_> {
    pub(crate) fn r_han_patterns(self) -> Vec<RHanPattern> {
        let characters = self.0;
        let mut patterns = Vec::new();
        let mut index = 0;
        while let Some(window) = characters.get(index..index + 3) {
            if ThaiScalar(window[0].character).is_base()
                && window[0].character != 'ร'
                && window[1].character == 'ร'
                && window[2].character == 'ร'
                && !index
                    .checked_sub(1)
                    .and_then(|position| characters.get(position))
                    .is_some_and(|previous| ThaiScalar(previous.character).is_leading_vowel())
                && characters
                    .get(index + 3)
                    .is_none_or(|next| !ThaiScalar(next.character).is_dependent())
            {
                let possible_final = characters
                    .get(index + 3)
                    .filter(|value| {
                        ThaiScalar(value.character).is_coda()
                            && characters
                                .get(index + 4)
                                .is_none_or(|next| !ThaiScalar(next.character).is_dependent())
                    })
                    .map(|value| value.start..value.end);
                patterns.push(RHanPattern {
                    nucleus_range: window[0].start..window[2].end,
                    possible_final,
                });
                index += 3;
            } else {
                index += 1;
            }
        }
        patterns
    }
}
