use crate::character::ThaiCharacter;
use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
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
    /// Inspects possible weak initials, consonant sharing, vowel scope, and รร.
    /// All offsets refer to `source`; non-Thai characters terminate the scan.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::RuleAnalysis;
    ///
    /// assert!(RuleAnalysis::new("กะ").reading_alternatives().is_empty());
    /// assert_eq!(RuleAnalysis::new("กะตกลม").reading_alternatives().len(), 4);
    /// ```
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut alternatives = Vec::new();
        let mut r_han = Vec::new();
        let mut run = Vec::new();
        for (start, character) in source.char_indices() {
            if ThaiScalar(character).is_accepted() {
                run.push(ThaiCharacter {
                    character,
                    start,
                    end: start + character.len_utf8(),
                });
            } else {
                let patterns = CharacterSequence(&run).r_han_patterns();
                alternatives.extend(CharacterSequence(&run).reading_alternatives(&patterns));
                r_han.extend(patterns);
                run.clear();
            }
        }
        let patterns = CharacterSequence(&run).r_han_patterns();
        alternatives.extend(CharacterSequence(&run).reading_alternatives(&patterns));
        r_han.extend(patterns);
        Self {
            alternatives,
            r_han,
        }
    }

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

    /// Returns recognized written รร nuclei and possible final consonants.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::RuleAnalysis;
    ///
    /// assert_eq!(RuleAnalysis::new("สรรพ").r_han_patterns().len(), 1);
    /// assert!(RuleAnalysis::new("กะ").r_han_patterns().is_empty());
    /// ```
    #[must_use]
    pub fn r_han_patterns(&self) -> &[RHanPattern] {
        &self.r_han
    }
}
