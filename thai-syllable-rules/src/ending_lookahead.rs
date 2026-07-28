use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;

impl CharacterSequence<'_> {
    pub(crate) fn starts_following_initial_cluster(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: a listed onset pair is evidence that the next base run begins a
        // following syllable rather than extending the current final.
        // Examples: after a closed candidate, `กรา` is recognized as a following
        // onset construction, while an unlisted `กม` remains an attachment
        // question for implicit resolution.
        characters.get(index..index + 2).is_some_and(|values| {
            ThaiScalar(values[0].character)
                .forms_initial_cluster_with(ThaiScalar(values[1].character))
        })
    }

    pub(crate) fn starts_following_open_carrier_syllable(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: a base followed directly by a bare อ is already a vowel-bearing
        // following candidate, even when no final base follows the carrier. If อ
        // is immediately followed by a dependent mark, it is instead the onset's
        // own explicit rime (`อับ`), not an open carrier; short-ิ ownership keeps
        // the preceding final unresolved in that shape.
        // Examples: `ตอ` and `กอ` are complete open carrier syllables; after
        // `เหมือน`, the `ตอ` in `เหมือนตอ` therefore prevents `น` from being
        // silently committed as the preceding final. In `กลิ่นอับ`, the `อั`
        // rime is handled by `starts_carrier_rime_syllable_at`, so `น` remains
        // unresolved. A later final, as in `ตอก`, is covered by the same
        // bare-carrier prefix and remains part of that candidate.
        characters
            .get(index)
            .is_some_and(|value| ThaiScalar(value.character).is_base())
            && characters
                .get(index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_vowel_carrier())
            && !characters
                .get(index + 2)
                .is_some_and(|value| ThaiScalar(value.character).is_dependent())
    }

    pub(crate) fn starts_following_carrier_syllable(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: base + อ + base is a following carrier syllable shape.
        // Examples: `กอน` has the `ก + อ + น` carrier shape after a preceding
        // candidate, while `กะ` has no such three-base lookahead. The test is
        // structural and does not claim that every possible spelling is lexical.
        self.starts_following_open_carrier_syllable(index)
            && characters
                .get(index + 2)
                .is_some_and(|value| ThaiScalar(value.character).is_base())
    }
}
