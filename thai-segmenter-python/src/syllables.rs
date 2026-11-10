use pyo3::prelude::*;
use thai_syllable_segmenter::SyllableSegmenter;

use crate::{
    error::{describe, failure},
    spans::{character_spans, consecutive_ranges},
};

/// Splits text into written Thai syllables.
///
/// The syllables are slices of the input, so concatenating them gives the input back. Building a
/// tokenizer takes a few milliseconds; build one and reuse it.
#[pyclass(frozen, module = "thai_orthographic_segmenter")]
pub(crate) struct SyllableTokenizer {
    segmenter: SyllableSegmenter,
}

#[pymethods]
impl SyllableTokenizer {
    #[new]
    fn new(py: Python<'_>) -> PyResult<Self> {
        let segmenter = py
            .detach(|| SyllableSegmenter::new().map_err(|error| describe(&error)))
            .map_err(failure)?;
        Ok(Self { segmenter })
    }

    /// Returns the syllables of `text`. Non-Thai text comes back as pieces between them.
    fn tokenize(&self, py: Python<'_>, text: &str) -> PyResult<Vec<String>> {
        py.detach(|| {
            self.segmenter
                .split(text)
                .map(|pieces| pieces.into_iter().map(str::to_owned).collect())
                .map_err(|error| describe(&error))
        })
        .map_err(failure)
    }

    /// Returns the `(start, end)` code point offsets of each syllable of `text`.
    fn spans(&self, py: Python<'_>, text: &str) -> PyResult<Vec<(usize, usize)>> {
        py.detach(|| {
            self.segmenter
                .split(text)
                .map(|pieces| character_spans(text, &consecutive_ranges(pieces)))
                .map_err(|error| describe(&error))
        })
        .map_err(failure)
    }
}
