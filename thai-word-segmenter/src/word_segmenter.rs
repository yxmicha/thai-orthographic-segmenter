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

impl std::fmt::Debug for WordSegmenter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WordSegmenter")
            .finish_non_exhaustive()
    }
}

impl WordSegmenter {

    /// Loads the bundled epoch-6 model with a custom newline-delimited dictionary.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DictionaryEntry`] when an entry is empty or contains non-Thai characters,
    /// or another [`Error`] when the bundled model cannot be loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::{Error, WordSegmenter};
    ///
    /// // The dictionary replaces the bundled one; a dictionary word is never split.
    /// let segmenter = WordSegmenter::with_dictionary("ปฐมกาลพระวาทะ\n")?;
    /// let words = segmenter.split("ในปฐมกาลพระวาทะทรงดำรงอยู่")?;
    /// assert!(words.iter().any(|word| word.contains("ปฐมกาลพระวาทะ")));
    /// assert!(matches!(WordSegmenter::with_dictionary("abc"), Err(Error::DictionaryEntry(_))));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_dictionary(dictionary_text: &str) -> Result<Self, Error> {
        let config: ClusterModelConfig =
            serde_json::from_str(BUNDLED_CONFIG).map_err(Error::ConfigFormat)?;
        let dictionary = ClusterDictionary::read(dictionary_text.as_bytes())?;
        let model = InferenceModel::from_weight_bytes(&config, BUNDLED_WEIGHTS)?;
        Ok(Self {
            syllables: SyllableSegmenter::new()?,
            config,
            model,
            dictionary,
        })
    }
}
