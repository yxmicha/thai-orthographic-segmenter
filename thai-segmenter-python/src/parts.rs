use pyo3::prelude::*;
use thai_syllable_rules::{Part, split};

use crate::spans::character_spans;

/// Splits `text` by spelling rules alone, keeping every ambiguity explicit.
///
/// Returns `(kind, text, start, end)` for each part, where `kind` is `"syllable"`,
/// `"undecided"`, `"invalid"`, `"dot"` or `"text"` and the offsets are code points. The parts
/// cover the input in order. `"undecided"` material can belong to the syllable before or after
/// it; the contextual syllable tokenizer decides it.
#[pyfunction]
pub(crate) fn orthographic_parts(
    py: Python<'_>,
    text: &str,
) -> Vec<(&'static str, String, usize, usize)> {
    py.detach(|| {
        let parts = split(text);
        let ranges = parts.iter().map(Part::byte_range).collect::<Vec<_>>();
        parts
            .iter()
            .zip(character_spans(text, &ranges))
            .map(|(part, (start, end))| (kind(part), part.text(text).to_owned(), start, end))
            .collect()
    })
}

fn kind(part: &Part) -> &'static str {
    match part {
        Part::Syllable(_) => "syllable",
        Part::Undecided(_) => "undecided",
        Part::Invalid(_) => "invalid",
        Part::Dot(_) => "dot",
        Part::Text(_) => "text",
    }
}
