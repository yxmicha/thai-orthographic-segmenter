//! Model dimensions, the window geometry shared by every model, and the checkpoint configuration.

use serde::{Deserialize, Serialize};

use crate::Error;

/// Syllables whose following gaps one inference window decides.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::model::OWNED_SYLLABLES;
///
/// assert_eq!(OWNED_SYLLABLES, 96);
/// ```
pub const OWNED_SYLLABLES: usize = 96;
/// Syllables of surrounding context on each side of the owned syllables.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::model::CONTEXT_SYLLABLES;
///
/// assert_eq!(CONTEXT_SYLLABLES, 16);
/// ```
pub const CONTEXT_SYLLABLES: usize = 16;
/// Syllable capacity of every model: the owned syllables plus context on both sides.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::model::{CONTEXT_SYLLABLES, OWNED_SYLLABLES, WINDOW_SYLLABLES};
///
/// assert_eq!(WINDOW_SYLLABLES, OWNED_SYLLABLES + 2 * CONTEXT_SYLLABLES);
/// ```
pub const WINDOW_SYLLABLES: usize = OWNED_SYLLABLES + 2 * CONTEXT_SYLLABLES;
/// Upper bound on ordered character slots per syllable.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::model::MAX_CHARACTER_SLOTS;
///
/// assert_eq!(MAX_CHARACTER_SLOTS, 128);
/// ```
pub const MAX_CHARACTER_SLOTS: usize = 128;

/// Model dimensions serialized alongside each checkpoint.
///
/// Start from the defaults with [`ClusterModelConfig::new`] and override what differs with the
/// `with_*` methods. Deserializing fills any missing field with its default, so a configuration
/// file may list only the fields it changes.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::model::ClusterModelConfig;
///
/// let config = ClusterModelConfig::new().with_model_dimension(32).with_heads(4);
/// assert_eq!(config.model_dimension(), 32);
/// let json = serde_json::to_string(&config)?;
/// assert_eq!(serde_json::from_str::<ClusterModelConfig>(&json)?, config);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClusterModelConfig {
    /// Maximum syllables in one contextual window.
    pub(crate) max_syllables: usize,
    /// Ordered character slots per syllable; longer forms retain both ends.
    pub(crate) max_characters: usize,
    /// Width of each character embedding.
    pub(crate) character_dimension: usize,
    /// Number of stable syllable hash buckets, including padding.
    pub(crate) syllable_buckets: usize,
    /// Width of the whole-syllable embedding.
    pub(crate) syllable_dimension: usize,
    /// Transformer width.
    pub(crate) model_dimension: usize,
    /// Transformer feed-forward width.
    pub(crate) feed_forward_dimension: usize,
    /// Attention heads per layer.
    pub(crate) heads: usize,
    /// Number of contextual layers.
    pub(crate) layers: usize,
    /// Training dropout; zero for inference.
    pub(crate) dropout: f64,
    /// Validation-selected BREAK probability threshold.
    pub(crate) break_threshold: f32,
}

impl Default for ClusterModelConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ClusterModelConfig {
    /// The default dimensions of the bundled model family.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// let config = ClusterModelConfig::new();
    /// assert_eq!(config.model_dimension(), 192);
    /// assert_eq!(config, ClusterModelConfig::default());
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_syllables: 128,
            max_characters: 48,
            character_dimension: 16,
            syllable_buckets: 16384,
            syllable_dimension: 32,
            model_dimension: 192,
            feed_forward_dimension: 768,
            heads: 6,
            layers: 4,
            dropout: 0.1,
            break_threshold: 0.5,
        }
    }

    /// Maximum syllables in one contextual window.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// assert_eq!(ClusterModelConfig::new().max_syllables(), 128);
    /// ```
    #[must_use]
    pub const fn max_syllables(&self) -> usize {
        self.max_syllables
    }

    /// Sets the maximum syllables in one contextual window.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// let config = ClusterModelConfig::new().with_max_syllables(64);
    /// assert_eq!(config.max_syllables(), 64);
    /// ```
    #[must_use]
    pub const fn with_max_syllables(mut self, max_syllables: usize) -> Self {
        self.max_syllables = max_syllables;
        self
    }

    /// Ordered character slots per syllable; longer forms retain both ends.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// assert_eq!(ClusterModelConfig::new().max_characters(), 48);
    /// ```
    #[must_use]
    pub const fn max_characters(&self) -> usize {
        self.max_characters
    }

    /// Sets the ordered character slots per syllable; longer forms retain both ends.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// let config = ClusterModelConfig::new().with_max_characters(6);
    /// assert_eq!(config.max_characters(), 6);
    /// ```
    #[must_use]
    pub const fn with_max_characters(mut self, max_characters: usize) -> Self {
        self.max_characters = max_characters;
        self
    }

    /// Width of each character embedding.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// assert_eq!(ClusterModelConfig::new().character_dimension(), 16);
    /// ```
    #[must_use]
    pub const fn character_dimension(&self) -> usize {
        self.character_dimension
    }

    /// Sets the width of each character embedding.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// let config = ClusterModelConfig::new().with_character_dimension(4);
    /// assert_eq!(config.character_dimension(), 4);
    /// ```
    #[must_use]
    pub const fn with_character_dimension(mut self, character_dimension: usize) -> Self {
        self.character_dimension = character_dimension;
        self
    }

    /// Number of stable syllable hash buckets, including padding.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// assert_eq!(ClusterModelConfig::new().syllable_buckets(), 16384);
    /// ```
    #[must_use]
    pub const fn syllable_buckets(&self) -> usize {
        self.syllable_buckets
    }

    /// Sets the number of stable syllable hash buckets, including padding.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// let config = ClusterModelConfig::new().with_syllable_buckets(64);
    /// assert_eq!(config.syllable_buckets(), 64);
    /// ```
    #[must_use]
    pub const fn with_syllable_buckets(mut self, syllable_buckets: usize) -> Self {
        self.syllable_buckets = syllable_buckets;
        self
    }

    /// Width of the whole-syllable embedding.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// assert_eq!(ClusterModelConfig::new().syllable_dimension(), 32);
    /// ```
    #[must_use]
    pub const fn syllable_dimension(&self) -> usize {
        self.syllable_dimension
    }

    /// Sets the width of the whole-syllable embedding.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// let config = ClusterModelConfig::new().with_syllable_dimension(8);
    /// assert_eq!(config.syllable_dimension(), 8);
    /// ```
    #[must_use]
    pub const fn with_syllable_dimension(mut self, syllable_dimension: usize) -> Self {
        self.syllable_dimension = syllable_dimension;
        self
    }

    /// Transformer width.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// assert_eq!(ClusterModelConfig::new().model_dimension(), 192);
    /// ```
    #[must_use]
    pub const fn model_dimension(&self) -> usize {
        self.model_dimension
    }

    /// Sets the transformer width.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::model::ClusterModelConfig;
    ///
    /// let config = ClusterModelConfig::new().with_model_dimension(32);
    /// assert_eq!(config.model_dimension(), 32);
    /// ```
    #[must_use]
    pub const fn with_model_dimension(mut self, model_dimension: usize) -> Self {
        self.model_dimension = model_dimension;
        self
    }

    /// Rejects dimensions that cannot be encoded or attended safely.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] when a dimension is zero or out of range, the model dimension is
    /// not a multiple of the head count, or the dropout or break threshold is outside its range.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::{Error, model::ClusterModelConfig};
    ///
    /// assert!(ClusterModelConfig::new().validate().is_ok());
    /// // The model dimension must divide evenly among the heads.
    /// assert!(matches!(
    ///     ClusterModelConfig::new().with_heads(7).validate(),
    ///     Err(Error::Config(_))
    /// ));
    /// ```
    pub fn validate(&self) -> Result<(), Error> {
        if [
            self.max_syllables,
            self.max_characters,
            self.character_dimension,
            self.syllable_dimension,
            self.model_dimension,
            self.feed_forward_dimension,
            self.heads,
            self.layers,
        ]
        .contains(&0)
            || self.syllable_buckets < 2
            || self.max_syllables != WINDOW_SYLLABLES
            || self.max_characters > MAX_CHARACTER_SLOTS
            || !self.model_dimension.is_multiple_of(self.heads)
            || !self.dropout.is_finite()
            || !(0.0..1.0).contains(&self.dropout)
            || !self.break_threshold.is_finite()
            || !(0.0..=1.0).contains(&self.break_threshold)
        {
            return Err(Error::Config("invalid lexical cluster model dimensions"));
        }
        Ok(())
    }
}
