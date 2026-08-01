use crate::character_sequence::CharacterSequence;
use crate::ending::{
    CodaEvidence, EndingClosure, EndingDecision, EndingExtension, HardEndingContext,
};
use crate::ending_finals::EndingScan;

// The helpers in this module implement hard syllable endings and final
// extensions and silent marks. They report
// structural evidence to atomization; they do not choose lexical boundaries.
// Example: these helpers can mark `กำ` as closed and `ฟิล์ม` as an opaque
// silent-mark spelling, but `resolution` still decides how an adjacent bare
// consonant is exposed between neighboring candidates.
impl CharacterSequence<'_> {
    pub(crate) fn consume_hard_ending(
        self,
        start: usize,
        index: usize,
        context: HardEndingContext,
    ) -> EndingDecision {
        let mut index = index;
        let index = &mut index;
        if self.is_standalone_ko_taikhu(start, *index) {
            // Rule: `ก็` is the one bare taikhu spelling that is itself a
            // complete syllable. Keep its boundary before a following carrier
            // or consonant; a taikhu pattern that began with a leading vowel or
            // another local rime does not receive this exception.
            // Examples: `ก็รอ` is `ก็ | รอ`, `เธอก็ไม่` is `เธอ | ก็ | ไม่`,
            // while `เก็บ` remains one pattern because its vowel began at `เ`.
            return EndingDecision {
                end: *index,
                closure: EndingClosure::Hard,
                coda: CodaEvidence::None,
                extension: EndingExtension::None,
                deferred_silent_coda: false,
            };
        }
        let evidence = self.hard_ending_evidence(start, *index, &context);
        let mut scan = EndingScan::new(*index, &context, &evidence);
        let has_explicit_hard_vowel_ending =
            self.consume_local_finals(start, &mut scan, &context, &evidence);
        let local_ending_proves_ownership = self.consume_ordinary_final(
            &mut scan,
            &context,
            &evidence,
            has_explicit_hard_vowel_ending,
        );
        self.consume_ending_extensions(
            start,
            &mut scan,
            &context,
            &evidence,
            local_ending_proves_ownership,
        );
        let EndingScan {
            index,
            has_hard_ending,
            consumed_hard_final,
            final_r_extension,
            consumed_garan_extension,
            deferred_silent_coda,
        } = scan;
        let coda = if context.consumed_short_a_coda {
            CodaEvidence::ShortA
        } else if evidence.has_tone_marked_implicit_nucleus && consumed_hard_final {
            CodaEvidence::ToneMarkedImplicit
        } else if context.consumed_ue_carrier && consumed_hard_final {
            CodaEvidence::CarrierFinal
        } else if consumed_hard_final {
            CodaEvidence::OrdinaryFinal
        } else {
            CodaEvidence::None
        };
        let extension = final_r_extension.map_or_else(
            || {
                if consumed_garan_extension {
                    EndingExtension::Garan
                } else {
                    EndingExtension::None
                }
            },
            EndingExtension::FinalR,
        );
        EndingDecision {
            end: index,
            closure: if has_hard_ending {
                EndingClosure::Hard
            } else {
                EndingClosure::Open
            },
            coda,
            extension,
            deferred_silent_coda,
        }
    }

    pub(crate) fn is_standalone_ko_taikhu(self, start: usize, end: usize) -> bool {
        let characters = self.0;
        // Rule: recognize exactly the independent `ก็` spelling: ก (U+0E01)
        // followed by taikhu ็ (U+0E47), with no preceding vowel pattern or
        // trailing continuation already consumed by atomization.
        characters.get(start..end).is_some_and(|values| {
            values.len() == 2
                && values[0].character == '\u{0e01}' // ก
                && values[1].character == '\u{0e47}' // ็
        })
    }
}
