use crate::character::ThaiScalar;
use crate::resolution::ResolutionState;

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
