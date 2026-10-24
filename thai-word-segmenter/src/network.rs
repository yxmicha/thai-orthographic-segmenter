//! The Burn network behind the bundled model, for training and for checking the CPU engine.
//!
//! Everything here is built on Burn, so these types follow Burn's stability, not this crate's.
//! The module exists only with the `burn` feature; the bundled model and the segmenters need
//! none of it.

#![cfg_attr(not(feature = "burn"), allow(dead_code))]

use std::path::Path;

use burn::{
    module::{Module, ModuleVisitor, Param},
    store::ModuleRecord,
    tensor::{Bool, Bytes, Device, Int, Tensor, TensorData},
};
use burn_nn::{
    Embedding, EmbeddingConfig, Linear, LinearConfig,
    transformer::{TransformerEncoder, TransformerEncoderConfig, TransformerEncoderInput},
};

use crate::{
    Error,
    encoding::{CHARACTER_VOCABULARY_SIZE, EncodedBatch},
    model::ClusterModelConfig,
};

/// Padded features consumed by the contextual model.
///
/// # Examples
///
/// ```
/// # use burn::tensor::Device;
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
/// # let device = Device::flex();
/// use thai_word_segmenter::{encoding::EncodedBatch, network::{ClusterInput, ClusterModel}};
///
/// let input = ClusterInput::new(EncodedBatch::new(&[&["การ", "เรียน"][..]], &config)?, &device);
/// let logits = ClusterModel::new(&config, &device)?.forward(input);
/// assert_eq!(logits.dims(), [1, 8, 2]); // syllables are padded to a multiple of eight
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct ClusterInput {
    /// Character IDs with shape `[batch, syllables * character_slots]`.
    pub(crate) characters: Tensor<2, Int>,
    /// Whole-syllable hashes with shape `[batch, syllables]`.
    pub(crate) syllables: Tensor<2, Int>,
    /// Relative syllable positions.
    pub(crate) positions: Tensor<2, Int>,
    /// Padding mask, true at missing syllables.
    pub(crate) padding: Tensor<2, Bool>,
}

impl ClusterInput {
    /// Materializes tensors on `device` from an encoded batch.
    ///
    /// # Examples
    ///
    /// ```
    /// # use burn::tensor::Device;
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
    /// use thai_word_segmenter::{encoding::EncodedBatch, network::ClusterInput};
    ///
    /// let batch = EncodedBatch::new(&[&["การ"][..]], &config)?;
    /// let input = ClusterInput::new(batch, &Device::flex());
    /// # drop(input);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn new(batch: EncodedBatch, device: &Device) -> Self {
        let shape = [batch.rows, batch.length];
        let character_slots = batch.characters.len() / batch.rows;
        Self {
            characters: Tensor::from_data(
                TensorData::new(batch.characters, [batch.rows, character_slots]),
                device,
            ),
            syllables: Tensor::from_data(TensorData::new(batch.syllables, shape), device),
            positions: Tensor::from_data(TensorData::new(batch.positions, shape), device),
            padding: Tensor::from_data(TensorData::new(batch.padding, shape), device),
        }
    }
}

/// Scores KEEP/BREAK after each syllable while retaining its internal character order.
///
/// # Examples
///
/// ```
/// # use burn::tensor::Device;
/// # use thai_word_segmenter::{model::ClusterModelConfig, network::ClusterModel};
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
/// # let device = Device::flex();
/// let model = ClusterModel::new(&config, &device)?;
/// let copy = model.clone();
/// # drop(copy);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Module, Debug)]
pub struct ClusterModel {
    pub(crate) characters: Embedding,
    pub(crate) syllables: Embedding,
    pub(crate) projection: Linear,
    pub(crate) positions: Embedding,
    pub(crate) encoder: TransformerEncoder,
    pub(crate) boundary: Linear,
}

impl ClusterModel {
    /// Initializes a network with the dimensions of `config`, for CPU inference or training.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] when the configuration is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use burn::tensor::Device;
    /// # use thai_word_segmenter::{model::ClusterModelConfig, network::ClusterModel};
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
    /// # let device = Device::flex();
    /// let model = ClusterModel::new(&config, &device)?;
    /// # drop(model);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(config: &ClusterModelConfig, device: &Device) -> Result<Self, Error> {
        config.validate()?;
        let ordered_width = config
            .max_characters
            .checked_mul(config.character_dimension)
            .and_then(|width| width.checked_add(config.syllable_dimension))
            .ok_or(Error::Config("cluster input width overflow"))?;
        let model = Self {
            characters: EmbeddingConfig::new(CHARACTER_VOCABULARY_SIZE, config.character_dimension)
                .init(device),
            syllables: EmbeddingConfig::new(config.syllable_buckets, config.syllable_dimension)
                .init(device),
            projection: LinearConfig::new(ordered_width, config.model_dimension).init(device),
            positions: EmbeddingConfig::new(config.max_syllables, config.model_dimension)
                .init(device),
            encoder: TransformerEncoderConfig::new(
                config.model_dimension,
                config.feed_forward_dimension,
                config.heads,
                config.layers,
            )
            .with_dropout(config.dropout)
            .with_norm_first(true)
            .init(device),
            boundary: LinearConfig::new(config.model_dimension, 2).init(device),
        };
        // Burn clones uninitialized parameters independently; initialize before a
        // checkpoint clone can produce different random weights from this model.
        model.visit(&mut InitializeParameters);
        Ok(model)
    }

    /// Saves full-precision weights so deployment does not silently quantize decisions.
    ///
    /// # Errors
    ///
    /// Returns [`Error::CheckpointSave`] when the record cannot be written to `path`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use burn::tensor::Device;
    /// # use thai_word_segmenter::{model::ClusterModelConfig, network::ClusterModel};
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
    /// # let device = Device::flex();
    /// let model = ClusterModel::new(&config, &device)?;
    /// let directory = tempfile::tempdir()?;
    /// model.save(directory.path().join("model.bpk"))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        self.clone()
            .into_record()
            .save(path)
            .map_err(|error| Error::CheckpointSave(Box::new(error)))
    }

    /// Returns two-class logits with shape `[batch, syllables, 2]`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use burn::tensor::Device;
    /// # use thai_word_segmenter::{model::ClusterModelConfig, network::ClusterModel};
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
    /// # let device = Device::flex();
    /// use thai_word_segmenter::{encoding::EncodedBatch, network::ClusterInput};
    ///
    /// let input = ClusterInput::new(EncodedBatch::new(&[&["การ", "เรียน"][..]], &config)?, &device);
    /// let logits = ClusterModel::new(&config, &device)?.forward(input);
    /// assert_eq!(logits.dims(), [1, 8, 2]); // KEEP and BREAK logits per syllable slot
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn forward(&self, input: ClusterInput) -> Tensor<3> {
        let [batch, length] = input.syllables.dims();
        let [_, character_slots] = input.characters.dims();
        let embeddings = self.characters.forward(input.characters);
        let [_, _, dimension] = embeddings.dims();
        let ordered = embeddings.reshape([batch, length, character_slots / length * dimension]);
        let whole = self.syllables.forward(input.syllables);
        let projected = self
            .projection
            .forward(Tensor::cat(vec![ordered, whole], 2))
            + self.positions.forward(input.positions);
        let encoded = self
            .encoder
            .forward(TransformerEncoderInput::new(projected).mask_pad(input.padding));
        self.boundary.forward(encoded)
    }
}

struct InitializeParameters;

impl ModuleVisitor for InitializeParameters {
    fn visit_float<const D: usize>(&mut self, parameter: &Param<Tensor<D>>) {
        drop(parameter.val());
    }
}
