use crate::resolution_input::{FollowingSource, ThaiRun};

/// Rejects impossible local mark stacks before any syllable can claim them.
pub(crate) struct ThaiLegality<'a> {
    text: &'a str,
    source_offset: usize,
    following_source: FollowingSource,
}

impl<'a> ThaiLegality<'a> {
    pub(crate) const fn new(
        text: &'a str,
        source_offset: usize,
        following_source: FollowingSource,
    ) -> Self {
        Self {
            text,
            source_offset,
            following_source,
        }
    }
}
