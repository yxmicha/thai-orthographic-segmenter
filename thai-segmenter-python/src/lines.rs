use pyo3::prelude::*;
use thai_line_segmenter::LineSegmenter;

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
