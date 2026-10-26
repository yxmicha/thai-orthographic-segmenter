use std::ops::Range;

/// The syllable pieces of one text, grouped into maximal runs that are entirely Thai or not.
pub struct TextRuns<'a> {
    pieces: Vec<&'a str>,
    /// Piece-index range of each run and whether its pieces are Thai.
    runs: Vec<(Range<usize>, bool)>,
}
