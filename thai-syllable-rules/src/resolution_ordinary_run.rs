use crate::atom::SilentMarkShape;
use crate::resolution::ResolutionState;

impl ResolutionState<'_> {

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
