use crate::character::ThaiCharacter;
use crate::ownership_plan::OwnershipPlan;
use crate::resolution_input::FollowingSource;

// This module turns local `ThaiAtom` candidates into source-preserving `Part`
// ranges. Its
// central invariant is conservative output: emit `Undecided` for unresolved
// attachment choices and for every bare one-scalar base, and never split
// dependent marks away from the construction that owns them.
// Examples: `กะตกลม` may expose bounded undecided atoms between `กะ` and
// `ลม`; `นะก.` permits the documented `ก` abbreviation tail; `วัต` never
// exposes a dependent vowel as its own public part.
pub(crate) struct ResolutionState<'a> {
    text: &'a str,
    source_offset: usize,
    following_source: FollowingSource,
    characters: Vec<ThaiCharacter>,
    atoms: Vec<crate::atom::ThaiAtom>,
    pub(crate) ownership: OwnershipPlan<'a>,
}
