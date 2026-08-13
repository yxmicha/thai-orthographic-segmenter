use std::ops::Range;
use crate::character::ThaiScalar;
use crate::resolution::ResolutionState;

/// A maximal run of bare atoms and the facts about its neighbors that decide its ownership.
#[derive(Clone)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts of one scan, each tested on its own"
)]
pub(crate) struct BareRun {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) range: Range<usize>,
    pub(crate) run_has_silent_mark: bool,
    pub(crate) has_previous_syllable: bool,
    pub(crate) has_following_syllable: bool,
    pub(crate) previous_has_hard_ending: bool,
    pub(crate) is_terminal_singleton_before_dot: bool,
    pub(crate) contains_orphan_dependent_mark: bool,
    pub(crate) contains_orphan_vowel_mark: bool,
    pub(crate) previous_ends_in_base: bool,
    pub(crate) is_single_vowel_carrier: bool,
}

impl ResolutionState<'_> {

    /// Preserves a run holding orphan dependent marks or vowels as text or undecided parts; `None` when the run has none.
    fn resolve_malformed_run(&mut self, run: &BareRun) -> Option<usize> {
        let BareRun {
            start,
            end: index,
            range,
            previous_has_hard_ending,
            contains_orphan_dependent_mark,
            contains_orphan_vowel_mark,
            previous_ends_in_base,
            ..
        } = run.clone();
        if contains_orphan_vowel_mark && (!previous_has_hard_ending || !previous_ends_in_base) {
            // Rule: a second dependent vowel without a base is malformed source
            // material, not a continuation of the preceding spoken-vowel
            // pattern. Preserve it as Text instead of creating a multi-vowel
            // Syllable or an orphan public syllable.
            // Examples: `กีา` becomes `กี | Text("า")`, while a valid carrier
            // such as `มือ` is consumed before this check and remains one
            // syllable. A mark after a hard final base, such as the `ิ` in
            // `บัติ`, still follows the post-final handling below; a mark
            // immediately after a hard vowel, such as `กะา`, remains Text.
            self.ownership.push_text(range);
            return Some(index);
        }

        if contains_orphan_dependent_mark {
            if previous_has_hard_ending {
                // Rule: a dependent mark after a closed atom is a post-final
                // spelling continuation, not the beginning of a malformed
                // mark-first syllable. Keep it with the closed spelling.
                // Example: if a hard `กัด`-style atom is followed by a mark,
                // the mark remains in the preceding range rather than making
                // a public `ิ` or `์` part at the next position.
                if self.ownership.extend_last_syllable(range.end) {
                    return Some(index);
                }
            }

            // A later base cannot license a mark written before it.
            // Preserve malformed source instead of repairing its order.
            self.ownership.push_text(range);
            return Some(index);
        }

        if self.atoms()[start..index]
            .iter()
            .any(crate::atom::ThaiAtom::has_phinthu)
        {
            // Ordinary implicit-vowel completion does not establish the
            // reading of a consonant sequence containing a virama.
            for atom_index in start..index {
                self.ownership
                    .push_undecided(self.ownership.atom_absolute_range(atom_index));
            }
            return Some(index);
        }
        None
    }

    /// Resolves a run that starts with a vowel carrier, precedes a garan carrier, or follows a hard ending; `None` when none applies.
    fn resolve_carrier_run(&mut self, run: &BareRun) -> Option<usize> {
        let BareRun {
            start,
            end: mut index,
            range,
            has_following_syllable,
            previous_has_hard_ending,
            is_terminal_singleton_before_dot,
            is_single_vowel_carrier,
            ..
        } = run.clone();
        let starts_with_vowel_carrier = self.atoms()[start]
            .first_base()
            .is_some_and(|character| ThaiScalar(character).is_vowel_carrier());
        if has_following_syllable && starts_with_vowel_carrier {
            if start > 0
                && self.atoms()[start - 1].starts_with_leading_vowel()
                && !self.atoms()[start - 1].has_hard_ending()
            {
                if is_single_vowel_carrier {
                    // Rule: a singleton อ after an open leading-vowel atom
                    // remains unresolved when a later nucleus-bearing atom
                    // follows. The carrier may complete the left spelling or
                    // begin an implicit-vowel spelling on the right, so
                    // attaching it to either side would exceed certainty.
                    // Examples: `เจอแบ` becomes `เจ | ?อ | แบ`, while the
                    // terminal `เจอ` still merges its carrier into `เจ` below
                    // because no right-hand nucleus can claim `อ`.
                    self.ownership
                        .push_undecided(self.ownership.atom_absolute_range(start));
                    index = start + 1;
                    return Some(index);
                }

                // Rule: a multi-scalar carrier construction after a
                // leading-vowel atom completes that atom's spelling when
                // its own structure already proves the attachment. The
                // singleton ambiguity above is deliberately handled first.
                // Examples: a carrier with its own base remains a complete
                // candidate such as `เยอร...`; only its bases are exposed
                // when the carrier construction itself is structurally
                // ambiguous.
                let end = self.ownership.atom_absolute_range(start).end;
                if self.ownership.extend_last_syllable(end) {
                    index = start + 1;
                    return Some(index);
                }
            }

            if is_single_vowel_carrier && self.atoms()[index].has_explicit_rime() {
                // Rule: a bare carrier between syllables belongs to the
                // following explicit-rime construction, as in ขออนุ. It is
                // not a second vowel attached to the preceding ขอ.
                // Example: `ขออนุ` keeps the carrier boundary visible as the
                // next `อนุ` construction instead of treating `อ` as a second
                // nucleus inside `ขอ`.
                self.ownership.adopt_atom_prefix(index, start);
                return Some(index);
            }
        }

        let following_is_garan_carrier = has_following_syllable
            && self.atoms()[index]
                .first_base()
                .is_some_and(|character| ThaiScalar(character).is_vowel_carrier())
            && self.atoms()[index].has_nucleus()
            && self.atoms()[index].has_silent_mark();
        if following_is_garan_carrier && self.atoms()[start..index].len() == 1 {
            // Rule: a bare onset immediately before an อ-carrier with garan
            // belongs to that spelling; do not emit the invalid `อ...์`
            // suffix without its onset.
            // Example: in a shape such as `กอ...์`, the preceding bare onset
            // is carried into the following silent-mark spelling; output like
            // `ก | อ...์` would leave a garan construction with no onset.
            self.ownership.adopt_atom_prefix(index, start);
            return Some(index);
        }

        if has_following_syllable
            && index - start == 1
            && self.atoms()[start].has_tone_mark
            && !self.atoms()[start].has_hard_ending()
        {
            // Rule: retain the legacy onset-modifier path only for a
            // tone-marked atom that did not form an implicit nucleus. Ordinary
            // base + tone self.atoms are made hard and bypass this branch.
            // Example: a malformed tone-marked modifier before `นา` can still
            // attach forward, while valid `ก้` in `นะก้กา` remains its own
            // closed candidate.
            self.ownership.adopt_atom_prefix(index, start);
            return Some(index);
        }

        if previous_has_hard_ending {
            self.resolve_hard_ending_run(
                start,
                index,
                range,
                has_following_syllable,
                is_terminal_singleton_before_dot,
            );
            return Some(index);
        }
        None
    }
}
