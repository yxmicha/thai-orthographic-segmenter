use crate::character::ThaiScalar;
use crate::resolution::ResolutionState;

impl ResolutionState<'_> {
    // Rule: only the short ั, ิ, and ุ rimes enter the unmarked final-ร pass.
    // Reusing that same local evidence in resolution prevents a different hard
    // ending such as ำ from being treated as if its following `ร` could attach
    // backward. Examples: `มันรสเหมือน` exposes `ร` and `ส`, while `ทำรถไฟ`
    // keeps the certain `รถ` candidate intact.
    fn preceding_atom_allows_unmarked_final_r(&self, start: usize) -> bool {
        let Some(previous) = start.checked_sub(1) else {
            return false;
        };
        let range = self.ownership.atom_range(previous);
        self.characters().iter().any(|character| {
            if character.start < range.start || character.end > range.end {
                return false;
            }
            matches!(
                character.character,
                '\u{0e31}' // อั
                    | '\u{0e34}' // อิ
                    | '\u{0e38}' // อุ
            )
        })
    }

    /// Whether the lone base at `start` cannot close the inherent-final ใ-/ไ- atom before it and
    /// must instead be adopted by the following candidate.
    fn singleton_cannot_attach_to_inherent_final_left(&self, start: usize) -> bool {
        let Some(previous) = start.checked_sub(1) else {
            return false;
        };
        let previous_atom = &self.atoms()[previous];
        let previous_range = self.ownership.atom_range(previous);
        let starts_with_inherent_final = self
            .characters()
            .iter()
            .find(|character| character.start == previous_range.start)
            .is_some_and(|character| {
                ThaiScalar(character.character).is_inherent_final_leading_vowel()
            });
        if !starts_with_inherent_final {
            return false;
        }

        let Some(previous_base) = previous_atom.first_base() else {
            return false;
        };
        let Some(current_base) = self.atoms()[start].first_base() else {
            return false;
        };

        // Rule: a bare ย remains ambiguous after ใ-/ไ- because the inherent-final
        // spelling may own it; this is the one unresolved bare continuation that
        // the hard-ending closure deliberately preserves. A listed initial
        // cluster has the same positive evidence for the following syllable's
        // onset, so its first base remains ambiguous as well. A coda-capable
        // singleton also has a valid left attachment and may start the following
        // candidate, so it remains undecided. Every other singleton has no valid
        // left attachment to the already closed inherent-final atom and must be
        // adopted by the following candidate.
        // Examples: `ในจมก` moves `จ` into `จมก`, while `ไหลย้อน` keeps `ล`
        // undecided because ห+ล is a listed cluster, and `ไกยกะ` keeps `ย`
        // undecided because ย can still belong to the ไก spelling. `ไหนโอ` keeps
        // `น` undecided because it can close `ไห` or begin `โอ`.
        if current_base == '\u{0e22}' // ย
        || ThaiScalar(previous_base).forms_initial_cluster_with(ThaiScalar(current_base))
        || ThaiScalar(current_base).is_coda()
        {
            return false;
        }

        true
    }
}
