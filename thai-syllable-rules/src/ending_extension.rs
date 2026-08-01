use crate::attachment_evidence::AttachmentEvidence;
use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;

impl CharacterSequence<'_> {

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
