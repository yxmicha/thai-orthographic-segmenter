/// Local closure evidence passed from atomization to the ending rules.
///
/// This is intentionally separate from following-shape lookahead: the fields
/// describe what the current local construction consumed, while lookahead
/// decides whether the next source characters can begin another candidate.
#[derive(Clone, Copy)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts of one scan, each tested on its own"
)]
pub(crate) struct HardEndingContext {
    pub(crate) has_leading_vowel: bool,
    pub(crate) has_independent_vowel: bool,
    pub(crate) has_nucleus: bool,
    pub(crate) has_carrier: bool,
    pub(crate) has_tone_mark: bool,
    pub(crate) consumed_short_a_coda: bool,
    pub(crate) consumed_short_a_glide: bool,
    pub(crate) consumed_ue_carrier: bool,
    pub(crate) has_glide: bool,
    pub(crate) allow_ambiguous_garan_trailing_base: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EndingClosure {
    Open,
    Hard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CodaEvidence {
    None,
    ShortA,
    ToneMarkedImplicit,
    OrdinaryFinal,
    CarrierFinal,
}
