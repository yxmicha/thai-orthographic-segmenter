use crate::attachment_evidence::AttachmentEvidence;
use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
use crate::ending::FinalRExtension;

impl CharacterSequence<'_> {
    pub(crate) fn consume_final_r_extension(
        self,
        index: &mut usize,
        final_r_is_bound: bool,
    ) -> Option<FinalRExtension> {
        let characters = self.0;
        // Rule: allow a single final ร or a base + ร + garan cluster as a rime
        // extension; unrelated consonants remain available to resolution.
        // Examples: `บัตร` uses the single-final shape, while `จันทร์` uses the
        // base-plus-`ร`-plus-garan shape. In `การ์ดา`, the `ร` belongs to `การ์`
        // but `ดา` remains a separate following nucleus.
        let first = characters.get(*index)?;
        let following_leading_vowel = self.starts_following_leading_vowel_syllable(*index);
        let following_leading_vowel_after_r =
            self.starts_following_leading_vowel_syllable(*index + 1);
        let following_open_carrier = self.starts_following_open_carrier_syllable(*index);
        let following_other_candidate = self.starts_following_syllable(*index)
            && !following_leading_vowel
            && !following_open_carrier;
        let following_syllable_after_r =
            self.starts_syllable_after_final(*index) || following_leading_vowel_after_r;
        let preceded_by_supported_final_r_cluster = (*index > 0).then(|| {
            characters.get(*index - 1).is_some_and(|value| {
                ThaiScalar(value.character).forms_final_r_cluster_with(ThaiScalar(first.character))
            })
        }) == Some(true);
        // A supported `กร`/`ตร`/`ทร`/`สร` shape permits final-ร attachment, but it
        // does not prove that `ร` belongs to the left syllable when a following
        // syllable can start immediately after it. The pair is also a valid onset
        // shape, so `จักรยาน` must remain `จัก | ?ร | ยาน`; the final-ร ownership
        // becomes certain only when no following candidate remains possible.
        let supported_final_r_is_unambiguous =
            preceded_by_supported_final_r_cluster && !following_syllable_after_r;
        if ThaiScalar(first.character).is_final_r()
            && (!characters
                .get(*index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_dependent())
                || characters
                    .get(*index + 1)
                    .is_some_and(|value| ThaiScalar(value.character).is_garan_mark()))
        // อ์
            && (characters
                .get(*index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_garan_mark())
                || (!following_open_carrier
                    && (supported_final_r_is_unambiguous
                        || (!following_syllable_after_r
                            && (final_r_is_bound || !following_other_candidate)
                            && (final_r_is_bound
                                || (!following_leading_vowel
                                    && !self.starts_following_implicit_syllable(*index)))))))
        {
            *index += 1;
            return Some(FinalRExtension::Single);
        }

        let second = characters.get(*index + 1)?;
        if ThaiScalar(first.character).is_base()
            && ThaiScalar(second.character).is_final_r()
            && characters
                .get(*index + 2)
                .is_some_and(|value| ThaiScalar(value.character).is_garan_mark())
        // อ์
        {
            *index += 2;
            return Some(FinalRExtension::Cluster);
        }

        None
    }

    pub(crate) fn consume_garan_extension(
        self,
        index: &mut usize,
        allow_trailing_base: bool,
    ) -> Option<AttachmentEvidence> {
        let characters = self.0;
        // Rule: a garan mark silences the bases immediately before it. A trailing
        // base is optional and is admitted only by the shape policy supplied by
        // the caller.
        // Examples: `ฟิล์ม` and `โฟล์ค` silence their local bases; a trailing base
        // is retained only when lookahead says it is part of that spelling. This
        // prevents `การ์ดา` from becoming one opaque range containing `ดา`.
        let mut probe = *index;
        let garan_follows_final = characters
            .get(probe)
            .is_some_and(|value| ThaiScalar(value.character).is_dependent());

        if garan_follows_final {
            let mark_start = probe;
            let mut mark_has_garan = false;
            while characters
                .get(probe)
                .is_some_and(|value| ThaiScalar(value.character).is_dependent())
            {
                mark_has_garan |= ThaiScalar(characters[probe].character).is_garan_mark();
                probe += 1;
            }
            if mark_start != probe && mark_has_garan {
                let attachment = if allow_trailing_base {
                    AttachmentEvidence::after_silent_mark(characters, probe)
                } else {
                    AttachmentEvidence::Ineligible
                };
                if attachment == AttachmentEvidence::LeftOnly {
                    // Rule: a base after garan belongs to the same spelling only
                    // when lookahead does not identify a following syllable.
                    // Examples: `ฟอร์มและ` may retain the trailing `ม` with the
                    // garan spelling, while `การ์ดา` leaves `ดา` outside because
                    // its dependent rime proves a new syllable starts there.
                    probe += 1;
                }
                *index = probe;
                return Some(attachment);
            }
            return None;
        }

        while characters
            .get(probe)
            .is_some_and(|value| ThaiScalar(value.character).is_base())
        {
            probe += 1;
        }
        let mark_start = probe;
        let mut mark_has_garan = false;
        while characters
            .get(probe)
            .is_some_and(|value| ThaiScalar(value.character).is_dependent())
        {
            mark_has_garan |= ThaiScalar(characters[probe].character).is_garan_mark();
            probe += 1;
        }
        if !mark_has_garan || mark_start == probe {
            return None;
        }

        *index = probe;
        Some(AttachmentEvidence::Ineligible)
    }
}
