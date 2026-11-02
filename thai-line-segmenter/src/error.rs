/// Failures from line-boundary normalization.
///
/// # Examples
///
/// ```
/// use thai_line_segmenter::{Error, LineSegmenter};
///
/// match LineSegmenter::with_icu_dictionary("a\0b") {
///     Err(Error::IcuDictionary(_)) => {}
///     Err(other) => panic!("unexpected error: {other}"),
///     Ok(_) => panic!("a NUL entry must be rejected"),
/// }
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The built-in segment pattern failed to compile.
    #[error("compile line segment pattern")]
    Pattern(#[source] regex::Error),
    /// An ICU dictionary entry cannot be stored: it contains a NUL character.
    #[error("invalid ICU dictionary entry")]
    IcuDictionary(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// Word segmentation of a Thai run failed.
    #[error(transparent)]
    Words(#[from] thai_word_segmenter::Error),
}
