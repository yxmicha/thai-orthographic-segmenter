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

/// Layer-normalization epsilon of the Burn transformer the weights were trained with.
const LAYER_NORM_EPSILON: f32 = 1e-5;

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

impl Scratch {

    /// Buffers for `rows` stacked positions in windows of at most `window` positions.
    ///
    /// # Panics
    ///
    /// Aborts on allocation failure, and panics when `rows` times a dimension overflows `usize`,
    /// which only absurd sizes reach.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::{InputBlocks, Scratch};
    ///
    /// let scratch = Scratch::new(3, 8, 16, 3, InputBlocks::default());
    /// # drop(scratch);
    /// ```
    #[must_use]
    pub fn new(
        rows: usize,
        model_dimension: usize,
        feed_forward_dimension: usize,
        window: usize,
        blocks: InputBlocks,
    ) -> Self {
        Self {
            blocks,
            rows,
            window_capacity: window,
            normalized: vec![0.0; rows * model_dimension],
            qkv: vec![0.0; rows * 3 * model_dimension],
            context: vec![0.0; rows * model_dimension],
            projected: vec![0.0; rows * model_dimension],
            hidden: vec![0.0; rows * feed_forward_dimension],
            scores: vec![0.0; window.next_multiple_of(LANES)],
            keys: vec![0.0; model_dimension * window.next_multiple_of(LANES)],
        }
    }
}

impl EncoderLayer {
    /// Builds a layer from its attention and feed-forward weights, fusing the query, key and
    /// value projections into one product.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError`] when the two halves disagree on the model width.
    ///
    /// # Examples
    ///
    /// ```
    /// # use thai_syllable_segmenter::inference::{AttentionWeights, FeedForwardWeights, LayerNormWeights, WeightMatrix};
    /// use thai_syllable_segmenter::inference::EncoderLayer;
    ///
    /// # let matrix = |inputs: usize, outputs: usize| WeightMatrix::new(vec![0.0; inputs * outputs], inputs, outputs, vec![0.0; outputs]);
    /// # let norm = || LayerNormWeights::new(vec![1.0; 8], vec![0.0; 8]);
    /// # let attention = AttentionWeights::new(norm()?, matrix(8, 8)?, matrix(8, 8)?, matrix(8, 8)?, matrix(8, 8)?, 2)?;
    /// # let feed_forward = FeedForwardWeights::new(norm()?, matrix(8, 16)?, matrix(16, 8)?)?;
    /// let layer = EncoderLayer::new(attention, feed_forward)?;
    /// assert_eq!(layer.feed_forward_dimension(), 16);
    /// # Ok::<(), thai_syllable_segmenter::inference::ShapeError>(())
    /// ```
    pub fn new(
        attention: AttentionWeights,
        feed_forward: FeedForwardWeights,
    ) -> Result<Self, ShapeError> {
        let (attention_norm, projections, output, heads) = attention.into_parts();
        let (feed_forward_norm, inner, outer) = feed_forward.into_parts();
        let model_dimension = attention_norm.width();
        if feed_forward_norm.width() != model_dimension {
            return Err(ShapeError);
        }
        Ok(Self {
            attention_norm: Norm::new(&attention_norm),
            qkv: WeightMatrix::concatenate_outputs(&projections)?.try_into()?,
            output: output.try_into()?,
            feed_forward_norm: Norm::new(&feed_forward_norm),
            inner: inner.try_into()?,
            outer: outer.try_into()?,
            heads,
            model_dimension,
        })
    }

    /// Width of the feed-forward hidden layer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use thai_syllable_segmenter::inference::{AttentionWeights, EncoderLayer, FeedForwardWeights, InputBlocks, LayerNormWeights, Scratch, WeightMatrix};
    /// # let matrix = |inputs: usize, outputs: usize| WeightMatrix::new(vec![0.0; inputs * outputs], inputs, outputs, vec![0.0; outputs]);
    /// # let norm = || LayerNormWeights::new(vec![1.0; 8], vec![0.0; 8]);
    /// # let attention = AttentionWeights::new(norm()?, matrix(8, 8)?, matrix(8, 8)?, matrix(8, 8)?, matrix(8, 8)?, 2)?;
    /// # let feed_forward = FeedForwardWeights::new(norm()?, matrix(8, 16)?, matrix(16, 8)?)?;
    /// # let layer = EncoderLayer::new(attention, feed_forward)?;
    /// assert_eq!(layer.feed_forward_dimension(), 16);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn feed_forward_dimension(&self) -> usize {
        self.inner.outputs()
    }
}
