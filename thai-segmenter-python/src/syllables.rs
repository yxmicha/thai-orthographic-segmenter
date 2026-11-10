use pyo3::prelude::*;
use thai_syllable_segmenter::SyllableSegmenter;

/// Splits text into written Thai syllables.
///
/// The syllables are slices of the input, so concatenating them gives the input back. Building a
/// tokenizer takes a few milliseconds; build one and reuse it.
#[pyclass(frozen, module = "thai_orthographic_segmenter")]
pub(crate) struct SyllableTokenizer {
    segmenter: SyllableSegmenter,
}
