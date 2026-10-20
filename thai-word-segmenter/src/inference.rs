//! CPU forward pass equivalent to the Burn network's (`network::ClusterModel::forward`, with the
//! `burn` feature) up to float rounding, specialised for inference: no padding, no tensor-graph copies, weights packed once, and the character
//! embedding folded into the input projection.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

use thai_syllable_segmenter::inference::{
    BlockPool, Checkpoint, EncoderLayer, InputBlocks, PackedMatrix, Scratch, WeightMatrix,
    run_on_cpu,
};

use crate::{
    Error,
    encoding::{CHARACTER_VOCABULARY_SIZE, SyllableFeatures},
    model::ClusterModelConfig,
};

/// Rows processed together by one forward pass; bounds scratch memory and keeps it cache-sized.
const ROW_BUDGET: usize = 512;

/// Inference weights read from a trained checkpoint.
///
/// # Examples
///
/// ```no_run
/// # use thai_word_segmenter::{inference::InferenceModel, model::ClusterModelConfig};
/// # let config: ClusterModelConfig = serde_json::from_slice(&std::fs::read("config.json")?)?;
/// # let inference = InferenceModel::from_weight_bytes(&config, &std::fs::read("model.safetensors")?)?;
/// let scores = inference.differences(&[&["การ", "เรียน"][..], &["ฉัน"][..]])?;
/// assert_eq!(scores.iter().map(Vec::len).collect::<Vec<_>>(), [2, 1]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct InferenceModel {
    model_dimension: usize,
    character_slots: usize,
    syllable_buckets: u64,
    max_syllables: usize,
    slot_features: SlotFeatures,
    /// `[bucket][syllable_dimension]`.
    whole_embeddings: Vec<f32>,
    whole_projection: PackedMatrix,
    input_bias: Vec<f32>,
    /// `[position][model_dimension]`.
    positions: Vec<f32>,
    layers: Vec<EncoderLayer>,
    boundary: PackedMatrix,
    blocks: BlockPool,
}

/// Character embeddings already multiplied by the input-projection rows of their slot, so the
/// ordered character features of a syllable are a sum of table rows.
struct SlotFeatures {
    model_dimension: usize,
    /// `[slot][character id][model_dimension]`.
    characters: Vec<f32>,
    /// `[first padded slot][model_dimension]`: summed features of the padding character in every slot
    /// from the index to the last one.
    padding_tail: Vec<f32>,
}

impl SlotFeatures {
    /// Multiplies each slot's share of the input projection `rows` by the character embeddings.
    fn new(
        embeddings: &[f32],
        projection: &WeightMatrix,
        config: &ClusterModelConfig,
    ) -> Result<Self, Error> {
        let model_dimension = config.model_dimension;
        let slot_width = config.character_dimension * model_dimension;
        let mut characters =
            vec![0.0; config.max_characters * CHARACTER_VOCABULARY_SIZE * model_dimension];
        let mut blocks = InputBlocks::default();
        for (rows, features) in projection
            .values()
            .chunks_exact(slot_width)
            .zip(characters.chunks_exact_mut(CHARACTER_VOCABULARY_SIZE * model_dimension))
        {
            PackedMatrix::new(
                rows,
                config.character_dimension,
                model_dimension,
                vec![0.0; model_dimension],
            )?
            .multiply_into::<false>(embeddings, &mut blocks, features)?;
        }
        let mut padding_tail = vec![0.0; (config.max_characters + 1) * model_dimension];
        for (slot, padding) in characters
            .chunks_exact(CHARACTER_VOCABULARY_SIZE * model_dimension)
            .enumerate()
            .rev()
        {
            let (current, later) =
                padding_tail[slot * model_dimension..].split_at_mut(model_dimension);
            for ((total, next), feature) in current.iter_mut().zip(later).zip(padding) {
                *total = *next + feature;
            }
        }
        Ok(Self {
            model_dimension,
            characters,
            padding_tail,
        })
    }

    /// Features of the character with `id` in `slot`.
    fn character(&self, slot: usize, id: usize) -> Result<&[f32], Error> {
        let start = (slot * CHARACTER_VOCABULARY_SIZE + id) * self.model_dimension;
        self.characters
            .get(start..start + self.model_dimension)
            .ok_or(Error::Batch("character slot outside the model"))
    }

    /// Summed features of the padding character in every slot from `first_padded` onward.
    fn padding(&self, first_padded: usize) -> Result<&[f32], Error> {
        self.padding_tail
            .get(first_padded * self.model_dimension..(first_padded + 1) * self.model_dimension)
            .ok_or(Error::Batch("padding slot outside the model"))
    }
}

impl std::fmt::Debug for InferenceModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InferenceModel")
            .finish_non_exhaustive()
    }
}

impl InferenceModel {

    /// Builds a model from the named tensors of `checkpoint`, folding the character embeddings
    /// into the input projection.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] when `config` is invalid, [`Error::Checkpoint`] when the
    /// checkpoint lacks a tensor of these dimensions or holds an unused one, or another
    /// [`Error`] when the weights do not fit the engine.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use thai_syllable_segmenter::inference::Checkpoint;
    /// use thai_word_segmenter::{inference::InferenceModel, model::ClusterModelConfig};
    ///
    /// let config: ClusterModelConfig = serde_json::from_slice(&std::fs::read("config.json")?)?;
    /// let checkpoint = Checkpoint::from_bytes(&std::fs::read("model.safetensors")?)?;
    /// let model = InferenceModel::from_checkpoint(&config, checkpoint)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_checkpoint(
        config: &ClusterModelConfig,
        mut checkpoint: Checkpoint,
    ) -> Result<Self, Error> {
        config.validate()?;
        let ordered_width = config
            .max_characters
            .checked_mul(config.character_dimension)
            .and_then(|width| width.checked_add(config.syllable_dimension))
            .ok_or(Error::Config("cluster input width overflow"))?;
        let embeddings = checkpoint
            .embedding(
                "characters",
                CHARACTER_VOCABULARY_SIZE,
                config.character_dimension,
            )?
            .into_values();
        let whole_embeddings = checkpoint
            .embedding(
                "syllables",
                config.syllable_buckets,
                config.syllable_dimension,
            )?
            .into_values();
        let projection = checkpoint.linear("projection", ordered_width, config.model_dimension)?;
        let positions = checkpoint
            .embedding("positions", config.max_syllables, config.model_dimension)?
            .into_values();
        let layers = (0..config.layers)
            .map(|index| {
                checkpoint.encoder_layer(
                    &format!("encoder.layers.{index}"),
                    config.model_dimension,
                    config.feed_forward_dimension,
                    config.heads,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let boundary = checkpoint.linear("boundary", config.model_dimension, 2)?;
        checkpoint.finish()?;
        let slot_features = SlotFeatures::new(&embeddings, &projection, config)?;
        let whole_start =
            config.max_characters * config.character_dimension * config.model_dimension;
        let whole_weights = projection
            .values()
            .get(whole_start..)
            .ok_or(Error::Config("projection has no whole-syllable rows"))?;
        Ok(Self {
            model_dimension: config.model_dimension,
            character_slots: config.max_characters,
            syllable_buckets: u64::try_from(config.syllable_buckets - 1)
                .map_err(|_| Error::Batch("syllable vocabulary overflow"))?,
            max_syllables: config.max_syllables,
            slot_features,
            whole_embeddings,
            whole_projection: PackedMatrix::new(
                whole_weights,
                config.syllable_dimension,
                config.model_dimension,
                vec![0.0; config.model_dimension],
            )?,
            input_bias: projection.into_bias(),
            positions,
            layers,
            boundary: boundary.try_into()?,
            blocks: BlockPool::default(),
        })
    }
}
