use std::{num::NonZeroUsize, path::PathBuf, thread};

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

    /// Loads a trained artifact directory holding `config.json`, `dictionary.txt` and `model.safetensors`.
    #[staticmethod]
    #[allow(
        clippy::needless_pass_by_value,
        reason = "PyO3 extracts owned arguments and cannot borrow a Python list or path"
    )]
    fn load(py: Python<'_>, directory: PathBuf) -> PyResult<Self> {
        Self::build(py, || WordSegmenter::load(&directory))
    }

    /// Returns the words of `text`. Non-Thai text and punctuation come back as pieces too, with
    /// closing punctuation attached to the word before it and opening punctuation to the next.
    fn tokenize(&self, py: Python<'_>, text: &str) -> PyResult<Vec<String>> {
        py.detach(|| {
            self.segmenter
                .split(text)
                .map(|words| words.into_iter().map(str::to_owned).collect())
                .map_err(|error| describe(&error))
        })
        .map_err(failure)
    }

    /// Returns the `(start, end)` code point offsets of each word of `text`.
    fn spans(&self, py: Python<'_>, text: &str) -> PyResult<Vec<(usize, usize)>> {
        py.detach(|| {
            self.segmenter
                .ranges(text)
                .map(|ranges| character_spans(text, &ranges))
                .map_err(|error| describe(&error))
        })
        .map_err(failure)
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

    /// Segments `texts` in consecutive batches of bounded total size.
    fn tokenize_batches(
        &self,
        texts: &[String],
        threads: NonZeroUsize,
    ) -> Result<Vec<Vec<String>>, String> {
        let mut results = Vec::with_capacity(texts.len());
        for batch in batches(texts) {
            let views = batch.iter().map(String::as_str).collect::<Vec<_>>();
            let words = self
                .segmenter
                .split_many_on(&views, threads)
                .map_err(|error| describe(&error))?;
            results.extend(
                words
                    .into_iter()
                    .map(|words| words.into_iter().map(str::to_owned).collect()),
            );
        }
        Ok(results)
    }
}

/// Consecutive groups of `texts`, each holding at most [`BATCH_BYTES`] bytes unless a single text
/// is longer.
fn batches(texts: &[String]) -> Vec<&[String]> {
    let mut groups = Vec::new();
    let mut start = 0;
    let mut bytes = 0;
    for (index, text) in texts.iter().enumerate() {
        if index > start && bytes + text.len() > BATCH_BYTES {
            groups.push(&texts[start..index]);
            start = index;
            bytes = 0;
        }
        bytes += text.len();
    }
    if start < texts.len() {
        groups.push(&texts[start..]);
    }
    groups
}
