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
    pub(crate) fn implicit_syllable_candidates(
        self,
        source_offset: usize,
    ) -> Option<ImplicitCandidateRanges> {
        let atoms = self.0;
        // Rule: a terminal bare sequence can be represented as two structural
        // implicit-vowel candidates, or one three-base onset candidate plus the
        // remaining candidate, without consulting lexical data.
        // Examples: `จมก` can expose a left candidate and a right candidate while
        // preserving the middle attachment choice; `กเรณุ` recognizes the `กร`
        // onset shape before the following nucleus. No dictionary entry is needed
        // to decide that the source atoms are contiguous and structurally usable.
        let first_length = self.implicit_syllable_sequence_start()?;
        Some(ImplicitCandidateRanges {
            first: atoms[0].absolute_range(source_offset).start
                ..atoms[first_length - 1].absolute_range(source_offset).end,
            second: (first_length < atoms.len()).then(|| {
                atoms[first_length].absolute_range(source_offset).start
                    ..atoms[atoms.len() - 1].absolute_range(source_offset).end
            }),
        })
    }

    pub(crate) fn implicit_syllable_sequence_start(self) -> Option<usize> {
        let atoms = self.0;
        // Rule: emit a split only when both sides are independently valid
        // implicit candidates; otherwise preserve the full conservative range.
        // Examples: a four-atom bare run can split only when both resulting sides
        // have at least two atoms; a three-atom run stays whole rather than
        // producing an orphan one-character candidate. This is why terminal
        // `จมก` is conservative when no safe right-hand candidate exists.
        let first_length = self.implicit_syllable_length();
        if first_length < 2 {
            return None;
        }
        if first_length == atoms.len() {
            return Some(first_length);
        }
        let second_length = AtomSequence(&atoms[first_length..]).implicit_syllable_length();
        (second_length >= 2 && first_length + second_length == atoms.len()).then_some(first_length)
    }

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
