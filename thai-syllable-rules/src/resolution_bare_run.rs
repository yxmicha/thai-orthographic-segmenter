use std::ops::Range;
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
}
