//! Identical ordered-character encoding for training and inference.


/// Character embedding rows: padding, the 128 code points of the Thai block, and one row shared by
/// every other character.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::encoding::CHARACTER_VOCABULARY_SIZE;
///
/// // Padding, the 128 Thai block code points and one id for everything else.
/// assert_eq!(CHARACTER_VOCABULARY_SIZE, 130);
/// ```
pub const CHARACTER_VOCABULARY_SIZE: usize = 130;
const OTHER_CHARACTER_ID: i64 = 129;
/// FNV-1a offset basis and prime, hashing syllable bytes into embedding buckets.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
