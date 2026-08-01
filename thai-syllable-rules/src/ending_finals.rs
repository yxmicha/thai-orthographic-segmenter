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

impl CharacterSequence<'_> {
    /// Takes the finals that a tone-marked nucleus, a carrier, an inherent-final vowel, a taikhu
    /// carrier, or a completed ือ carrier proves local. Returns whether the construction has an
    /// explicit hard vowel ending (ะ, ำ, or เ...า).
    pub(crate) fn consume_local_finals(
        self,
        start: usize,
        scan: &mut EndingScan,
        context: &HardEndingContext,
        evidence: &HardEndingEvidence,
    ) -> bool {
        let characters = self.0;
        // Rule: an inherent-final ใ/ไ form is a certain hard boundary only when
        // neither a garan suffix nor an unmarked ย remains at the boundary. A
        // garan suffix provides a positive silent-mark continuation, while bare ย
        // can still attach to the preceding spelling or begin the following one.
        // Examples: `ไกกะ` closes `ไก` before `กะ`; `ไตรย์กะ` stays open for its
        // garan spelling; and `ไกยกะ` exposes `ย` as `Undecided` instead of
        // pretending that `ไก | ย` is certain.
        let has_unresolved_bare_inherent_final_y = characters
            .get(scan.index)
            .is_some_and(|value| value.character == '\u{0e22}') // ย
            && !characters
                .get(scan.index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_dependent());
        let has_inherent_final_leading_rime = evidence.starts_with_inherent_final_leading_vowel
            && !self.has_garan_suffix(scan.index)
            && !has_unresolved_bare_inherent_final_y;

        if evidence.can_consume_tone_marked_final {
            // Rule: a tone-marked implicit nucleus may take one ordinary final
            // consonant, just like the equivalent vowel-led closed shapes. Do not
            // take it when lookahead identifies a following rime, cluster, or
            // carrier. Examples: `ส่ง` keeps `ง` with `ส่`, while `ก่กา` keeps
            // `กา` as the next candidate and `ก่กรา` keeps `กร` as its onset.
            scan.index += 1;
            scan.has_hard_ending = true;
            scan.consumed_hard_final = true;
        }

        if evidence.can_consume_carrier_final {
            // Rule: a tone-marked carrier may consume its ordinary final when the
            // current base cannot itself begin a following candidate and the later
            // cluster has its own rime. Examples: `น้องหงส์` becomes
            // `น้อง | หงส์` because the tone mark anchors `น้อง` and `หงส์` has
            // a garan rime; a bare following cluster such as `ล่องหน` keeps
            // `ง` unresolved. Unmarked `ลอกหนัง` and `ทอดกรอบ` remain ambiguous
            // through the same attachment evidence.
            scan.index += 1;
            scan.has_hard_ending = true;
            scan.consumed_hard_final = true;
        }

        // Rule: ใ-/ไ-, independent vowels, and a completed ือ carrier carry an
        // inherent final or complete nucleus and therefore close the atom.
        // Examples: `ใจกะ`, `ไตรกะ`, `ฤกะ`, and `มือกะ` can close as `ใจ | กะ`,
        // `ไตร | กะ`, `ฤ | กะ`, and `มือ | กะ`. Their complete rimes prevent the
        // following bare consonant from being absorbed backward.
        scan.has_hard_ending |= has_inherent_final_leading_rime
            || context.has_independent_vowel
            || context.consumed_ue_carrier;

        let has_explicit_hard_vowel_ending =
            self.rime_has_explicit_hard_vowel(start, scan.index) || evidence.has_e_ao_pattern;
        if has_explicit_hard_vowel_ending {
            // Rule: ะ, ำ, and the เ...า pattern are explicit hard vowel endings.
            // Examples: `กะตกลม`, `กำกะ`, and `เกากะ` close their first spelling
            // before the following material. In particular, `เกา` is closed even
            // though its characters include a leading vowel and a long `า`.
            scan.has_hard_ending = true;
        }

        let taikhu_carrier_final_is_locally_owned = evidence.has_taikhu
            && characters
                .get(scan.index)
                .is_some_and(|value| ThaiScalar(value.character).is_vowel_carrier())
            && characters
                .get(scan.index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_base())
            && !self.starts_syllable_after_final(scan.index + 1);
        if taikhu_carrier_final_is_locally_owned {
            // Rule: taikhu + อ + base is a closed spelling only when the base
            // cannot begin a following candidate. This is the same certainty
            // requirement as every other final-coda attachment: `ก็อด` may keep
            // its `ด` when nothing follows it, but `บล็อกหยุด` must expose `ก`
            // between `บล็อ` and the following `หย` cluster as `Undecided`.
            scan.index += 1;
        }

        if context.consumed_ue_carrier
            && AttachmentEvidence::ordinary_final(characters, scan.index, LocalClosure::OptionalCoda)
                == AttachmentEvidence::LeftOnly
            && !self.contains_character(start, scan.index, '\u{0e30}') // ะ
            && !self.contains_character(start, scan.index, '\u{0e33}') // ำ
            && !self.starts_syllable_after_final(scan.index)
        {
            // Rule: a final base after ือ is allowed only when lookahead does not
            // identify a new syllable, and then it closes the current syllable.
            // Examples: `คืน` may consume its final `น`, while `เป็นอยู่` keeps
            // the `อ` carrier and its following rime available so the result is
            // `เป็น | อยู่` rather than an overlong first part.
            scan.index += 1;
            scan.has_hard_ending = true;
            scan.consumed_hard_final = true;
        }
        has_explicit_hard_vowel_ending
    }

    /// Takes one ordinary final consonant after the vowel shapes that call for it. Returns
    /// whether the rime itself proves that the final belongs on the left.
    pub(crate) fn consume_ordinary_final(
        self,
        scan: &mut EndingScan,
        context: &HardEndingContext,
        evidence: &HardEndingEvidence,
        has_explicit_hard_vowel_ending: bool,
    ) -> bool {
        let characters = self.0;
        let can_consume_ordinary_final = !has_explicit_hard_vowel_ending
            && (evidence.has_taikhu
                || evidence.has_ue_rime
                || evidence.has_preposed_i
                || evidence.has_short_i
                || evidence.can_consume_short_u_final)
            && (!context.has_glide || !self.starts_following_implicit_syllable(scan.index));

        let taikhu_final_is_locally_owned = evidence.has_taikhu
            && (!context.has_carrier || !evidence.taikhu_final_precedes_following_cluster);
        let local_ending_proves_ownership = taikhu_final_is_locally_owned
                // A tone-marked ื rime can still leave its final at the boundary
                // before a following candidate, as in `พื้นที่` -> `พื้ | ?น | ที่`.
                // The unmarked `คืน` shape retains its required local final.
                || (evidence.has_ue_rime && !context.has_tone_mark)
                || evidence.has_preposed_i;
        // Rule: required local rimes may consume an eligible coda even when a
        // bare continuation could be resegmented, because the rime itself proves
        // that the coda completes the spelling. Open carriers do not enter this
        // set; their coda must pass the full following-candidate test above.
        // Examples: `เป็นคน` keeps `น` in `เป็น`. A taikhu carrier rime follows
        // the same ownership rule;
        // this preserves `บล็อ | ?ก | หยุด` instead of committing `ก` to
        // `บล็อก`. A non-carrier taikhu rime such as `เต็ม` retains its eligible
        // final through the ordinary hard-rime path.
        // An ordinary short-ิ rime does not prove that its final belongs on the
        // left when another Thai candidate follows. The final may be the onset
        // boundary material of the next construction, so the resolver must keep
        // it explicit. For short-ิ forms, the preposed เ...ิ spelling is hard by
        // local structure; the base-led `กิ` in `กินซัก` therefore becomes
        // `กิ | ?น | ซัก`, while terminal `กิน` still consumes `น` because no
        // right-hand candidate exists.
        // Carrier-led cases such as `วิฟอยู่` and `กลิ่นอับ` follow the same
        // conservative rule, and open carriers such as `ทอดกรอบ` still leave
        // `ด` unresolved between `ทอ` and the next cluster.
        let closure = if local_ending_proves_ownership {
            LocalClosure::RequiredCoda
        } else {
            LocalClosure::OptionalCoda
        };
        let can_consume_final = AttachmentEvidence::ordinary_final(characters, scan.index, closure)
            == AttachmentEvidence::LeftOnly;
        if can_consume_ordinary_final && can_consume_final {
            // Rule: these vowel shapes take one ordinary final consonant. A base
            // followed by a dependent mark is left for the next construction.
            // Examples: `กัด`, `เด็ก`, `แข็ง`, `น็อต`, `คืน`, and `เดิน` each
            // consume their local final when the spelling calls for it. If the
            // next base has its own mark, as in `...กะ...`, it remains a following
            // candidate instead of becoming a second final.
            scan.index += 1;
            scan.has_hard_ending = true;
            scan.consumed_hard_final = true;
        }
        local_ending_proves_ownership
    }

    /// Takes the garan that follows an already consumed short-ั coda.
    fn consume_short_coda_garan(self, scan: &mut EndingScan, context: &HardEndingContext) {
        if context.consumed_short_a_coda
            && !scan.consumed_hard_final
            && self.has_immediate_garan_suffix(scan.index)
        {
            // Rule: a garan immediately after an already consumed short-ั coda
            // belongs to that final spelling even when no extra base is consumed.
            // Example: `สัตว์` keeps the garan mark with the consumed `ัต`-style
            // ending; it does not produce `สัต | ว์` or a standalone `์` part.
            scan.consumed_hard_final = true;
        }

        if context.consumed_short_a_coda
            && !scan.consumed_hard_final
            && self.has_garan_suffix(scan.index)
            && !self.starts_following_initial_cluster(scan.index)
            && self
                .consume_garan_extension(&mut scan.index, false)
                .is_some()
        {
            // Rule: a non-cluster garan suffix may extend a short-ั ending when
            // the suffix does not look like the start of an initial cluster.
            // Examples: `บัตร` and `สัตว์` can retain their garan-bearing endings,
            // while a following cluster-shaped onset is left for the next
            // syllable so the extension cannot swallow a legitimate boundary.
            scan.has_hard_ending = true;
            scan.consumed_hard_final = true;
        }
    }

    /// Takes a final ร extension after a short-ั, short-ิ or short-ุ ending that has a vowel rime.
    fn consume_final_r(
        self,
        start: usize,
        scan: &mut EndingScan,
        context: &HardEndingContext,
        evidence: &HardEndingEvidence,
    ) {
        let characters = self.0;
        let consumed_base_is_glide = scan.consumed_hard_final
            && characters
                .get(scan.index - 1)
                .is_some_and(|value| ThaiScalar(value.character).is_glide_base());
        if (context.consumed_short_a_coda || evidence.has_short_i || evidence.can_consume_short_u_final)
            && !consumed_base_is_glide
            // Rule: final ร extensions require an actual vowel rime before the final.
            // Examples: `ฟอร์ม` and `บัตร` have a preceding vowel-bearing structure;
            // a bare run shaped only like `กร` has no such evidence and leaves its
            // bases to cluster or ambiguity handling instead.
            && characters[start..scan.index]
                .iter()
                .any(|value| ThaiScalar(value.character).is_vowel())
            && let Some(extension) = self.consume_final_r_extension(
&mut scan.index,
                context.consumed_short_a_coda || scan.consumed_hard_final,
            )
        {
            // Rule: a final ร is a permitted rime extension after these vowel
            // forms; retain whether it is a single base or a garan cluster.
            // Examples: `บัตร` admits a final `ร`-extension shape, and `จันทร์`
            // keeps the `ร` plus garan sequence with the preceding rime. A bare
            // consonant run with no preceding vowel does not qualify merely
            // because it contains `ร`. When a short-ั or already hard ending has
            // established the rime, the final ร is bound only when the supported
            // final-ร cluster proves ownership and no competing following
            // candidate exists. Thus `จักรยาน` leaves `ร` available to both
            // neighboring constructions, while terminal `จักร` can retain it.
            // An explicit following rime still wins: `อิสรอ` is `อิส | รอ`,
            // because the open carrier proves that `ร` can be the next syllable's
            // onset.
            scan.has_hard_ending = true;
            scan.consumed_hard_final = true;
            scan.final_r_extension = Some(extension);
        }
    }

    /// Takes a garan sequence that follows a consumed hard final, within its trailing-base policy.
    fn consume_trailing_garan(
        self,
        scan: &mut EndingScan,
        context: &HardEndingContext,
        evidence: &HardEndingEvidence,
        local_ending_proves_ownership: bool,
    ) {
        if scan.consumed_hard_final
            && !scan.consumed_garan_extension
            // A new initial cluster after a consumed final belongs to the next
            // spelling; do not absorb that whole cluster as a garan extension.
            && !self.starts_following_initial_cluster(scan.index)
            && let Some(trailing_attachment) = self.consume_garan_extension(&mut scan.index,
                match scan.final_r_extension {
                    Some(FinalRExtension::Single) => context.allow_ambiguous_garan_trailing_base,
                    Some(FinalRExtension::Cluster) => false,
                    None => true,
                },)
        {
            // Only a consumed base that is itself silenced frees the ordinary
            // coda slot. A short-ั coda or a final-ร extension retains its prior
            // pronounced coda, even though another silent suffix follows it.
            scan.deferred_silent_coda = trailing_attachment == AttachmentEvidence::Undecided
                && !context.consumed_short_a_coda
                && !evidence.has_tone_marked_implicit_nucleus
                && !local_ending_proves_ownership
                && scan.final_r_extension.is_none();
            // Rule: consume a following garan sequence only when its trailing-base
            // policy allows it; final clusters are more restrictive than singles.
            // Examples: `ฟอร์ม` may keep its local silent-mark sequence, while a
            // final cluster followed by a new onset is not extended arbitrarily.
            // This is why `การ์ดา` can stop before `ดา` even though `การ์` is
            // itself opaque.
            scan.has_hard_ending = true;
        }
    }
}
