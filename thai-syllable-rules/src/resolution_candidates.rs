use crate::character::ThaiScalar;
use crate::resolution::ResolutionState;

impl ResolutionState<'_> {
    pub(crate) fn expose_repeated_carrier_onset(&mut self, start: usize, end: usize) -> bool {
        // Rule: when an unresolved coda-capable base is immediately followed by a
        // carrier atom beginning with the same base, the repeated base may instead
        // continue the preceding implicit-vowel construction. This rule applies
        // only after an implicit candidate; an explicit carrier such as `ตอ`
        // already supplies its vowel, so the following `นอ` keeps its onset.
        // Keep both bases undecided and strip only the carrier's onset from the
        // following atom when the implicit context proves that choice remains
        // ambiguous.
        // Examples: `จมรรอก` becomes `จม | ?ร | ?ร | อก`; the two ร bases may
        // form the implicit-a material after จม, or the second one may begin รอก.
        // `ตอนนอนสิ` becomes `ตอ | ?น | นอ | ?น | สิ`; the explicit carrier in
        // ตอ does not make the onset of นอ ambiguous. `กุรอก` uses ordinary
        // singleton attachment instead: ร remains a possible coda of กุ.
        let Some(previous) = start
            .checked_sub(1)
            .and_then(|index| self.atoms().get(index))
        else {
            return false;
        };
        if previous.has_explicit_rime()
            || previous.has_carrier()
            || previous.starts_with_leading_vowel()
            || previous.has_tone_mark
            || previous.has_hard_ending()
        {
            return false;
        }
        if end - start != 1
            || !self.can_attach_singleton_as_coda(start)
            || !self.atoms()[end].has_carrier()
            || self.atoms()[start].first_base() != self.atoms()[end].first_base()
        {
            return false;
        }

        let repeated_base_start = self.ownership.atom_starts[end];
        let Some(repeated_base) = self.atoms()[end].first_base() else {
            return false;
        };
        for atom_index in start..end {
            self.ownership
                .push_undecided(self.ownership.atom_absolute_range(atom_index));
        }
        let repeated_base_end = repeated_base_start + repeated_base.len_utf8();
        {
            let (index, end) = (end, repeated_base_end);
            let plan = &mut self.ownership;
            let start = plan.atom_starts[index];
            debug_assert!(end > start && end < plan.atom_ranges[index].end);
            plan.push_undecided((plan.source_offset + start)..(plan.source_offset + end));
            plan.atom_starts[index] = end;
        };
        true
    }
}

impl ResolutionState<'_> {
    pub(crate) fn can_attach_singleton_as_coda(&self, start: usize) -> bool {
        // Ordinary ร is a /n/ coda, including after an explicit vowel (การ).
        // Its silent-extension role must not remove this competing attachment.
        self.atoms()
            .get(start)
            .and_then(super::atom::ThaiAtom::first_base)
            .is_some_and(|character| ThaiScalar(character).is_coda())
    }
}
