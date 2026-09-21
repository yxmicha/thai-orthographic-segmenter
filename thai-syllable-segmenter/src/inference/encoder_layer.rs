//! Pre-norm Transformer encoder layer for inference over stacked variable-length windows.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

use super::{
    numerics::{Activations, AttentionRow, LANES, Values},
    packed_matrix::{InputBlocks, PackedMatrix, ShapeError, multiply_add},
    weights::{AttentionWeights, FeedForwardWeights, LayerNormWeights, WeightMatrix},
};

#[derive(Clone)]
struct Norm {
    gamma: Vec<f32>,
    beta: Vec<f32>,
}

/// One encoder layer with fused query/key/value projections.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{
///     AttentionWeights, EncoderLayer, FeedForwardWeights, LayerNormWeights, WeightMatrix,
/// };
///
/// let matrix = |inputs: usize, outputs: usize| {
///     WeightMatrix::new(vec![0.0; inputs * outputs], inputs, outputs, vec![0.0; outputs])
/// };
/// let norm = || LayerNormWeights::new(vec![1.0; 8], vec![0.0; 8]);
/// let attention =
///     AttentionWeights::new(norm()?, matrix(8, 8)?, matrix(8, 8)?, matrix(8, 8)?, matrix(8, 8)?, 2)?;
/// let feed_forward = FeedForwardWeights::new(norm()?, matrix(8, 16)?, matrix(16, 8)?)?;
/// let layer = EncoderLayer::new(attention, feed_forward)?;
/// assert_eq!(layer.feed_forward_dimension(), 16);
/// # Ok::<(), thai_syllable_segmenter::inference::ShapeError>(())
/// ```
#[derive(Clone)]
pub struct EncoderLayer {
    attention_norm: Norm,
    qkv: PackedMatrix,
    output: PackedMatrix,
    feed_forward_norm: Norm,
    inner: PackedMatrix,
    outer: PackedMatrix,
    heads: usize,
    model_dimension: usize,
}
