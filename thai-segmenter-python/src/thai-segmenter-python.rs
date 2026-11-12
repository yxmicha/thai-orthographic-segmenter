#![warn(missing_docs)]
//! Python bindings for the Thai syllable, word and line segmenters.
//!
//! The extension module `thai_orthographic_segmenter._native` exposes one tokenizer class per segmenter and
//! the rule-based parts of `thai-syllable-rules`. The `thai_orthographic_segmenter` package wraps them in
//! `syllable_tokenize`, `word_tokenize` and the other tokenizer functions.
//! Every call releases the Python interpreter lock while it segments.

mod error;
mod lines;
mod parts;
mod spans;
mod syllables;
mod words;

use pyo3::prelude::*;

/// The native half of the `thai_orthographic_segmenter` package.
#[pymodule(name = "_native")]
fn native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "SegmenterError",
        module.py().get_type::<error::SegmenterError>(),
    )?;
    module.add_class::<syllables::SyllableTokenizer>()?;
    module.add_class::<words::WordTokenizer>()?;
    module.add_class::<lines::LineBreaker>()?;
    module.add_function(wrap_pyfunction!(parts::orthographic_parts, module)?)?;
    Ok(())
}
