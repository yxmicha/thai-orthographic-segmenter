use crate::character::ThaiCharacter;
use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
use crate::part::{ByteRange, Part};
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

    /// Refines caller-validated parts at recoverable boundaries. Source ownership
    /// remains disjoint; a รร nucleus is kept together even if flat rules split it.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::{Part, RuleAnalysis, split};
    ///
    /// let source = "เกรียน";
    /// let parts = RuleAnalysis::new(source).recover_parts(&split(source));
    /// // The single syllable is reopened at the weak initial `กร`'s boundary.
    /// assert_eq!(parts, [Part::Undecided(0..6), Part::Undecided(6..18)]);
    /// ```
    #[must_use]
    pub fn recover_parts(&self, parts: &[Part]) -> Vec<Part> {
        let mut boundaries = Vec::new();
        for part in parts {
            let range = part.byte_range();
            boundaries.extend([range.start, range.end]);
        }
        for alternative in &self.alternatives {
            boundaries.push(alternative.boundary());
            if let ReadingAlternative::WeakInitial { source_range, .. } = alternative
                && !self.alternatives.iter().any(|other| matches!(other, ReadingAlternative::LeadingVowelScope { consonant_range, .. } if consonant_range == source_range))
            {
                boundaries.push(source_range.start);
            }
        }
        for pattern in &self.r_han {
            boundaries.extend([pattern.nucleus_range.start, pattern.nucleus_range.end]);
            if let Some(final_range) = &pattern.possible_final {
                boundaries.push(final_range.end);
            }
        }
        boundaries.sort_unstable();
        boundaries.dedup();
        boundaries.retain(|offset| {
            !self.r_han.iter().any(|pattern| {
                pattern.nucleus_range.start < *offset && *offset < pattern.nucleus_range.end
            })
        });
        let mut recovered = Vec::with_capacity(boundaries.len().saturating_sub(1));
        let mut original_index = 0;
        for pair in boundaries.windows(2) {
            let range = pair[0]..pair[1];
            while parts
                .get(original_index)
                .is_some_and(|part| part.byte_range().end <= range.start)
            {
                original_index += 1;
            }
            let Some(first) = parts.get(original_index) else {
                break;
            };
            if first.byte_range() == range {
                recovered.push(first.clone());
                continue;
            }
            recovered.push(match first {
                Part::Syllable(_) | Part::Undecided(_) => Part::Undecided(range),
                Part::Invalid(_) => Part::Invalid(range),
                Part::Dot(_) => Part::Dot(range),
                Part::Text(_) => Part::Text(range),
            });
        }
        recovered
    }
}
