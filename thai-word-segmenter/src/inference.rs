//! CPU forward pass equivalent to the Burn network's (`network::ClusterModel::forward`, with the
//! `burn` feature) up to float rounding, specialised for inference: no padding, no tensor-graph copies, weights packed once, and the character
//! embedding folded into the input projection.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

/// Rows processed together by one forward pass; bounds scratch memory and keeps it cache-sized.
const ROW_BUDGET: usize = 512;

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
