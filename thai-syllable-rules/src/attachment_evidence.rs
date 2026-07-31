use crate::character::ThaiCharacter;
use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;

/// Closure supplied by the preceding rime, independently of coda eligibility.
#[derive(Clone, Copy)]
pub(crate) enum LocalClosure {
    OptionalCoda,
    RequiredCoda,
    ToneMarkedCarrier,
}

/// A possible coda is not an ownership proof when a right reading survives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttachmentEvidence {
    Ineligible,
    LeftOnly,
    Undecided,
}

impl AttachmentEvidence {
    pub(crate) fn ordinary_final(
        characters: &[ThaiCharacter],
        index: usize,
        closure: LocalClosure,
    ) -> Self {
        let sequence = CharacterSequence(characters);
        let following_bare_carrier_competes = {
            // Rule: a preceding base may be either a final or the onset of a
            // following bare อ+base syllable, as in พิทอม. Once that carrier's base
            // has its own dependent vowel, as in เป็นอยู่, the preceding final is no
            // longer ambiguous and may be consumed.
            // Examples: `พิทอม` keeps the boundary choice visible, while `เป็นอยู่`
            // has enough following rime structure to consume `น` as `เป็น` and begin
            // `อยู่`. The distinction comes from lookahead, not a lexical exception.
            sequence.starts_following_carrier_syllable(index)
                && !characters
                    .get(index + 3)
                    .is_some_and(|value| ThaiScalar(value.character).is_dependent())
        } && !matches!(closure, LocalClosure::RequiredCoda);
        let eligible = characters
            .get(index)
            .is_some_and(|value| ThaiScalar(value.character).is_coda())
            && (!sequence.starts_following_initial_cluster(index)
                || {
                    // Rule: กร, ตร, ทร, and สร are the supported unmarked final-ร spellings.
                    // They are also listed onset clusters, so they must remain eligible for
                    // the preceding rime instead of being rejected by onset lookahead.
                    // Examples: terminal `จักร`, `บัตร`, and `มิตร` remain eligible for this
                    // extension, while `จักรยาน` and `มิตรภาพ` leave `ร` unresolved because
                    // a following candidate can claim the boundary. `ติพร` does not treat
                    // พร as an equivalent final cluster.
                    characters.get(index..index + 2).is_some_and(|values| {
                        ThaiScalar(values[0].character)
                            .forms_final_r_cluster_with(ThaiScalar(values[1].character))
                    })
                }
                || sequence.starts_initial_cluster_with_rime(index))
            && !sequence.starts_carrier_syllable_at(index)
            && !following_bare_carrier_competes
            && characters.get(index + 1).is_none_or(|value| {
                !ThaiScalar(value.character).is_dependent()
                    || ThaiScalar(value.character).is_garan_mark()
            });
        if !eligible {
            return Self::Ineligible;
        }
        let implicit_right = sequence.starts_following_implicit_syllable(index);
        let competing_right = match closure {
            LocalClosure::RequiredCoda => false,
            LocalClosure::OptionalCoda => {
                implicit_right || sequence.starts_syllable_after_final(index)
            }
            LocalClosure::ToneMarkedCarrier => {
                let following_cluster_has_rime = sequence
                    .starts_initial_cluster_with_rime(index + 1)
                    || sequence.has_garan_suffix(index + 1);
                implicit_right
                    || (sequence.starts_syllable_after_final(index)
                        && !sequence.starts_initial_cluster_with_rime(index)
                        && !following_cluster_has_rime)
            }
        };
        if competing_right {
            Self::Undecided
        } else {
            Self::LeftOnly
        }
    }
}
