use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;

// These helpers keep silent-mark scanning beside atomization. They report
// source-local shape facts; resolution still decides whether an opaque suffix
// owns a neighboring bare consonant.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SilentMarkSuffixScan {
    pub(crate) base_probe: usize,
    pub(crate) dependent_end: usize,
    pub(crate) base_count: usize,
    pub(crate) has_immediate_garan: bool,
    pub(crate) has_garan_after_base: bool,
}

impl CharacterSequence<'_> {
    pub(crate) fn has_ambiguous_silent_mark_suffix(
        self,
        start: usize,
        index: usize,
        suffix: Option<SilentMarkSuffixScan>,
    ) -> bool {
        let characters = self.0;
        // Rule: a multi-base tail ending in garan is ambiguous unless either the
        // first pair or the second pair is an allowed initial cluster.
        // Examples: `กะมเหสักข์` keeps the silent-mark tail opaque for later
        // resolution, while `การ์ดา` can close as `การ์ | ดา` because `ดา` is a
        // clear following nucleus. The cluster-shaped `กะกเรนทร์` is not treated
        // as the same kind of free tail because `กร` is an allowed onset pair.
        let Some(first_base) = characters.get(start).map(|value| value.character) else {
            return false;
        };
        let Some(suffix) = suffix else {
            return false;
        };
        if suffix.base_count < 2 || !suffix.has_garan_after_base {
            return false;
        }

        let Some(first_following_base) = characters.get(index).map(|value| value.character) else {
            return false;
        };
        let second_following_base = characters.get(index + 1).map(|value| value.character);

        !ThaiScalar(first_base).forms_initial_cluster_with(ThaiScalar(first_following_base))
            && !second_following_base.is_some_and(|second| {
                ThaiScalar(first_following_base).forms_initial_cluster_with(ThaiScalar(second))
            })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SilentMarkSuffixShape {
    pub(crate) starts_at_last_initial_cluster: bool,
    pub(crate) precedes_initial_cluster: bool,
    pub(crate) precedes_any_initial_cluster: bool,
}
