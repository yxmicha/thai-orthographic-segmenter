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
}
