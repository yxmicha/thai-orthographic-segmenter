use pyo3::{exceptions::PyValueError, prelude::*};
use thai_word_segmenter::WordSegmenter;

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
