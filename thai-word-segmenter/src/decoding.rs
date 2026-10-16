//! Model-independent boundary decisions and source-preserving punctuation attachment.


use crate::{Error, dictionary::ClusterDictionary};

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

    /// Applies accepted dictionary protection after thresholding contextual BREAK scores.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Scores`] when the threshold is outside `0.0..=1.0`, or when the
    /// probabilities are not finite, outside `0.0..=1.0`, and one per syllable.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::{decoding::BoundaryScores, dictionary::ClusterDictionary};
    ///
    /// let dictionary = ClusterDictionary::read("".as_bytes())?;
    /// let breaks = BoundaryScores::new(&[0.9, 0.1, 0.9], 0.5).select_breaks(&["ก", "ข", "ค"], &dictionary)?;
    /// // The last syllable never breaks; the middle one is below the threshold.
    /// assert_eq!(breaks, [true, false, false]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn select_breaks<S: AsRef<str>>(
        &self,
        syllables: &[S],
        dictionary: &ClusterDictionary,
    ) -> Result<Vec<bool>, Error> {
        let probabilities = self.probabilities;
        if !self.threshold.is_finite() || !(0.0..=1.0).contains(&self.threshold) {
            return Err(Error::Scores("decision threshold is outside 0.0..=1.0"));
        }
        if probabilities.len() != syllables.len()
            || probabilities
                .iter()
                .any(|score| !score.is_finite() || !(0.0..=1.0).contains(score))
        {
            return Err(Error::Scores(
                "probabilities must be finite, in 0.0..=1.0, and one per syllable",
            ));
        }
        let protected = dictionary.protected_gaps(syllables);
        Ok(probabilities
            .iter()
            .zip(protected)
            .enumerate()
            .map(|(index, (probability, keep))| {
                index + 1 < syllables.len() && *probability >= self.threshold && !keep
            })
            .collect())
    }
}

/// Original source text whose characters every attached cluster must retain.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::decoding::SourceText;
///
/// let text = "ก, ข";
/// let attached = SourceText::new(text).attach_punctuation(&[0..3, 3..5, 5..8])?;
/// assert_eq!(attached, [0..5, 5..8]); // the comma stays with the word before it
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceText<'a>(&'a str);

impl<'a> SourceText<'a> {
    /// Wraps the original `text`.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::decoding::SourceText;
    ///
    /// assert_eq!(SourceText::new("abc"), SourceText::new("abc"));
    /// ```
    #[must_use]
    pub const fn new(text: &'a str) -> Self {
        Self(text)
    }
}
