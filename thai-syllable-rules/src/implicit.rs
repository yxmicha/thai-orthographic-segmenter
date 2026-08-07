use crate::atom::ThaiAtom;
use crate::atom_sequence::AtomSequence;
use crate::character::ThaiScalar;
use crate::part::ByteRange;

/// Structural ranges synthesized from a terminal bare run.
///
/// The candidate pass returns ranges rather than public parts. Ownership
/// resolution decides whether these ranges become syllables or remain part of
/// an unresolved run; public parts are emitted only after that decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImplicitCandidateRanges {
    pub(crate) first: ByteRange,
    pub(crate) second: Option<ByteRange>,
}

impl AtomSequence<'_> {

    pub(crate) fn implicit_syllable_length(self) -> usize {
        let atoms = self.0;
        // Rule: implicit candidates use two bases by default; a recognized
        // initial cluster plus a third base forms the three-base shape. The second
        // base must also be a legal Thai coda, because a bare two-base candidate
        // implicitly places that base at the end of the syllable. This is the
        // shared structural guard for every synthetic implicit candidate; without
        // it, an onset-only base such as ห could be misread as the final of `นห`.
        // Examples: `จม` is the default two-base candidate, `กรา` uses the
        // cluster-plus-base shape, and `นห` is not an implicit candidate because
        // ห cannot be a bare coda and must remain available to the following
        // `หน...` onset. A pair such as `กม` is still not treated as `กร` and is
        // left to the normal unresolved attachment path.
        if atoms.iter().any(ThaiAtom::has_phinthu) {
            // PHINTHU suppresses the inherent vowel. A marked consonant sequence
            // therefore cannot be promoted as an ordinary implicit-vowel form.
            return 1;
        }
        let Some(second) = atoms.get(1) else {
            return 1;
        };
        if atoms[0].first_base().is_none() || second.first_base().is_none() {
            return 1;
        }
        if !second
            .first_base()
            .is_some_and(|character| ThaiScalar(character).is_coda())
        {
            return 1;
        }
        if atoms.len() >= 3
            && atoms[0]
                .first_base()
                .zip(second.first_base())
                .is_some_and(|(first, second)| {
                    ThaiScalar(first).forms_initial_cluster_with(ThaiScalar(second))
                })
            && atoms[2]
                .first_base()
                .is_some_and(|character| ThaiScalar(character).is_coda())
        {
            return 3;
        }
        2
    }
}
