use crate::atomization_rime::{long_a_dependent_character, short_a_dependent_character};
use crate::character::ThaiCharacter;
use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
use winnow::Parser;
use winnow::token::any;

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

    /// Whether the source after the second base contains a separate nucleus.
    fn cluster_has_following_nucleus(self, second_base_index: usize) -> bool {
        let characters = self.0;
        // Rule: an ordinary leading-vowel cluster exposes its coda-capable second
        // base only when the source after that base contains a separate nucleus.
        // A bare consonant by itself is not enough evidence: it may simply be the
        // final of the leading-vowel spelling. This narrower probe is deliberately
        // different from the inherent-final ใ/ไ lookahead above, which has its own
        // positive final-ownership rule.
        // Examples: `เครดิต` becomes `เค | ?ร | ดิต` because ดิ is a following
        // rime, while terminal `เบรก` exposes `เบ | รก` because รก is a complete
        // right-hand implicit candidate. A following leading-vowel or independent-
        // vowel construction also supplies the required right-hand nucleus.
        let following_index = second_base_index + 1;
        let Some(following) = characters.get(following_index) else {
            return false;
        };
        if ThaiScalar(following.character).is_leading_vowel() {
            return characters
                .get(following_index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_base());
        }
        if ThaiScalar(following.character).is_independent_vowel() {
            return true;
        }
        if following.character == '\u{0e31}' {
            // A standalone short ั cannot complete the preceding leading-vowel
            // cluster. It starts the following base's own rime, as in
            // `เซรั่ม` -> `เซ | รั่ม`; longer leading-vowel rimes such as `เครือ`
            // and recognized `เ...าะ` forms are handled by the branches below.
            return true;
        }
        if ThaiScalar(following.character).is_vowel_carrier()
            && characters
                .get(following_index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_dependent())
        {
            // The carrier and its dependent mark complete the leading-vowel
            // spelling's local rime; they are not a separate right-hand nucleus.
            // Example: `เปรอะ` keeps `ร` in the same candidate instead of exposing
            // `เป | ?ร | อะ`.
            return false;
        }
        ThaiScalar(following.character).is_base()
            && (self.starts_following_syllable(following_index)
                || self.starts_following_implicit_syllable(following_index))
    }

    /// Whether the second base and a terminal base form a complete implicit candidate.
    fn cluster_has_terminal_implicit_candidate(self, second_base_index: usize) -> bool {
        let characters = self.0;
        // Rule: a terminal bare base after a leading-vowel cluster can provide a
        // complete right-hand implicit candidate together with that cluster's
        // second base. This is positive evidence to expose the cluster boundary:
        // `เบรก` becomes `เบ | รก` rather than hiding both candidates in `เบรก`.
        // The candidate must end at the Thai run boundary and its final base must
        // be coda-capable; dependent marks or more Thai material require the
        // ordinary lookahead rules instead.
        let following_index = second_base_index + 1;
        characters
            .get(second_base_index..following_index + 1)
            .is_some_and(|values| {
                ThaiScalar(values[0].character).is_base()
                    && ThaiScalar(values[1].character).is_base()
                    && ThaiScalar(values[1].character).is_coda()
            })
            && characters.get(following_index + 1).is_none()
    }

    /// Whether a trailing `ี` starts a new syllable at the second base.
    fn cluster_starts_unrelated_i_rime(self, start: usize, second_base_index: usize) -> bool {
        let characters = self.0;
        let leading_vowel_index = start;
        // Rule: เ + base + cluster-base + ี is not one compound vowel unless ย
        // completes the written เ-ีย frame. Without ย, the ี belongs to a new
        // syllable beginning at the cluster's second base. This keeps `เสรี` as
        // `เส | รี` while retaining `เกรียน` as one leading-vowel construction.
        characters
                .get(leading_vowel_index)
                .is_some_and(|value| value.character == '\u{0e40}') // เ
                && characters
                    .get(second_base_index + 1)
                    .is_some_and(|value| value.character == '\u{0e35}') // ี
                && !'next_base_after_tone_marks_is_y: {
                    let mut index = second_base_index + 2;
                    while characters
                        .get(index)
                        .is_some_and(|value| ThaiScalar(value.character).is_tone_mark())
                    {
                        index += 1;
                    }
                    if characters
                        .get(index)
                        .is_none_or(|value| value.character != '\u{0e22}')
                    // ย
                    {
                        break 'next_base_after_tone_marks_is_y false;
                    }
                    !characters
                        .get(index + 1)
                        .is_some_and(|value| ThaiScalar(value.character).is_vowel())

                }
    }

    /// Whether a leading `เ` and two bases precede a `าะ` rime.
    fn starts_leading_ao_pattern(self, start: usize, index: usize) -> bool {
        let characters = self.0;
        // Rule: a leading เ followed by one non-cluster base and a second base
        // before าะ is one cross-atom เ-าะ vowel pattern. The second base is not
        // an independent onset merely because the first base did not form one of
        // the listed initial clusters. Examples: เฉพาะ, เรพาะ, and เสนาะ keep
        // their เ...าะ spelling together; without this lookahead the scanner
        // would incorrectly emit เฉ | พาะ, เร | พาะ, or เส | นาะ.
        if characters
            .get(start)
            .is_none_or(|value| value.character != '\u{0e40}')
        // เ
        {
            return false;
        }

        let Some(mut input) = characters.get(index..) else {
            return false;
        };
        let mut parser = (
            any.verify(|value: &ThaiCharacter| ThaiScalar(value.character).is_base()),
            long_a_dependent_character,
            short_a_dependent_character,
        )
            .void();
        parser.parse_next(&mut input).is_ok()
    }
}
