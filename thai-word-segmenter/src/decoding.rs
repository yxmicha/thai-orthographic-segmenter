//! Model-independent boundary decisions and source-preserving punctuation attachment.


/// Contextual probabilities and their validation-selected decision threshold.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::{decoding::BoundaryScores, dictionary::ClusterDictionary};
///
/// let dictionary = ClusterDictionary::read("ปฐมกาล\n".as_bytes())?;
/// let syllables = ["ใน", "ปฐม", "กาล"];
/// // Only the gap after `ใน` may break: the gap inside the dictionary word is protected.
/// let scores = BoundaryScores::new(&[0.9, 0.9, 0.9], 0.5);
/// assert_eq!(scores.select_breaks(&syllables, &dictionary)?, [true, false, false]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundaryScores<'a> {
    probabilities: &'a [f32],
    threshold: f32,
}

impl<'a> BoundaryScores<'a> {
    /// Pairs the BREAK `probabilities` after each syllable with the `threshold` a probability
    /// must reach to allow a break. Nothing is validated until [`Self::select_breaks`].
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::decoding::BoundaryScores;
    ///
    /// let scores = BoundaryScores::new(&[0.2, 0.8], 0.5);
    /// # drop(scores);
    /// ```
    #[must_use]
    pub const fn new(probabilities: &'a [f32], threshold: f32) -> Self {
        Self {
            probabilities,
            threshold,
        }
    }
}
