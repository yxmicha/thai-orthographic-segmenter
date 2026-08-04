use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;

/// Lookahead facts that decide whether the second base after a leading vowel joins its cluster.
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts of one scan, each tested on its own"
)]
struct SecondBaseEvidence {
    inherent_final_cluster_has_following_onset: bool,
    cluster_has_following_nucleus: bool,
    cluster_has_terminal_implicit_candidate: bool,
    cluster_starts_unrelated_i_rime: bool,
    starts_leading_ao_pattern: bool,
}

impl CharacterSequence<'_> {

    /// Whether another onset-bearing Thai construction follows the second base.
    fn inherent_final_cluster_has_following_onset(self, second_base_index: usize) -> bool {
        let characters = self.0;
        // Rule: a second base in an inherent-final leading-vowel cluster is not
        // certainly owned by that cluster when another onset-bearing Thai
        // construction follows it. The second base may instead close the
        // preceding `ไ-/ใ-` spelling, so leave it as a separate bare atom for the
        // ambiguity resolver. A following base, leading vowel, independent
        // vowel, or listed initial cluster is enough evidence that another
        // candidate follows; the cluster does not prove ownership of either side
        // of the boundary. A dependent mark is still part of the current cluster.
        // Examples: `ไหลย้อน` becomes `ไห | ?ล | ย้อน`, while terminal `ไหล` keeps
        // `ล` because no following candidate can claim it. `ไหนครับ` becomes
        // `ไห | ?น | ?ค | รับ`: `คร` identifies the right-hand candidate but does
        // not prove that `น` belongs to `ไหน` or that `ค` belongs to `ครับ`.
        let following_index = second_base_index + 1;
        characters.get(following_index).is_some_and(|value| {
            ThaiScalar(value.character).is_base()
                || ThaiScalar(value.character).is_leading_vowel()
                || ThaiScalar(value.character).is_independent_vowel()
        })
    }
}
