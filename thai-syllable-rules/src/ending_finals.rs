use crate::attachment_evidence::{AttachmentEvidence, LocalClosure};
use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
use crate::ending::{FinalRExtension, HardEndingContext};

/// What a construction's hard-ending decision has consumed and learned so far.
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts of one scan, each tested on its own"
)]
pub(crate) struct EndingScan {
    pub(crate) index: usize,
    pub(crate) has_hard_ending: bool,
    pub(crate) consumed_hard_final: bool,
    pub(crate) final_r_extension: Option<FinalRExtension>,
    pub(crate) consumed_garan_extension: bool,
    pub(crate) deferred_silent_coda: bool,
}

impl EndingScan {
    /// Starts at `index`, already hard when a short-ั coda, a closed short-ั glide, or a
    /// tone-marked implicit nucleus closed the construction.
    pub(crate) fn new(
        index: usize,
        context: &HardEndingContext,
        evidence: &HardEndingEvidence,
    ) -> Self {
        Self {
            index,
            has_hard_ending: context.consumed_short_a_coda
                || context.consumed_short_a_glide
                || evidence.has_tone_marked_implicit_nucleus,
            consumed_hard_final: false,
            final_r_extension: None,
            consumed_garan_extension: false,
            deferred_silent_coda: false,
        }
    }
}

/// Lookahead and shape facts about a construction at the point its hard ending is decided.
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts of one scan, each tested on its own"
)]
pub(crate) struct HardEndingEvidence {
    pub(crate) has_tone_marked_implicit_nucleus: bool,
    pub(crate) can_consume_tone_marked_final: bool,
    pub(crate) has_taikhu: bool,
    pub(crate) taikhu_final_precedes_following_cluster: bool,
    pub(crate) can_consume_carrier_final: bool,
    pub(crate) starts_with_inherent_final_leading_vowel: bool,
    pub(crate) has_e_ao_pattern: bool,
    pub(crate) has_ue_rime: bool,
    pub(crate) has_preposed_i: bool,
    pub(crate) has_short_i: bool,
    pub(crate) can_consume_short_u_final: bool,
}

impl CharacterSequence<'_> {
    /// Gathers the evidence for the ending of the construction spanning `start..index`.
    pub(crate) fn hard_ending_evidence(
        self,
        start: usize,
        index: usize,
        context: &HardEndingContext,
    ) -> HardEndingEvidence {
        let characters = self.0;
        let index = &index;
        // Rule: a short-ั coda, a closed short-ั glide, or a bare base carrying
        // a tone mark is already a hard end. A tone-marked base has an implicit
        // vowel even though it has no dependent vowel scalar of its own.
        // Examples: `กัด` closes after its required `ด`, `ตัว` does not absorb
        // the next syllable's onset, and `ก้กา` closes `ก้` before `กา`. The hard
        // flag is structural evidence for resolution, not a dictionary assertion.
        let has_tone_marked_implicit_nucleus = !context.has_nucleus
            && characters[start..*index]
                .iter()
                .any(|value| ThaiScalar(value.character).is_tone_mark())
            && !self.starts_carrier_syllable_at(*index);
        let can_consume_tone_marked_final = has_tone_marked_implicit_nucleus
            && AttachmentEvidence::ordinary_final(characters, *index, LocalClosure::RequiredCoda)
                == AttachmentEvidence::LeftOnly;
        let has_taikhu = self.contains_character(start, *index, '\u{0e47}'); // อ็
        let taikhu_final_precedes_following_cluster =
            has_taikhu && self.starts_following_initial_cluster(*index + 1);
        let can_consume_carrier_final = context.has_carrier
            // An unmarked open carrier does not prove ownership of its ordinary
            // final before another cluster. A tone mark anchors the carrier's
            // rime to the current onset, so only that marked shape can close the
            // final on this path.
            && context.has_tone_mark
            && !context.consumed_ue_carrier
            // A taikhu carrier rime does not prove that its final belongs on the
            // left when a following initial cluster can claim the boundary. This
            // keeps `บล็อ | ?ก | หยุด` conservative while ordinary carrier forms
            // such as `น้อง | หงส์` retain their certain final.
            && !taikhu_final_precedes_following_cluster
            // An unmarked open carrier can accept a coda, but the carrier itself
            // does not prove that the current base belongs on the left. A tone
            // mark anchors the carrier rime and is the only positive local proof
            // used by this cluster-following path. Without it, lookahead keeps
            // `ลอกหนัง` as `ลอ | ?ก | หนัง` and `ทอดกรอบ` as `ทอ | ?ด | ?ก |
            // รอบ`.
            && AttachmentEvidence::ordinary_final(characters, *index, LocalClosure::ToneMarkedCarrier)
                == AttachmentEvidence::LeftOnly
            && self.starts_following_initial_cluster(*index + 1);
        let starts_with_inherent_final_leading_vowel = context.has_leading_vowel
            && characters
                .get(start)
                .is_some_and(|value| ThaiScalar(value.character).is_inherent_final_leading_vowel());
        let has_e_ao_pattern = context.has_leading_vowel
            && characters
                .get(start)
                .is_some_and(|value| value.character == '\u{0e40}') // เ
            && self.contains_character(start, *index, '\u{0e32}'); // า
        let has_ue_rime =
            !context.consumed_ue_carrier && self.contains_character(start, *index, '\u{0e37}'); // อื
        let has_preposed_i =
            context.has_leading_vowel && self.contains_character(start, *index, '\u{0e34}'); // อิ
        let has_short_i = self.contains_character(start, *index, '\u{0e34}'); // อิ
        let has_short_u = self.contains_character(start, *index, '\u{0e38}'); // อุ
        // Rule: short อุ can consume an eligible coda only when following-shape
        // lookahead proves that the coda cannot instead begin the next candidate.
        // The rime is not itself ownership evidence: `ปทุมธานี` must keep `ม`
        // undecided between `ปทุ` and `ธา`, while terminal `พุทธ` can still close
        // `พุท` because no following nucleus can claim `ท`.
        let can_consume_short_u_final = has_short_u
            && AttachmentEvidence::ordinary_final(characters, *index, LocalClosure::OptionalCoda)
                == AttachmentEvidence::LeftOnly
            && !({
                let index = *index;
                let characters = self.0;
                // Rule: a bare ย/ว followed by another base can begin an implicit
                // syllable; a short-ุ rime must not consume that onset as its final.
                // Examples: in a shape like `กุวา`, the `ว` can begin the following
                // implicit or explicit construction; `สุวรรณ` must not blindly consume
                // the first `ว` as the final of `สุ`.
                characters
                    .get(index)
                    .is_some_and(|value| ThaiScalar(value.character).is_glide_base())
            } && self.starts_following_implicit_syllable(*index))
            && !characters.get(*index..*index + 2).is_some_and(|values| {
                ThaiScalar(values[0].character)
                    .forms_initial_cluster_with(ThaiScalar(values[1].character))
                    && characters
                        .get(*index + 2)
                        .is_some_and(|value| ThaiScalar(value.character).is_dependent())
            });
        HardEndingEvidence {
            has_tone_marked_implicit_nucleus,
            can_consume_tone_marked_final,
            has_taikhu,
            taikhu_final_precedes_following_cluster,
            can_consume_carrier_final,
            starts_with_inherent_final_leading_vowel,
            has_e_ao_pattern,
            has_ue_rime,
            has_preposed_i,
            has_short_i,
            can_consume_short_u_final,
        }
    }
}
