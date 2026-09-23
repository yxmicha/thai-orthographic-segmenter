//! Plain-data weight containers for the CPU inference engine.
//!
//! Trained weights arrive as row-major `f32` vectors. These types validate their shapes once, so
//! the engine can trust them, and own no framework types: any trainer can fill them.

use super::packed_matrix::{PackedMatrix, ShapeError};

/// A row-major `[inputs, outputs]` weight matrix and its bias, before packing.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::WeightMatrix;
///
/// // Three inputs, two outputs.
/// let matrix = WeightMatrix::new(vec![0.0; 6], 3, 2, vec![0.0; 2])?;
/// assert_eq!((matrix.inputs(), matrix.outputs()), (3, 2));
/// assert_eq!(matrix.values().len(), 6);
/// # Ok::<(), thai_syllable_segmenter::inference::ShapeError>(())
/// ```
#[derive(Clone)]
pub struct WeightMatrix {
    values: Vec<f32>,
    inputs: usize,
    outputs: usize,
    bias: Vec<f32>,
}

impl std::fmt::Debug for WeightMatrix {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WeightMatrix")
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .finish_non_exhaustive()
    }
}

impl WeightMatrix {
    /// Wraps row-major `values` of shape `[inputs, outputs]` and one `bias` per output.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError`] when a dimension is zero, `values` is not `inputs * outputs` long,
    /// or `bias` is not `outputs` long.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::{ShapeError, WeightMatrix};
    ///
    /// assert!(WeightMatrix::new(vec![0.0; 6], 3, 2, vec![0.0; 2]).is_ok());
    /// // Five values cannot form a 3x2 matrix.
    /// assert_eq!(WeightMatrix::new(vec![0.0; 5], 3, 2, vec![0.0; 2]).unwrap_err(), ShapeError);
    /// ```
    pub fn new(
        values: Vec<f32>,
        inputs: usize,
        outputs: usize,
        bias: Vec<f32>,
    ) -> Result<Self, ShapeError> {
        if inputs == 0
            || outputs == 0
            || inputs.checked_mul(outputs) != Some(values.len())
            || bias.len() != outputs
        {
            return Err(ShapeError);
        }
        Ok(Self {
            values,
            inputs,
            outputs,
            bias,
        })
    }
}

/// A row-major embedding table, `[rows, width]`.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::EmbeddingTable;
///
/// // Four rows of three values.
/// let table = EmbeddingTable::from(vec![0.0; 12]);
/// assert_eq!(table.values().len(), 12);
/// ```
#[derive(Clone)]
pub struct EmbeddingTable(Vec<f32>);

/// Layer-normalization scale and shift, one value of each per feature.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::LayerNormWeights;
///
/// let norm = LayerNormWeights::new(vec![1.0; 8], vec![0.0; 8])?;
/// assert_eq!(norm.width(), 8);
/// # Ok::<(), thai_syllable_segmenter::inference::ShapeError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct LayerNormWeights {
    gamma: Vec<f32>,
    beta: Vec<f32>,
}

/// The weights of the self-attention half of a pre-norm encoder layer.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{AttentionWeights, LayerNormWeights, WeightMatrix};
///
/// let square = || WeightMatrix::new(vec![0.0; 64], 8, 8, vec![0.0; 8]);
/// let norm = LayerNormWeights::new(vec![1.0; 8], vec![0.0; 8])?;
/// // Width 8 with two heads of width 4.
/// let attention = AttentionWeights::new(norm, square()?, square()?, square()?, square()?, 2)?;
/// # drop(attention);
/// # Ok::<(), thai_syllable_segmenter::inference::ShapeError>(())
/// ```
#[derive(Clone, Debug)]
pub struct AttentionWeights {
    norm: LayerNormWeights,
    query: WeightMatrix,
    key: WeightMatrix,
    value: WeightMatrix,
    output: WeightMatrix,
    heads: usize,
}

/// The weights of the feed-forward half of a pre-norm encoder layer.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{FeedForwardWeights, LayerNormWeights, WeightMatrix};
///
/// let norm = LayerNormWeights::new(vec![1.0; 8], vec![0.0; 8])?;
/// let inner = WeightMatrix::new(vec![0.0; 128], 8, 16, vec![0.0; 16])?;
/// let outer = WeightMatrix::new(vec![0.0; 128], 16, 8, vec![0.0; 8])?;
/// let feed_forward = FeedForwardWeights::new(norm, inner, outer)?;
/// # drop(feed_forward);
/// # Ok::<(), thai_syllable_segmenter::inference::ShapeError>(())
/// ```
#[derive(Clone, Debug)]
pub struct FeedForwardWeights {
    norm: LayerNormWeights,
    inner: WeightMatrix,
    outer: WeightMatrix,
}
