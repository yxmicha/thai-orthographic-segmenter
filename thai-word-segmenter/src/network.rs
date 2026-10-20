//! The Burn network behind the bundled model, for training and for checking the CPU engine.
//!
//! Everything here is built on Burn, so these types follow Burn's stability, not this crate's.
//! The module exists only with the `burn` feature; the bundled model and the segmenters need
//! none of it.

#![cfg_attr(not(feature = "burn"), allow(dead_code))]

use burn::{
    module::{Module, ModuleVisitor, Param},
    store::ModuleRecord,
    tensor::{Bool, Bytes, Device, Int, Tensor, TensorData},
};
use burn_nn::{
    Embedding, EmbeddingConfig, Linear, LinearConfig,
    transformer::{TransformerEncoder, TransformerEncoderConfig, TransformerEncoderInput},
};

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
