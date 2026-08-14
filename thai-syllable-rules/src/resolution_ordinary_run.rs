use crate::atom::SilentMarkShape;
use crate::atom_sequence::AtomSequence;
use crate::character::ThaiScalar;
use crate::part::{ByteRange, Part};
use crate::resolution::ResolutionState;
use crate::resolution_bare_run::BareRun;

impl ResolutionState<'_> {
    /// Resolves a run that the other cases leave: singletons, silent-mark suffixes, and implicit syllables.
    pub(crate) fn resolve_ordinary_run(&mut self, run: &BareRun) -> usize {
        let BareRun {
            start,
            end: mut index,
            range,
            run_has_silent_mark,
            has_previous_syllable,
            has_following_syllable,
            is_terminal_singleton_before_dot,
            is_single_vowel_carrier,
            ..
        } = run.clone();
        if index - start == 1
            && !has_following_syllable
            && self.promote_last_singleton_carrier(&range)
        {
            // Rule: a terminal bare base completes a preceding singleton
            // `อ` carrier when atomization had exposed the carrier as an
            // unresolved scalar. The two scalars are then one implicit
            // candidate, not two standalone parts. Example: the repeated
            // carrier path in `จมรรอก` first exposes `อ`, then this rule
            // completes `อก` after the two ambiguous `ร` atoms.
            return index;
        }

        if has_previous_syllable && (!has_following_syllable || run_has_silent_mark) {
            if is_terminal_singleton_before_dot {
                // Rule: retain the abbreviation-tail exception even when the
                // run is terminal before its period.
                // Example: the final bare singleton in `นะก.` remains
                // `Undecided` immediately before `Dot`, even though the Thai
                // run has no later syllable candidate.
                self.ownership.push_undecided(range);
            } else if !(run_has_silent_mark
                || (is_single_vowel_carrier
                    && start > 0
                    && self.atoms()[start - 1].starts_with_leading_vowel())
                || self.atoms()[start..index].iter().all(|atom| {
                    atom.first_base()
                        .is_some_and(|character| ThaiScalar(character).is_coda())
                }))
            {
                // Exhausting the input does not make an onset-only base a
                // legal coda: นาห retains ห as unresolved source material.
                for atom_index in start..index {
                    self.ownership
                        .push_undecided(self.ownership.atom_absolute_range(atom_index));
                }
            } else if self.ownership.last_is_syllable() {
                self.ownership.extend_last_syllable(range.end);
            }
        } else if has_previous_syllable && has_following_syllable {
            return self.resolve_run_between_syllables(run);
        } else if has_following_syllable {
            if self.atoms()[start].starts_with_leading_vowel()
                && self.atoms()[start].first_base().is_none()
            {
                // Rule: an unpaired leading vowel is malformed Thai source,
                // not a valid syllable prefix. Preserve it as Text instead of
                // hiding it inside a following vowel pattern.
                // Example: the leading `เ` in malformed `เแท` remains `Text`
                // rather than being combined with a later independent vowel;
                // valid `เกา` still follows the normal leading-vowel path.
                let range = self.ownership.atom_absolute_range(start);
                self.ownership.push_text(range);
                index = start + 1;
                return index;
            }
            // Rule: a leading bare prefix that reaches this fallback has no
            // independently proven boundary. Preserve the contiguous
            // candidate here; leading-vowel and onset-only leading runs are
            // handled earlier by `absorb_leading_bases`.
            self.ownership.adopt_atom_prefix(index, start);
        } else {
            // Input exhaustion cannot supply a missing nucleus or license
            // an onset-only final, including after an initial cluster.
            if self.atoms()[index - 1]
                .first_base()
                .is_some_and(|character| ThaiScalar(character).is_coda())
            {
                self.ownership.push_syllable(range);
            } else {
                for atom_index in start..index {
                    self.ownership
                        .push_undecided(self.ownership.atom_absolute_range(atom_index));
                }
            }
        }
        index
    }

    /// Resolves a run between two syllables: repeated carrier onsets, singletons, silent-mark
    /// suffixes, and runs left as undecided atoms. Returns the index after the run.
    fn resolve_run_between_syllables(&mut self, run: &BareRun) -> usize {
        let BareRun {
            start, end: index, ..
        } = run.clone();
        // Rule: with candidates on both sides, preserve attachment
        // ambiguity unless a silent-mark shape provides a stronger merge.
        // Examples: the bare middle of `กะตกลม` stays explicit because
        // both `กะ` and `ลม` can anchor it. A compatible silent-mark suffix
        // such as `ฟิล์ม` uses the stronger opaque-shape rule instead.
        if self.expose_repeated_carrier_onset(start, index)
            || self.move_onset_only_cluster_suffix_forward(start, index)
        {
            return index;
        }
        if index - start == 1
            && !self.can_attach_singleton_as_coda(start)
            && (AtomSequence(&self.atoms()[start..index])
                .following_initial_prefix_start(&self.atoms()[index])
                .is_some()
                || self.atoms()[index]
                    .first_base()
                    .is_some_and(|character| ThaiScalar(character).is_vowel_carrier()))
        {
            // Rule: an onset-only singleton cannot attach as the previous
            // syllable's coda. When it starts a listed cluster or an
            // อ-carrier construction, move it forward instead of exposing
            // an impossible `Undecided` attachment.
            // Examples: `แกหรอก` becomes `แก | หรอก`, while `จมรรอก`
            // is handled by the repeated-carrier
            // rule first and exposes both `ร` bases as undecided.
            self.ownership.adopt_atom_prefix(index, start);
            return index;
        }
        if self.resolve_silent_mark_suffix_run(start, index) {
            return index + 1;
        }
        // Rule: each unresolved atom is public ambiguity, one part per
        // atom, so later resolution can choose its attachment.
        // Examples: a middle run `ตกล` is represented atom-by-atom,
        // allowing later logic to choose `ตก | ลม` or another valid
        // attachment without losing the original character spans.
        for atom_index in start..index {
            self.ownership
                .push_undecided(self.ownership.atom_absolute_range(atom_index));
        }
        index
    }

    /// Completes a preceding singleton carrier with the final coda-capable base of the run.
    fn promote_last_singleton_carrier(&mut self, range: &ByteRange) -> bool {
        let (start, end) = (range.start, range.end);
        let plan = &mut self.ownership;
        let Some(previous) = plan.parts.last().and_then(|part| match part {
            Part::Undecided(range) => Some(range.clone()),
            _ => None,
        }) else {
            return false;
        };
        if previous.end != start
            || !plan
                .single_base_character(&(start..end))
                .is_some_and(|character| ThaiScalar(character).is_coda())
        {
            return false;
        }
        let Some(carrier) = plan.single_base_character(&previous) else {
            return false;
        };
        if !ThaiScalar(carrier).is_vowel_carrier() {
            return false;
        }
        let Some(last) = plan.parts.last_mut() else {
            return false;
        };
        *last = Part::Syllable(previous.start..end);
        true
    }

    /// Resolves the run when a silent-mark suffix atom follows it. Returns whether it did, in which
    /// case the suffix atom is consumed as well.
    fn resolve_silent_mark_suffix_run(&mut self, start: usize, end: usize) -> bool {
        let Some(suffix) = self.atoms().get(end) else {
            return false;
        };
        if !suffix.is_opaque() || suffix.opaque_has_carrier() {
            return false;
        }

        if matches!(
            suffix.silent_mark,
            SilentMarkShape::Opaque {
                has_initial_cluster: true,
                ..
            }
        ) {
            // Rule: an opaque garan candidate with its own initial cluster is a
            // right-hand syllable, not a suffix that borrows the last bare base
            // on the left. Keep the entire intervening run undecided before
            // emitting that candidate.
            // Examples: `หารพรหมจรรย์` becomes `หา | ร? | พ? | ร? | ห? | ม? |
            // จรรย์`; the `จร` onset belongs to `จรรย์`, while the earlier `พร`
            // and `หม` choices remain unresolved.
            for atom_index in start..end {
                self.ownership
                    .push_undecided(self.ownership.atom_absolute_range(atom_index));
            }
            self.ownership
                .push_syllable(self.ownership.atom_absolute_range(end));
            return true;
        }

        if end - start == 2 {
            // Rule: when two bare bases stand between a resolved syllable and a
            // non-carrier garan suffix, both bases may attach across the
            // boundary. Preserve both choices and keep the silent-mark suffix
            // separate.
            // Examples: `แกล้งพจน์` becomes `แกล้ | ง? | พ? | จน์`, while
            // `เจียรวนนท์` keeps its two middle bases explicit.
            for atom_index in start..end {
                self.ownership
                    .push_undecided(self.ownership.atom_absolute_range(atom_index));
            }
            self.ownership
                .push_syllable(self.ownership.atom_absolute_range(end));
            return true;
        }

        // Rule: a non-carrier silent-mark suffix cannot stand alone; keep its
        // final bare onset with the suffix. When at least two earlier bare bases
        // intervene, expose them as attachment choices; one base retains the
        // normal left attachment.
        // Examples: `เจียรวนนท์` becomes `เจีย | ร? | ว? | นนท์`, while a
        // one-base run such as `แกล้งจน์` remains `แกล้ | งจน์` because no second
        // bare base can take the suffix boundary.
        let leading_bare_atoms = start..end.saturating_sub(1);
        if leading_bare_atoms.len() >= 2 {
            for atom_index in leading_bare_atoms.clone() {
                self.ownership
                    .push_undecided(self.ownership.atom_absolute_range(atom_index));
            }
        } else {
            for atom_index in leading_bare_atoms {
                let end = self.ownership.atom_absolute_range(atom_index).end;
                self.ownership.extend_last_syllable(end);
            }
        }
        let preceding = self.ownership.atom_absolute_range(end - 1);
        let following = self.ownership.atom_absolute_range(end);
        self.ownership.push_syllable(preceding.start..following.end);
        true
    }
}
