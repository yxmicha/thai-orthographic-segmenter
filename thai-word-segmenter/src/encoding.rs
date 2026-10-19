//! Identical ordered-character encoding for training and inference.

#[cfg(feature = "burn")]
use crate::{Error, model::ClusterModelConfig};

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

/// Backend-independent arrays; one allocation scope per optimizer/inference batch. Needs the
/// `burn` feature, since its only consumer is [`crate::network::ClusterInput`].
///
/// # Examples
///
/// ```
/// # use thai_word_segmenter::model::ClusterModelConfig;
/// # let config = ClusterModelConfig::new()
/// #     .with_model_dimension(32)
/// #     .with_feed_forward_dimension(64)
/// #     .with_heads(4)
/// #     .with_layers(2)
/// #     .with_character_dimension(4)
/// #     .with_syllable_dimension(8)
/// #     .with_max_characters(6)
/// #     .with_syllable_buckets(64)
/// #     .with_dropout(0.0);
/// use thai_word_segmenter::encoding::EncodedBatch;
///
/// let batch = EncodedBatch::new(&[&["การ", "เรียน"][..], &["ฉัน"][..]], &config)?;
/// // The sequences are padded to a common length that is a multiple of eight.
/// assert!(format!("{batch:?}").contains("length: 8"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[cfg(feature = "burn")]
#[derive(Clone, Debug)]
pub struct EncodedBatch {
    pub(crate) rows: usize,
    pub(crate) length: usize,
    pub(crate) characters: Vec<i64>,
    pub(crate) syllables: Vec<i64>,
    pub(crate) positions: Vec<i64>,
    pub(crate) padding: Vec<bool>,
}

#[cfg(feature = "burn")]
impl EncodedBatch {
    /// Encodes bounded syllable sequences without fitting a vocabulary on evaluation data.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] when `config` is invalid, or [`Error::Batch`] when the batch is
    /// empty or a sequence is empty or longer than the model capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use thai_word_segmenter::model::ClusterModelConfig;
    /// # let config = ClusterModelConfig::new()
    /// #     .with_model_dimension(32)
    /// #     .with_feed_forward_dimension(64)
    /// #     .with_heads(4)
    /// #     .with_layers(2)
    /// #     .with_character_dimension(4)
    /// #     .with_syllable_dimension(8)
    /// #     .with_max_characters(6)
    /// #     .with_syllable_buckets(64)
    /// #     .with_dropout(0.0);
    /// use thai_word_segmenter::{Error, encoding::EncodedBatch};
    ///
    /// assert!(EncodedBatch::new(&[&["การ"][..]], &config).is_ok());
    /// // A sequence must have between one syllable and the model capacity.
    /// let empty: &[&str] = &[];
    /// assert!(matches!(EncodedBatch::new(&[empty], &config), Err(Error::Batch(_))));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<S: AsRef<str>>(
        sequences: &[&[S]],
        config: &ClusterModelConfig,
    ) -> Result<Self, Error> {
        config.validate()?;
        // Pad to a multiple of 8, not to the exact length. Padded positions are masked, so results
        // do not change, but other row counts miss the matrix-multiply fast path: dropping this
        // rounding made inference 5.7 times slower on the benchmark corpus.
        let length = sequences
            .iter()
            .map(|sequence| sequence.len())
            .max()
            .ok_or(Error::Batch("cannot encode an empty cluster batch"))?
            .next_multiple_of(8);
        if length == 0
            || length > config.max_syllables
            || sequences.iter().any(|sequence| sequence.is_empty())
        {
            return Err(Error::Batch(
                "cluster sequence length outside model capacity",
            ));
        }
        let rows = sequences.len();
        let cells = rows
            .checked_mul(length)
            .ok_or(Error::Batch("cluster batch size overflow"))?;
        let mut result = Self {
            rows,
            length,
            characters: vec![
                0;
                cells
                    .checked_mul(config.max_characters)
                    .ok_or(Error::Batch("character batch size overflow"))?
            ],
            syllables: vec![0; cells],
            positions: vec![0; cells],
            padding: vec![true; cells],
        };
        let buckets = u64::try_from(config.syllable_buckets - 1)
            .map_err(|_| Error::Batch("syllable vocabulary overflow"))?;
        for (row, sequence) in sequences.iter().enumerate() {
            for (column, syllable) in sequence.iter().enumerate() {
                let cell = row * length + column;
                result.padding[cell] = false;
                result.positions[cell] =
                    i64::try_from(column).map_err(|_| Error::Batch("position overflow"))?;
                let features = SyllableFeatures::new(syllable.as_ref());
                result.syllables[cell] = i64::try_from(features.bucket(buckets))
                    .map_err(|_| Error::Batch("syllable hash overflow"))?;
                for (slot, id) in features.character_ids(config.max_characters).enumerate() {
                    result.characters[cell * config.max_characters + slot] = id;
                }
            }
        }
        Ok(result)
    }
}
