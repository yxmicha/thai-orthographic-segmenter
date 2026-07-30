use crate::ending::{FinalRExtension, HardEndingContext};

/// What a construction's hard-ending decision has consumed and learned so far.
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts of one scan, each tested on its own"
)]
pub(crate) struct EndingScan {
    pub(crate) index: usize,
    pub(crate) has_hard_ending: bool,
    pub(crate) consumed_hard_final: bool,
    pub(crate) final_r_extension: Option<FinalRExtension>,
    pub(crate) consumed_garan_extension: bool,
    pub(crate) deferred_silent_coda: bool,
}

impl EndingScan {
    /// Starts at `index`, already hard when a short-ั coda, a closed short-ั glide, or a
    /// tone-marked implicit nucleus closed the construction.
    pub(crate) fn new(
        index: usize,
        context: &HardEndingContext,
        evidence: &HardEndingEvidence,
    ) -> Self {
        Self {
            index,
            has_hard_ending: context.consumed_short_a_coda
                || context.consumed_short_a_glide
                || evidence.has_tone_marked_implicit_nucleus,
            consumed_hard_final: false,
            final_r_extension: None,
            consumed_garan_extension: false,
            deferred_silent_coda: false,
        }
    }
}

/// Lookahead and shape facts about a construction at the point its hard ending is decided.
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts of one scan, each tested on its own"
)]
pub(crate) struct HardEndingEvidence {
    pub(crate) has_tone_marked_implicit_nucleus: bool,
    pub(crate) can_consume_tone_marked_final: bool,
    pub(crate) has_taikhu: bool,
    pub(crate) taikhu_final_precedes_following_cluster: bool,
    pub(crate) can_consume_carrier_final: bool,
    pub(crate) starts_with_inherent_final_leading_vowel: bool,
    pub(crate) has_e_ao_pattern: bool,
    pub(crate) has_ue_rime: bool,
    pub(crate) has_preposed_i: bool,
    pub(crate) has_short_i: bool,
    pub(crate) can_consume_short_u_final: bool,
}
