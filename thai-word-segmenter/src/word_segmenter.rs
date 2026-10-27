use thai_syllable_segmenter::SyllableSegmenter;

use crate::{
    Error,
    decoding::{BoundaryScores, SourceText},
    dictionary::ClusterDictionary,
    inference::InferenceModel,
    model::{CONTEXT_SYLLABLES, ClusterModelConfig, OWNED_SYLLABLES},
    text_groups::TextGroups,
    text_runs::TextRuns,
};

const BUNDLED_CONFIG: &str = include_str!("../model/config.json");
const BUNDLED_DICTIONARY: &str = include_str!("../model/dictionary.txt");
const BUNDLED_WEIGHTS: &[u8] = include_bytes!("../model/model.safetensors");

/// Reusable CPU word segmenter: syllables from `thai-syllable-segmenter`, grouped by a contextual
/// cluster model with accepted-dictionary protection and punctuation attachment.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::WordSegmenter;
///
/// let segmenter = WordSegmenter::new()?;
/// let text = "ในปฐมกาลพระวาทะทรงดำรงอยู่";
/// let words = segmenter.split(text)?;
/// assert_eq!(words.concat(), text);
/// assert_eq!(words[..3], ["ในปฐมกาล", "พระวาทะ", "ทรงดำรง"]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct WordSegmenter {
    syllables: SyllableSegmenter,
    config: ClusterModelConfig,
    model: InferenceModel,
    dictionary: ClusterDictionary,
}
