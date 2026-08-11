use std::ops::Range;

/// A maximal run of bare atoms and the facts about its neighbors that decide its ownership.
#[derive(Clone)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts of one scan, each tested on its own"
)]
pub(crate) struct BareRun {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) range: Range<usize>,
    pub(crate) run_has_silent_mark: bool,
    pub(crate) has_previous_syllable: bool,
    pub(crate) has_following_syllable: bool,
    pub(crate) previous_has_hard_ending: bool,
    pub(crate) is_terminal_singleton_before_dot: bool,
    pub(crate) contains_orphan_dependent_mark: bool,
    pub(crate) contains_orphan_vowel_mark: bool,
    pub(crate) previous_ends_in_base: bool,
    pub(crate) is_single_vowel_carrier: bool,
}
