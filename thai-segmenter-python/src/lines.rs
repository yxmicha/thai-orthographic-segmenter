use pyo3::prelude::*;
use thai_line_segmenter::LineSegmenter;

use crate::error::{describe, failure};

/// Marks where Thai text may and may not wrap.
///
/// The marked text inserts a zero-width space (U+200B) at each break opportunity between words
/// and a zero-width joiner (U+200D) inside words that a dictionary-based line breaker would
/// split, and it attaches punctuation to its neighbouring word. Rendering the result with
/// ordinary line breaking gives breaks at word boundaries only. Building a breaker loads a word
/// model, which takes about 40 ms; build one and reuse it.
#[pyclass(frozen, module = "thai_orthographic_segmenter")]
pub(crate) struct LineBreaker {
    segmenter: LineSegmenter,
}

#[pymethods]
impl LineBreaker {
    /// Loads the bundled models. `icu_dictionary` is an optional newline-delimited word list for
    /// the joiner insertion, replacing the bundled one.
    #[new]
    #[pyo3(signature = (icu_dictionary=None))]
    fn new(py: Python<'_>, icu_dictionary: Option<&str>) -> PyResult<Self> {
        let segmenter = py
            .detach(|| {
                match icu_dictionary {
                    Some(dictionary) => LineSegmenter::with_icu_dictionary(dictionary),
                    None => LineSegmenter::new(),
                }
                .map_err(|error| describe(&error))
            })
            .map_err(failure)?;
        Ok(Self { segmenter })
    }

    /// Returns `text` with zero-width spaces and joiners inserted for line breaking.
    fn mark(&self, py: Python<'_>, text: &str) -> PyResult<String> {
        py.detach(|| {
            self.segmenter
                .normalize_line_boundaries(text)
                .map_err(|error| describe(&error))
        })
        .map_err(failure)
    }
}
