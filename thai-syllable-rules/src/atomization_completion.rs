use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
use crate::construction_scan::ConstructionScan;

impl CharacterSequence<'_> {

    /// Extends the scan over the carrier and leading-vowel spellings of a rime, then its glide.
    fn complete_rime(self, scan: &mut ConstructionScan) {
        let characters = self.0;
        let start = scan.start;
        let ue_carrier_starts_following_rime = characters
            .get(scan.index..scan.index + 2)
            .is_some_and(|values| {
                ThaiScalar(values[0].character).is_vowel_carrier()
                    && ThaiScalar(values[1].character).is_dependent()
            });
        if scan.has_ue_rime
            && !ue_carrier_starts_following_rime
            && characters
                .get(scan.index)
                .is_some_and(|value| ThaiScalar(value.character).is_vowel_carrier())
        {
            // Rule: the อ carrier completes a ือ-style rime before final
            // ending recognition.
            // Examples: `มือ` consumes `อ` after `ื` as one `ือ` rime, and a
            // following final is considered only after that carrier is known.
            // This prevents `มื | อ` from exposing the carrier as a new base.
            scan.index += 1;
            scan.consumed_ue_carrier = true;
            self.consume_dependent_characters(&mut scan.index, &mut scan.has_nucleus, true);
        }

        let has_explicit_hard_vowel_rime = self.rime_has_explicit_hard_vowel(start, scan.index)
            || (characters
                .get(start)
                .is_some_and(|value| value.character == '\u{0e40}') // เ
                && characters[start..scan.index]
                    .iter()
                    .any(|value| value.character == '\u{0e32}')); // า
        if characters[start].character == 'เ'
            && !scan.has_vowel_rime
            && characters
                .get(scan.index)
                .is_some_and(|value| value.character == 'อ')
            && characters
                .get(scan.index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_coda())
            && (scan.index + 2 == characters.len()
                || (characters
                    .get(scan.index + 1)
                    .is_some_and(|value| matches!(value.character, 'ม' | 'ญ'))
                    && characters
                        .get(scan.index + 2)
                        .is_some_and(|value| !ThaiScalar(value.character).is_dependent())))
        {
            // Retained เ-อ before a final (เทอม, เทอญ, เทอมหน้า) supplies
            // one nucleus; อ must not become a second implicit onset.
            scan.index += 1;
            scan.has_carrier = true;
        }
        if scan.has_leading_vowel
            && characters.get(start).is_some_and(|value| {
                value.character == '\u{0e40}' // เ
                        && !ThaiScalar(value.character).is_inherent_final_leading_vowel()
            })
            && !has_explicit_hard_vowel_rime
            && (!scan.has_vowel_rime || scan.has_ue_rime)
            && characters
                .get(scan.index..scan.index + 2)
                .is_some_and(|values| {
                    values[0].character == '\u{0e2d}' // อ
                    && values[1].character == '\u{0e30}' // ะ
                })
        {
            // Rule: a leading-vowel spelling may consume อ plus its rime as a
            // carrier completion, except for the inherent-final ใ/ไ forms.
            // Examples: `เธอ` keeps the trailing carrier spelling with the
            // leading-vowel construction. The special `ใ`/`ไ` forms do not
            // consume an unrelated carrier through this branch because their
            // inherent final is already complete.
            scan.index += 1;
            self.consume_dependent_characters(&mut scan.index, &mut scan.has_nucleus, false);
        }

        self.complete_glide(scan);
    }

    /// Extends the scan over a glide spelling that the rime's vowel pattern permits.
    fn complete_glide(self, scan: &mut ConstructionScan) {
        let characters = self.0;
        let start = scan.start;
        if scan.has_vowel_rime
            && !scan.consumed_short_a_coda
            && characters
                .get(scan.index)
                .is_some_and(|value| ThaiScalar(value.character).is_glide_base())
            && 'glide_vowel_pattern_is_allowed: {
                let glide_index = scan.index;
                let characters = self.0;
                // Rule: a second vowel after ย/ว is part of the same atom only for the
                // recognized vowel-plus-glide patterns; otherwise it starts a new atom.
                // Examples: `เปี๊ยะ`, `เจือะ`, and `ผลัวะ` keep their supported
                // vowel-plus-glide-plus-vowel shape. In contrast, `ระยะ` is resolved as
                // `ระ | ยะ`; the second vowel is not swallowed by the first ordinary
                // base-led glide. The same guard prevents `นาวา` from becoming one atom
                // containing two unrelated vowel patterns.
                let preceding_vowel = characters[start..glide_index]
                    .iter()
                    .rev()
                    .find(|value| ThaiScalar(value.character).is_vowel())
                    .map(|value| value.character);

                if preceding_vowel == Some('\u{0e38}') // อุ
                    && characters
                        .get(glide_index)
                        .is_some_and(|value| value.character == '\u{0e27}')
                // ว
                {
                    // Rule: short อุ + ว is not one of the supported spoken glide
                    // patterns. Leaving ว for final/implicit resolution prevents a
                    // following implicit vowel from being swallowed into this atom.
                    // Example: in a run shaped like `กุวา`, `วา` remains available as the
                    // next candidate instead of forcing the entire run into `กุวา`.
                    break 'glide_vowel_pattern_is_allowed false;
                }

                let Some(next) = characters.get(glide_index + 1) else {
                    break 'glide_vowel_pattern_is_allowed true;
                };

                if !ThaiScalar(next.character).is_vowel() {
                    break 'glide_vowel_pattern_is_allowed true;
                }

                next.character == '\u{0e30}' // ะ
                    && (preceding_vowel == Some('\u{0e31}') // อั
                        || characters
                            .get(start)
                            .is_some_and(|value| ThaiScalar(value.character).is_leading_vowel()))
            }
        {
            // Rule: ย and ว may be a vowel glide only when the local vowel
            // pattern permits it and the following base does not independently
            // start a new syllable candidate. If that lookahead proves a
            // following candidate, leave the glide as a bare atom so
            // resolution can preserve both attachments as `Undecided`.
            // Examples: `เปี๊ยะ`, `เจือะ`, and `ผลัวะ` use the permitted glide
            // patterns, whereas `ระยะ` remains `ระ | ยะ` and `นาวา` is not
            // one atom containing two independent vowel nuclei. In `คราวนี้`,
            // `นี้` proves that `น` starts its own candidate, so `ว` remains
            // between `ครา` and `นี้` as an undecided attachment. The same
            // rule applies to any following consonant after an ordinary glide:
            // `ตายที่` therefore becomes `ตา | ?ย | ที่`, without special-casing
            // either the glide or the following consonant. Leading-vowel and
            // completed-ือ spellings provide positive local ownership for their
            // glide, preserving constructions such as `เจีย` and `เหนื่อย`.
            let glide_has_positive_local_ownership =
                scan.has_leading_vowel || scan.consumed_ue_carrier;
            let following_candidate_starts_after_glide = !glide_has_positive_local_ownership
                && (characters
                    .get(scan.index + 1)
                    .is_some_and(|value| ThaiScalar(value.character).is_base())
                    || self.starts_syllable_after_final(scan.index));
            let glide_starts_tone_marked_syllable = characters
                .get(scan.index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_tone_mark());
            if !following_candidate_starts_after_glide && !glide_starts_tone_marked_syllable {
                scan.index += 1;
                scan.has_glide = true;
                self.consume_dependent_characters(&mut scan.index, &mut scan.has_nucleus, false);
                let glide_has_hard_rime = self.rime_has_explicit_hard_vowel(start, scan.index);
                if !glide_has_hard_rime
                    && characters.get(scan.index).is_some_and(|value| {
                        ThaiScalar(value.character).is_base()
                            && !self.starts_following_syllable(scan.index)
                            && !self.starts_syllable_after_final(scan.index)
                            && !self.starts_following_implicit_syllable(scan.index)
                    })
                {
                    // Rule: a plain base after a consumed glide is a coda
                    // only when lookahead does not identify a following
                    // syllable. Examples: a coda can stay with a glide when
                    // the following base is not a rime or cluster start; in
                    // `...วา...`, a base followed by its own dependent mark
                    // remains available to the next syllable instead of being
                    // swallowed backward.
                    scan.index += 1;
                }
                scan.has_nucleus = true;
                scan.consumed_short_a_glide =
                    self.contains_character(start, scan.index, '\u{0e31}'); // ั
            }
        }
    }
}
