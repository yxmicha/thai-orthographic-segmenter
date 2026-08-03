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

    pub(crate) fn silent_mark_suffix_shape(
        self,
        start: usize,
        suffix: Option<SilentMarkSuffixScan>,
    ) -> SilentMarkSuffixShape {
        let characters = self.0;
        // Rule: an initial-cluster-shaped garan candidate may start an opaque
        // atom when that cluster is the last initial cluster before the silent
        // mark. Earlier clusters belong to unresolved material before the suffix,
        // not to the suffix itself.
        // Examples: `จรรย์` starts at its final `จร` cluster and remains one
        // candidate, while the earlier `พร` and `หม` in `พรหมจรรย์` remain
        // available to the surrounding ambiguity resolver. This prevents a
        // preceding bare base from swallowing every consonant through `จรรย์`.
        let Some(suffix) = suffix.filter(|suffix| suffix.has_immediate_garan) else {
            return SilentMarkSuffixShape::default();
        };

        // Rule: expose whether any later initial cluster occurs in the same bare
        // run separately from the ordinary multi-cluster guard. The broad fact is
        // used only when an unmarked open carrier precedes the run; that carrier
        // leaves its immediately following base available as a possible coda. A
        // compact spelling such as `ศาสตร์` has no such preceding carrier and
        // remains one candidate.
        // Examples: `ขอนหงส์` keeps `น` available as `ขอ | ?น | หงส์`, while
        // `จรรย์` starts at its own final `จร` cluster and `ศาสตร์` remains
        // compact.
        let (cluster_count, last_cluster_start) = {
            let (start, probe) = (start, suffix.base_probe);
            let characters = self.0;
            characters.get(start..probe).map_or((0, None), |values| {
                values
                    .windows(2)
                    .enumerate()
                    .filter(|(_, pair)| {
                        ThaiScalar(pair[0].character)
                            .forms_initial_cluster_with(ThaiScalar(pair[1].character))
                    })
                    .fold((0, None), |(count, _), (offset, _)| {
                        (count + 1, Some(start + offset))
                    })
            })
        };
        let current_continues_initial_cluster = start > 0
            && characters
                .get(start - 1)
                .zip(characters.get(start))
                .is_some_and(|(previous, current)| {
                    ThaiScalar(previous.character)
                        .forms_initial_cluster_with(ThaiScalar(current.character))
                });

        SilentMarkSuffixShape {
            starts_at_last_initial_cluster: last_cluster_start == Some(start),
            precedes_initial_cluster: current_continues_initial_cluster
                || (cluster_count > 1 && last_cluster_start != Some(start)),
            precedes_any_initial_cluster: current_continues_initial_cluster
                || (cluster_count > 0 && last_cluster_start != Some(start)),
        }
    }

    pub(crate) fn scan_silent_mark_suffix(self, index: usize) -> Option<SilentMarkSuffixScan> {
        let characters = self.0;
        let mut base_probe = index;
        while characters
            .get(base_probe..base_probe + 1)
            .is_some_and(|values| ThaiScalar(values[0].character).is_base())
        {
            base_probe += 1;
        }
        if base_probe == index {
            return None;
        }

        let mut dependent_end = base_probe;
        let mut has_immediate_garan = false;
        while characters
            .get(dependent_end..dependent_end + 1)
            .is_some_and(|values| ThaiScalar(values[0].character).is_dependent())
        {
            has_immediate_garan |= ThaiScalar(characters[dependent_end].character).is_garan_mark();
            dependent_end += 1;
        }
        if has_immediate_garan
            && characters[index..base_probe]
                .iter()
                .enumerate()
                .find(|(_, value)| ThaiScalar(value.character).is_vowel_carrier())
                .is_some_and(|(offset, _)| offset > 0)
        {
            // Rule: a carrier-bearing garan spelling starts at the base immediately
            // before its carrier. If an earlier bare run reaches across that start,
            // it must not be atomized as one opaque candidate. Examples: `ฟอร์ม`
            // starts at `ฟ`, while `กลมฟอร์ม` must leave `ก`, `ล`, and `ม` available
            // to ownership resolution before consuming the later `ฟอร์ม` suffix.
            return None;
        }
        // Rule: only multi-base candidates need the long-range garan probe. A
        // one-base suffix uses `has_immediate_garan` for consumption and cannot
        // satisfy either the compact hard-ending rule or the multi-base ambiguity
        // rule, so avoid rescanning the rest of a long Thai run.
        let has_garan_after_base = (base_probe - index >= 2)
            && characters.get(base_probe..).is_some_and(|values| {
                values
                    .iter()
                    .any(|value| ThaiScalar(value.character).is_garan_mark())
            });
        Some(SilentMarkSuffixScan {
            base_probe,
            dependent_end,
            base_count: base_probe - index,
            has_immediate_garan,
            has_garan_after_base,
        })
    }

    pub(crate) fn consume_silent_mark_suffix(
        self,
        index: &mut usize,
        allow_ambiguous_trailing_base: bool,
        suffix: Option<SilentMarkSuffixScan>,
    ) -> bool {
        // Rule: consume a garan-bearing suffix as one opaque candidate, then let
        // shape-based lookahead decide whether its trailing base belongs to it.
        // Examples: `ฟอร์มและ` consumes the silent-mark spelling before `และ`,
        // while `การ์ดา` leaves the `ดา` nucleus for the following candidate.
        // `เทิร์นเปี๊ยะ` likewise keeps `เทิร์น` intact without merging the next
        // leading-vowel construction into its garan suffix.
        let Some(suffix) = suffix.filter(|suffix| suffix.has_immediate_garan) else {
            return false;
        };

        *index = suffix.dependent_end;
        if self.can_consume_garan_trailing_base(suffix.dependent_end, allow_ambiguous_trailing_base)
        {
            *index = suffix.dependent_end + 1;
        }
        true
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SilentMarkSuffixShape {
    pub(crate) starts_at_last_initial_cluster: bool,
    pub(crate) precedes_initial_cluster: bool,
    pub(crate) precedes_any_initial_cluster: bool,
}
