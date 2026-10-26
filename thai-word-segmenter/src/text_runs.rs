use std::ops::Range;
use thai_syllable_rules::ThaiScalar;

/// The syllable pieces of one text, grouped into maximal runs that are entirely Thai or not.
pub struct TextRuns<'a> {
    pieces: Vec<&'a str>,
    /// Piece-index range of each run and whether its pieces are Thai.
    runs: Vec<(Range<usize>, bool)>,
}

impl<'a> TextRuns<'a> {
    /// Groups `pieces`, which concatenate to the text, into Thai and non-Thai runs.
    pub fn new(pieces: Vec<&'a str>) -> Self {
        let is_thai_piece = |piece: &&str| {
            piece
                .chars()
                .all(|character| ThaiScalar::new(character).is_accepted())
        };
        let mut start = 0;
        let runs = pieces
            .chunk_by(|left, right| is_thai_piece(left) == is_thai_piece(right))
            .map(|run| {
                let range = start..start + run.len();
                start = range.end;
                (range, run.iter().all(is_thai_piece))
            })
            .collect();
        Self { pieces, runs }
    }

    /// The syllables of every Thai run, in text order.
    pub fn thai_runs(&self) -> impl Iterator<Item = &[&'a str]> {
        self.runs
            .iter()
            .filter(|(_, thai)| *thai)
            .filter_map(|(range, _)| self.pieces.get(range.clone()))
    }
}
