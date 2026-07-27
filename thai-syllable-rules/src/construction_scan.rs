/// Source-local facts gathered while scanning one construction, before atom interpretation.
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts of one scan, each tested on its own"
)]
pub(crate) struct ConstructionScan {
    pub(crate) start: usize,
    pub(crate) index: usize,
    pub(crate) character: char,
    pub(crate) first_base: Option<char>,
    pub(crate) has_nucleus: bool,
    pub(crate) has_vowel_rime: bool,
    pub(crate) has_ue_rime: bool,
    pub(crate) has_independent_vowel: bool,
    pub(crate) is_opaque: bool,
    pub(crate) has_leading_vowel: bool,
    pub(crate) has_carrier: bool,
    pub(crate) has_glide: bool,
    pub(crate) consumed_short_a_coda: bool,
    pub(crate) consumed_short_a_glide: bool,
    pub(crate) consumed_ue_carrier: bool,
}
