use crate::character::ThaiScalar;
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

    pub(crate) fn resolve(self) -> Vec<Part> {
        let mut parts = Vec::new();
        let mut legal_start = 0;
        let mut characters = self.text.char_indices().peekable();
        while let Some((start, base)) = characters.next() {
            if ThaiScalar(base).is_combining_mark()
                || (base == 'ๅ' && !self.text[..start].ends_with(['ฤ', 'ฦ']))
                || (ThaiScalar(base).is_leading_vowel()
                    && characters
                        .peek()
                        .is_none_or(|&(_, next)| !ThaiScalar(next).is_base()))
            {
                // A mark reached here was not consumed with a consonant cell.
                // Keep its source, but do not let either neighbor adopt it.
                parts.extend(self.resolve_span(legal_start..start));
                let end = start + base.len_utf8();
                push_text(
                    &mut parts,
                    self.source_offset + start..self.source_offset + end,
                );
                legal_start = end;
                continue;
            }
            if !ThaiScalar(base).is_base() {
                continue;
            }
            let mut end = start + base.len_utf8();
            let mut marks = 0_u32;
            let mut vowels = 0;
            let mut top_marks = 0;
            let mut phinthu = false;
            let mut conflicting = false;
            while let Some(&(offset, mark)) = characters.peek() {
                if !ThaiScalar(mark).is_combining_mark() {
                    break;
                }
                characters.next();
                end = offset + mark.len_utf8();
                let bit = 1_u32 << (u32::from(mark) - u32::from('\u{0e31}'));
                conflicting |= marks & bit != 0;
                conflicting |=
                    top_marks > 0 && matches!(mark, '\u{0e31}' | '\u{0e34}'..='\u{0e39}');
                marks |= bit;
                vowels += usize::from(matches!(mark, '\u{0e31}' | '\u{0e34}'..='\u{0e39}'));
                top_marks += usize::from(matches!(mark, '\u{0e47}'..='\u{0e4c}'));
                phinthu |= mark == 'ฺ';
            }
            if conflicting || vowels > 1 || top_marks > 1 || (phinthu && marks.count_ones() > 1) {
                let start = self.text[..start]
                    .char_indices()
                    .next_back()
                    .filter(|&(offset, character)| {
                        offset >= legal_start && ThaiScalar(character).is_leading_vowel()
                    })
                    .map_or(start, |(offset, _)| offset);
                parts.extend(self.resolve_span(legal_start..start));
                parts.push(Part::Invalid(
                    self.source_offset + start..self.source_offset + end,
                ));
                legal_start = end;
            }
        }
        parts.extend(self.resolve_span(legal_start..self.text.len()));
        parts
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
