use std::{io, path::PathBuf};

/// Failures from loading a word model or segmenting text.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::{Error, WordSegmenter};
///
/// // Match the failures you handle; the enum is `#[non_exhaustive]`.
/// match WordSegmenter::with_dictionary("abc") {
///     Err(Error::DictionaryEntry(entry)) => assert_eq!(entry, "abc"),
///     Err(other) => panic!("unexpected error: {other}"),
///     Ok(_) => panic!("a non-Thai entry must be rejected"),
/// }
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Syllable segmentation of the input failed.
    #[error(transparent)]
    Syllables(#[from] thai_syllable_segmenter::Error),
    /// An artifact file could not be opened.
    #[error("open {}", path.display())]
    Open {
        /// Artifact file that could not be opened.
        path: PathBuf,
        /// Underlying I/O failure.
        #[source]
        source: io::Error,
    },
    /// The model configuration is not valid JSON for [`crate::model::ClusterModelConfig`].
    #[error("parse cluster model configuration")]
    ConfigFormat(#[source] serde_json::Error),
    /// The model configuration has dimensions that cannot be encoded or attended safely.
    #[error("invalid cluster model configuration: {0}")]
    Config(&'static str),
    /// A dictionary line could not be read.
    #[error("read cluster dictionary")]
    DictionaryRead(#[source] io::Error),
    /// A dictionary entry is empty or contains non-Thai characters.
    #[error("dictionary entry {0:?} is empty or contains non-Thai characters")]
    DictionaryEntry(String),
    /// Model weights could not be written.
    #[error("save cluster checkpoint")]
    CheckpointSave(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// Model weights could not be read.
    #[error("load cluster checkpoint")]
    CheckpointLoad(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// A checkpoint could not be read or does not match the model configuration.
    #[error("read cluster checkpoint")]
    Checkpoint(#[from] thai_syllable_segmenter::inference::CheckpointError),
    /// The inference engine received operands that do not fit the model's shape.
    #[error(transparent)]
    Shape(#[from] thai_syllable_segmenter::inference::ShapeError),
    /// Syllable sequences cannot be encoded into one model batch.
    #[error("invalid cluster batch: {0}")]
    Batch(&'static str),
    /// Boundary probabilities or their threshold are outside `0.0..=1.0` or misaligned.
    #[error("invalid cluster boundary scores: {0}")]
    Scores(&'static str),
    /// Cluster ranges do not partition the source text.
    #[error("invalid cluster ranges: {0}")]
    Ranges(&'static str),
}
