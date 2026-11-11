use pyo3::{exceptions::PyValueError, prelude::*};
use thai_word_segmenter::WordSegmenter;

use crate::{
    error::{describe, failure},
    spans::character_spans,
};

/// Texts scored together in one call to the model, which holds activations for all of them.
const BATCH_BYTES: usize = 256 * 1024;

/// Splits text into words (lexical clusters: words, compounds, names and bound forms).
///
/// The words are slices of the input, so concatenating them gives the input back. The model keeps
/// collocations and compounds together and cuts only where it is confident, so it returns fewer,
/// longer words than annotation-style word segmenters. Building a tokenizer takes about 40 ms;
/// build one and reuse it.
#[pyclass(frozen, module = "thai_orthographic_segmenter")]
pub(crate) struct WordTokenizer {
    segmenter: WordSegmenter,
}

#[pymethods]
impl WordTokenizer {
    /// Loads the bundled model with its bundled dictionary of known multi-syllable words.
    #[new]
    fn new(py: Python<'_>) -> PyResult<Self> {
        Self::build(py, WordSegmenter::new)
    }

    /// Loads the bundled model with `words` as the dictionary instead of the bundled one.
    ///
    /// Each word must be non-empty Thai text. Matching is leftmost-longest: a dictionary match
    /// is never split.
    #[staticmethod]
    #[allow(
        clippy::needless_pass_by_value,
        reason = "PyO3 extracts owned arguments and cannot borrow a Python list or path"
    )]
    fn with_dictionary(py: Python<'_>, words: Vec<String>) -> PyResult<Self> {
        let dictionary = words.join("\n");
        Self::build(py, || WordSegmenter::with_dictionary(&dictionary))
    }
}

impl WordTokenizer {
    fn build(
        py: Python<'_>,
        load: impl FnOnce() -> Result<WordSegmenter, thai_word_segmenter::Error> + Send,
    ) -> PyResult<Self> {
        let segmenter = py
            .detach(|| load().map_err(|error| describe(&error)))
            .map_err(failure)?;
        Ok(Self { segmenter })
    }
}
