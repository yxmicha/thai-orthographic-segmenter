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
