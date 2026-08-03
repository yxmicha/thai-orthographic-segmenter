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
