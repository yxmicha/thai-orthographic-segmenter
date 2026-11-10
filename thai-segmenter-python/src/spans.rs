use std::ops::Range;

/// Converts ascending, non-overlapping UTF-8 byte ranges of `text` into the Unicode code point
/// offsets Python uses to index a `str`.
///
/// A range that does not lie on character boundaries of `text` is clamped to the nearest
/// earlier boundary, which cannot happen for the ranges the segmenters return.
pub(crate) fn character_spans(text: &str, ranges: &[Range<usize>]) -> Vec<(usize, usize)> {
    let mut byte = 0;
    let mut character = 0;
    let mut advance_to = |target: usize| {
        let skipped = text
            .get(byte..target)
            .map_or(0, |between| between.chars().count());
        if text.get(byte..target).is_some() {
            byte = target;
        }
        character += skipped;
        character
    };
    ranges
        .iter()
        .map(|range| (advance_to(range.start), advance_to(range.end)))
        .collect()
}

/// The byte ranges of consecutive pieces that together make up their text.
pub(crate) fn consecutive_ranges<'a>(
    pieces: impl IntoIterator<Item = &'a str>,
) -> Vec<Range<usize>> {
    let mut end = 0;
    pieces
        .into_iter()
        .map(|piece| {
            let start = end;
            end += piece.len();
            start..end
        })
        .collect()
}
