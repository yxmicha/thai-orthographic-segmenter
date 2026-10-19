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
