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
