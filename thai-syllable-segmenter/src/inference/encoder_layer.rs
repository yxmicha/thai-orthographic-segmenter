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

/// Activations of one forward pass over stacked windows, reused by every layer.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{InputBlocks, Scratch};
///
/// // Buffers for 3 positions of width 8, with a feed-forward width of 16 and windows of 3.
/// let scratch = Scratch::new(3, 8, 16, 3, InputBlocks::default());
/// assert!(format!("{scratch:?}").contains("rows: 3"));
/// ```
pub struct Scratch {
    /// Reusable copies of matrix inputs, shared by every product of the pass.
    pub(crate) blocks: InputBlocks,
    rows: usize,
    window_capacity: usize,
    normalized: Vec<f32>,
    qkv: Vec<f32>,
    context: Vec<f32>,
    projected: Vec<f32>,
    hidden: Vec<f32>,
    scores: Vec<f32>,
    keys: Vec<f32>,
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

impl Norm {
    fn new(weights: &LayerNormWeights) -> Self {
        Self {
            gamma: weights.gamma().to_vec(),
            beta: weights.beta().to_vec(),
        }
    }
}
