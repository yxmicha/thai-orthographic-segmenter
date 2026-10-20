//! CPU forward pass equivalent to the Burn network's (`network::ClusterModel::forward`, with the
//! `burn` feature) up to float rounding, specialised for inference: no padding, no tensor-graph copies, weights packed once, and the character
//! embedding folded into the input projection.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

#[cfg(feature = "burn")]
use crate::network::ClusterModel;
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
    /// Reads a trained model from safetensors checkpoint `bytes`, folding the character embeddings
    /// into the input projection.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] when `config` is invalid, [`Error::Checkpoint`] when `bytes` are
    /// not a checkpoint of a model with these dimensions, or another [`Error`] when the weights
    /// do not fit the engine.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use thai_word_segmenter::{inference::InferenceModel, model::ClusterModelConfig};
    ///
    /// let config: ClusterModelConfig = serde_json::from_slice(&std::fs::read("config.json")?)?;
    /// let model = InferenceModel::from_weight_bytes(&config, &std::fs::read("model.safetensors")?)?;
    /// let scores = model.differences(&[&["การ", "เรียน"][..]])?;
    /// assert_eq!(scores[0].len(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_weight_bytes(config: &ClusterModelConfig, bytes: &[u8]) -> Result<Self, Error> {
        Self::from_checkpoint(config, Checkpoint::from_bytes(bytes)?)
    }

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

    /// Extracts the weights of a trained Burn model, folding character embeddings into the input
    /// projection. Consumes the model; clone it first to keep using it. Needs the `burn` feature.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] when `config` is invalid, or another [`Error`] when the model's
    /// weights cannot be serialized or do not fit.
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
    /// use thai_word_segmenter::{inference::InferenceModel, network::ClusterModel};
    ///
    /// let model = ClusterModel::new(&config, &Device::flex())?;
    /// let inference = InferenceModel::new(model, &config)?;
    /// let scores = inference.differences(&[&["การ", "เรียน"][..], &["ฉัน"][..]])?;
    /// assert_eq!(scores.iter().map(Vec::len).collect::<Vec<_>>(), [2, 1]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "burn")]
    pub fn new(model: ClusterModel, config: &ClusterModelConfig) -> Result<Self, Error> {
        use burn::module::Module;

        let save_error =
            |error: Box<dyn std::error::Error + Send + Sync>| Error::CheckpointSave(error);
        let record = model
            .into_record()
            .into_bytes()
            .map_err(|error| save_error(Box::new(error)))?;
        let reader =
            burn_pack::Reader::from_bytes(burn_pack::Bytes::from_bytes_vec(record.to_vec()))
                .map_err(|error| save_error(Box::new(error)))?;
        let mut tensors = Vec::new();
        for tensor in reader
            .into_tensors()
            .map_err(|error| save_error(Box::new(error)))?
        {
            let (name, dtype, shape, _, data) = tensor
                .into_parts()
                .map_err(|error| save_error(Box::new(error)))?;
            if dtype != burn_pack::DType::F32 {
                return Err(save_error(
                    format!("tensor {name} is not 32-bit float").into(),
                ));
            }
            let values = data
                .as_chunks::<4>()
                .0
                .iter()
                .map(|chunk| f32::from_le_bytes(*chunk))
                .collect();
            tensors.push((name, shape.as_slice().to_vec(), values));
        }
        Self::from_checkpoint(config, Checkpoint::from_tensors(tensors)?)
    }

    /// Scores stacked windows of `rows` syllables in total.
    #[inline(always)]
    fn forward<const FMA: bool>(
        &self,
        windows: &[&[&str]],
        wanted: &[Vec<bool>],
        rows: usize,
    ) -> Result<Vec<Vec<f32>>, Error> {
        let mut ranges = Vec::with_capacity(windows.len());
        let mut offset = 0;
        for window in windows {
            ranges.push(offset..offset + window.len());
            offset += window.len();
        }
        let flags = wanted.iter().flatten().copied().collect::<Vec<_>>();
        let selected = flags
            .iter()
            .enumerate()
            .filter(|(_, flag)| **flag)
            .map(|(row, _)| row)
            .collect::<Vec<_>>();
        let mut scratch = Scratch::new(
            rows,
            self.model_dimension,
            self.layers
                .first()
                .map_or(0, EncoderLayer::feed_forward_dimension),
            self.max_syllables,
            self.blocks.take(),
        );
        let mut state = self.embed::<FMA>(windows, &mut scratch)?;
        let compact = match self.layers.split_last() {
            Some((last, others)) => {
                for layer in others {
                    layer.forward::<FMA>(&mut state, &ranges, &mut scratch)?;
                }
                last.forward_selected::<FMA>(&state, &ranges, &selected, &flags, &mut scratch)?
            }
            None => selected
                .iter()
                .map(|row| {
                    state
                        .get(row * self.model_dimension..(row + 1) * self.model_dimension)
                        .ok_or(Error::Batch("selected row outside the state"))
                })
                .collect::<Result<Vec<_>, _>>()?
                .concat(),
        };
        let mut logits = vec![0.0; selected.len() * 2];
        self.boundary
            .multiply_into::<FMA>(&compact, scratch.blocks_mut(), &mut logits)?;
        self.blocks.give_back(scratch.into_blocks());
        let mut row_scores = vec![0.0; rows];
        for (row, [keep, break_logit]) in selected.iter().zip(logits.as_chunks::<2>().0) {
            if let Some(score) = row_scores.get_mut(*row) {
                *score = keep - break_logit;
            }
        }
        ranges
            .into_iter()
            .map(|range| {
                row_scores
                    .get(range)
                    .map(<[f32]>::to_vec)
                    .ok_or(Error::Batch("window outside the scored rows"))
            })
            .collect()
    }

    /// Input features for stacked windows: bias, position, ordered character features and the
    /// whole-syllable embedding projection.
    #[inline(always)]
    fn embed<const FMA: bool>(
        &self,
        windows: &[&[&str]],
        scratch: &mut Scratch,
    ) -> Result<Vec<f32>, Error> {
        let model_dimension = self.model_dimension;
        let whole_dimension = self.whole_projection.inputs();
        let rows = windows.iter().map(|window| window.len()).sum::<usize>();
        let mut state = vec![0.0; rows * model_dimension];
        let mut whole = vec![0.0; rows * whole_dimension];
        let syllables = windows.iter().flat_map(|window| window.iter().enumerate());
        let targets = state
            .chunks_exact_mut(model_dimension)
            .zip(whole.chunks_exact_mut(whole_dimension));
        for ((position, syllable), (target, whole_target)) in syllables.zip(targets) {
            let features = SyllableFeatures::new(syllable);
            let bucket = usize::try_from(features.bucket(self.syllable_buckets))
                .map_err(|_| Error::Batch("syllable hash overflow"))?;
            whole_target.copy_from_slice(
                self.whole_embeddings
                    .get(bucket * whole_dimension..(bucket + 1) * whole_dimension)
                    .ok_or(Error::Batch("syllable bucket outside the model"))?,
            );
            let position_features = self
                .positions
                .get(position * model_dimension..(position + 1) * model_dimension)
                .ok_or(Error::Batch("syllable position outside the model"))?;
            for ((value, bias), position_feature) in target
                .iter_mut()
                .zip(&self.input_bias)
                .zip(position_features)
            {
                *value = bias + position_feature;
            }
            let mut length = 0;
            for (slot, id) in features.character_ids(self.character_slots).enumerate() {
                let id = usize::try_from(id).map_err(|_| Error::Batch("character ID"))?;
                for (value, feature) in target
                    .iter_mut()
                    .zip(self.slot_features.character(slot, id)?)
                {
                    *value += feature;
                }
                length = slot + 1;
            }
            for (value, feature) in target.iter_mut().zip(self.slot_features.padding(length)?) {
                *value += feature;
            }
        }
        let mut projected = vec![0.0; rows * model_dimension];
        self.whole_projection
            .multiply_into::<FMA>(&whole, scratch.blocks_mut(), &mut projected)?;
        for (value, update) in state.iter_mut().zip(&projected) {
            *value += update;
        }
        Ok(state)
    }
}
