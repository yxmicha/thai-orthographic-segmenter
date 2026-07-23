use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
use crate::{RHanPattern, ReadingAlternative};

impl CharacterSequence<'_> {

    /// Adds the alternatives suggested by the scalar at `index` and the ones after it.
    fn push_adjacent_alternatives(
        self,
        index: usize,
        r_han: &[RHanPattern],
        alternatives: &mut Vec<ReadingAlternative>,
    ) {
        let characters = self.0;
        let Some(current) = characters.get(index) else {
            return;
        };
        let Some(next) = characters.get(index + 1) else {
            return;
        };
        if ThaiScalar(current.character).is_coda()
            && ThaiScalar(next.character).is_leading_vowel()
            && index > 0
            && characters
                .get(index + 2)
                .is_some_and(|following| ThaiScalar(following.character).is_base())
        {
            alternatives.push(ReadingAlternative::SharedConsonant {
                source_range: current.start..current.end,
                boundary: current.end,
            });
        }
        if !ThaiScalar(current.character).is_base() || !ThaiScalar(next.character).is_base() {
            return;
        }
        if r_han
            .iter()
            .any(|pattern| pattern.nucleus_range.contains(&current.start))
        {
            return;
        }
        let following_shape = characters.get(index + 2).is_some_and(|following| {
            (ThaiScalar(following.character).is_dependent()
                && !matches!(following.character, '์' | 'ฺ' | '๎'))
                || ThaiScalar(following.character).is_coda()
                || following.character == 'อ'
        });
        if following_shape {
            alternatives.push(ReadingAlternative::WeakInitial {
                source_range: current.start..current.end,
                boundary: current.end,
            });
            if let Some(previous) = index
                .checked_sub(1)
                .and_then(|position| characters.get(position))
                && ThaiScalar(previous.character).is_leading_vowel()
            {
                alternatives.push(ReadingAlternative::LeadingVowelScope {
                    vowel_range: previous.start..previous.end,
                    consonant_range: current.start..current.end,
                    following_onset_range: next.start..next.end,
                    boundary: current.end,
                });
            }
        }
        if ThaiScalar(current.character).is_coda()
            && following_shape
            && index > 0
            && !characters
                .get(index - 1)
                .is_some_and(|previous| ThaiScalar(previous.character).is_leading_vowel())
        {
            alternatives.push(ReadingAlternative::SharedConsonant {
                source_range: current.start..current.end,
                boundary: current.end,
            });
        }
    }
}
