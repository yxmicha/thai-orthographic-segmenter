//! Model dimensions, the window geometry shared by every model, and the checkpoint configuration.

use serde::{Deserialize, Serialize};

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
}
