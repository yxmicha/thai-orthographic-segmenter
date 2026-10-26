use std::ops::Range;

use crate::Error;
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

    /// Byte ranges of the text's segments before punctuation attachment. Each Thai run splits
    /// after the syllables its next entry of `breaks` marks, and the last syllable always ends a
    /// segment. Non-Thai pieces stay whole, except that closing punctuation at the start of
    /// one is split off so it can attach to the word before it.
    pub fn ranges(
        &self,
        breaks: &mut impl Iterator<Item = Vec<bool>>,
    ) -> Result<Vec<Range<usize>>, Error> {
        let mut ranges = Vec::new();
        let mut start = 0;
        for (range, thai) in &self.runs {
            let run = self
                .pieces
                .get(range.clone())
                .ok_or(Error::Ranges("run outside the pieces"))?;
            if *thai {
                let run_breaks = breaks.next().ok_or(Error::Scores("missing run breaks"))?;
                let mut end = start;
                for (index, syllable) in run.iter().enumerate() {
                    end += syllable.len();
                    if run_breaks.get(index).copied().unwrap_or(false) || index + 1 == run.len() {
                        ranges.push(start..end);
                        start = end;
                    }
                }
                continue;
            }
            for piece in run {
                let punctuation_prefix_length = piece
                    .chars()
                    .take_while(|character| {
                        matches!(character, '.' | '…' | '’' | '”' | ')' | ']' | '}')
                    })
                    .map(char::len_utf8)
                    .sum::<usize>();
                if punctuation_prefix_length > 0 && punctuation_prefix_length < piece.len() {
                    ranges.push(start..start + punctuation_prefix_length);
                    ranges.push(start + punctuation_prefix_length..start + piece.len());
                } else {
                    ranges.push(start..start + piece.len());
                }
                start += piece.len();
            }
        }
        Ok(ranges)
    }
}
