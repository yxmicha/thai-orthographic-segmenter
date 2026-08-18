use crate::part::{ByteRange, Part, push_text};
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

    fn resolve_span(&self, range: ByteRange) -> Vec<Part> {
        if range.is_empty() {
            return Vec::new();
        }
        // Invalid cells are hard barriers; only the original final span can
        // inherit an adjacent period's abbreviation policy.
        let following = if range.end == self.text.len() {
            self.following_source
        } else {
            FollowingSource::None
        };
        if let Some(mut run) = ThaiRun::parse(
            &self.text[range.clone()],
            self.source_offset + range.start,
            following,
        ) {
            run.prepare_candidates();
            run.resolve()
        } else {
            let mut parts = Vec::new();
            push_text(
                &mut parts,
                self.source_offset + range.start..self.source_offset + range.end,
            );
            parts
        }
    }
}
