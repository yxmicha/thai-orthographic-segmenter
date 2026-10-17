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
const FNV_PRIME: u64 = 0x100_0000_01b3;

/// Hashed and ordered character features of one syllable, shared by every encoder so training
/// and inference see identical inputs.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::encoding::SyllableFeatures;
///
/// let features = SyllableFeatures::new("กา");
/// assert!((1..=16383).contains(&features.bucket(16383)));
/// assert_eq!(features.character_ids(4).count(), 2);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SyllableFeatures<'a>(&'a str);

impl<'a> SyllableFeatures<'a> {
    /// Wraps one `syllable`.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::encoding::SyllableFeatures;
    ///
    /// assert_eq!(SyllableFeatures::new("กา"), SyllableFeatures::new("กา"));
    /// ```
    #[must_use]
    pub const fn new(syllable: &'a str) -> Self {
        Self(syllable)
    }

    /// Whole-syllable hash bucket in `1..=buckets`; bucket 0 is reserved for padding.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::encoding::SyllableFeatures;
    ///
    /// // Stable across runs and platforms; bucket 0 is reserved for padding.
    /// let bucket = SyllableFeatures::new("กา").bucket(16383);
    /// assert_eq!(bucket, SyllableFeatures::new("กา").bucket(16383));
    /// assert!(bucket >= 1);
    /// ```
    #[must_use]
    pub fn bucket(&self, buckets: u64) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        for byte in self.0.bytes() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME);
        }
        hash % buckets + 1
    }

    /// Character IDs for the first `slots` ordered slots. A syllable longer than `slots`
    /// characters keeps both ends. Slots past the returned length are padding, ID 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::encoding::SyllableFeatures;
    ///
    /// // One id per character slot, as many as the syllable has characters up to `slots`.
    /// let ids = SyllableFeatures::new("กา").character_ids(4).collect::<Vec<_>>();
    /// assert_eq!(ids.len(), 2);
    /// assert!(ids.iter().all(|id| *id > 0));
    /// ```
    pub fn character_ids(&self, slots: usize) -> impl Iterator<Item = i64> {
        let characters = self.0.chars().collect::<Vec<_>>();
        (0..slots.min(characters.len())).map(move |slot| {
            let source = if characters.len() > slots && slot >= slots / 2 {
                characters.len() - slots + slot
            } else {
                slot
            };
            let value = u32::from(characters[source]);
            if (0x0e00..=0x0e7f).contains(&value) {
                i64::from(value - 0x0e00 + 1)
            } else {
                OTHER_CHARACTER_ID
            }
        })
    }
}
