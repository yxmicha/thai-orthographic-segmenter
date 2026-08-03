use crate::atom::AtomScan;
use crate::atom::ThaiAtom;
use crate::attachment_evidence::AttachmentEvidence;
use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
use crate::construction_scan::ConstructionScan;
use crate::ending::{EndingClosure, HardEndingContext};

/// What the hard-ending decision learned about one construction.
struct EndingOutcome {
    has_tone_mark: bool,
    has_hard_ending: bool,
    deferred_silent_coda: bool,
}

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

    /// Consumes a hard ending and records its tone-mark and closure evidence.
    fn decide_ending(
        self,
        scan: &mut ConstructionScan,
        previous: Option<&ThaiAtom>,
    ) -> EndingOutcome {
        let characters = self.0;
        let start = scan.start;
        let has_tone_mark = characters[start..scan.index]
            .iter()
            .any(|value| ThaiScalar(value.character).is_tone_mark());
        let ending_decision = self.consume_hard_ending(
            start,
            scan.index,
            HardEndingContext {
                has_leading_vowel: scan.has_leading_vowel,
                has_independent_vowel: scan.has_independent_vowel,
                has_nucleus: scan.has_nucleus,
                has_carrier: scan.has_carrier,
                has_tone_mark,
                consumed_short_a_coda: scan.consumed_short_a_coda,
                consumed_short_a_glide: scan.consumed_short_a_glide,
                consumed_ue_carrier: scan.consumed_ue_carrier,
                has_glide: scan.has_glide,
                allow_ambiguous_garan_trailing_base: previous.is_none_or(|atom| !atom.has_glide()),
            },
        );
        scan.index = ending_decision.end;
        let has_hard_ending = matches!(ending_decision.closure, EndingClosure::Hard);
        if matches!(
            ending_decision.coda,
            crate::ending::CodaEvidence::ToneMarkedImplicit
        ) {
            scan.has_nucleus = true;
        }
        let deferred_silent_coda = ending_decision.deferred_silent_coda;
        if self.is_standalone_ko_taikhu(start, scan.index) {
            // Rule: the standalone `ก็` exception is a real nucleus, even
            // though taikhu normally contributes only an implicit rime when
            // it is attached to a larger vowel pattern.
            scan.has_nucleus = true;
        }
        EndingOutcome {
            has_tone_mark,
            has_hard_ending,
            deferred_silent_coda,
        }
    }

    /// Makes a garan-marked construction opaque and admits its trailing silent-mark suffix.
    fn consume_silent_marks(
        self,
        scan: &mut ConstructionScan,
        previous: Option<&ThaiAtom>,
        ending: &mut EndingOutcome,
    ) {
        let characters = self.0;
        let start = scan.start;
        // Rule: a garan mark makes the current structure opaque. Its possible
        // trailing base is admitted only by shape-based lookahead in ending.
        // Examples: `ฟิล์ม` and `เสน่ห์` are opaque silent-mark candidates;
        // `การ์ดา` keeps `ดา` outside the opaque spelling because lookahead
        // finds its own nucleus. The garan mark itself never becomes a public
        // part.
        let consumed_garan_mark = characters[start..scan.index]
            .iter()
            .any(|value| ThaiScalar(value.character).is_garan_mark());
        if !scan.has_nucleus && scan.first_base.is_some() && consumed_garan_mark {
            scan.has_nucleus = true;
            scan.is_opaque = true;
            let allow_ambiguous_trailing_base = previous.is_none_or(|atom| !atom.has_glide());
            ending.deferred_silent_coda = allow_ambiguous_trailing_base
                && previous.is_some_and(|atom| atom.has_nucleus() && !atom.has_hard_ending())
                && AttachmentEvidence::after_silent_mark(characters, scan.index)
                    == AttachmentEvidence::Undecided;
            if self.can_consume_garan_trailing_base(scan.index, allow_ambiguous_trailing_base) {
                scan.index += 1;
            }
        }

        let previous_has_hard_ending = previous.is_some_and(super::atom::ThaiAtom::has_hard_ending);
        // Rule: a consonant run with a garan suffix can form a structural
        // silent-mark candidate, but only when it is not a listed onset
        // cluster or an ambiguous suffix that should remain unresolved.
        // Examples: `กำน์` can be retained as one structural silent-mark
        // candidate, while `กะกเรนทร์` gives the recognized `กร` onset and
        // `กะมเหสักข์` remains available to ambiguity resolution rather than
        // being prematurely flattened into one arbitrary atom.
        let silent_mark_scan = if !scan.has_nucleus && scan.first_base.is_some() {
            self.scan_silent_mark_suffix(scan.index)
        } else {
            None
        };
        let can_start_silent_mark_suffix = silent_mark_scan.is_some_and(|suffix| {
            if previous_has_hard_ending {
                // Rule: a hard-ending predecessor may consume only a compact two- or
                // three-base garan spelling. Examples: `กำน์กะ` and `ฟิล์มและ` qualify;
                // a four-base run remains available to the ordinary ambiguity rules.
                (2..=3).contains(&suffix.base_count) && suffix.has_garan_after_base
            } else {
                let previous_starts_with_leading_vowel =
                    previous.is_some_and(super::atom::ThaiAtom::starts_with_leading_vowel);
                let starts_with_initial_cluster = self.starts_following_initial_cluster(start);
                let silent_mark_shape = self.silent_mark_suffix_shape(start, Some(suffix));
                // Rule: an onset-only base cannot be swallowed into an opaque
                // garan candidate when a later initial cluster supplies the
                // actual onset. In `สหกรณ์`, `ห` cannot close `ส` and cannot
                // begin the `กรณ์` suffix, so the resolver must expose both
                // uncertain bases before the `กร` candidate.
                let onset_only_base_precedes_initial_cluster = characters
                    .get(start)
                    .is_some_and(|value| !ThaiScalar(value.character).is_coda())
                    && silent_mark_shape.precedes_any_initial_cluster;
                let previous_is_unmarked_open_carrier = previous.is_some_and(|atom| {
                    atom.has_carrier()
                        && !atom.has_explicit_rime()
                        && !atom.has_tone_mark
                        && !atom.has_hard_ending()
                });
                let unmarked_carrier_precedes_initial_cluster = previous_is_unmarked_open_carrier
                    && silent_mark_shape.precedes_any_initial_cluster;
                (!starts_with_initial_cluster || silent_mark_shape.starts_at_last_initial_cluster)
                    && !self.has_ambiguous_silent_mark_suffix(start, scan.index, Some(suffix))
                    && !onset_only_base_precedes_initial_cluster
                    && !unmarked_carrier_precedes_initial_cluster
                    && (previous_starts_with_leading_vowel
                        || !silent_mark_shape.precedes_initial_cluster)
            }
        });
        if can_start_silent_mark_suffix
            && self.consume_silent_mark_suffix(
                &mut scan.index,
                previous.is_none_or(|atom| !atom.has_glide()),
                silent_mark_scan,
            )
        {
            scan.has_nucleus = true;
            scan.is_opaque = true;
        }
    }

    /// Settles the nucleus of a tone-marked implicit construction and records the scan facts.
    fn finish_scan(self, scan: &mut ConstructionScan, ending: &EndingOutcome) -> AtomScan {
        let characters = self.0;
        let start = scan.start;
        if !scan.has_nucleus && ending.has_tone_mark && !self.starts_carrier_syllable_at(scan.index)
        {
            // Rule: a base plus an ordinary tone mark is an implicit-vowel
            // atom and must not be reclassified as an onset modifier for the
            // next rime. Resolution later marks the atom `Invalid` when it is
            // between two syllables and has no vowel or final base.
            // Examples: `ก่`, `ก้`, `ก๊`, and `ก๋` stay bounded before `กา`,
            // while `นะก้กา` exposes the middle atom as Invalid. The `ป้` in
            // `ป้อง` is excluded because its following `อ+ง` carrier completes
            // the same tone-marked syllable.
            scan.has_nucleus = true;
        }

        let end = scan.index.max(start + 1);
        // AtomScan is the hand-off between structural scanning and semantic
        // atom construction. The scanner owns only source-local facts; the
        // atom module owns their interpretation as onset, nucleus, ending,
        // and silent-mark shapes.
        AtomScan {
            range: characters[start].start..characters[end - 1].end,
            character: scan.character,
            first_base: scan.first_base,
            has_nucleus: scan.has_nucleus,
            has_vowel_rime: scan.has_vowel_rime,
            has_ue_rime: scan.has_ue_rime,
            has_independent_vowel: scan.has_independent_vowel,
            has_tone_mark: ending.has_tone_mark,
            has_hard_ending: ending.has_hard_ending,
            deferred_silent_coda: ending.deferred_silent_coda,
            is_opaque: scan.is_opaque,
            has_leading_vowel: scan.has_leading_vowel,
            has_carrier: scan.has_carrier,
            has_glide: scan.has_glide,
            consumed_ue_carrier: scan.consumed_ue_carrier,
        }
    }
}
