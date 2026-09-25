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

    /// Joins matrices that share one input into a single wider matrix.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError`] when `parts` is empty or the parts have different input counts.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::WeightMatrix;
    ///
    /// let query = WeightMatrix::new(vec![0.0; 6], 3, 2, vec![0.0; 2])?;
    /// let key = WeightMatrix::new(vec![0.0; 6], 3, 2, vec![0.0; 2])?;
    /// // Two products that share one input become one product with twice the outputs.
    /// let fused = WeightMatrix::concatenate_outputs(&[query, key])?;
    /// assert_eq!((fused.inputs(), fused.outputs()), (3, 4));
    /// # Ok::<(), thai_syllable_segmenter::inference::ShapeError>(())
    /// ```
    pub fn concatenate_outputs(parts: &[Self]) -> Result<Self, ShapeError> {
        let first = parts.first().ok_or(ShapeError)?;
        let inputs = first.inputs;
        if parts.iter().any(|part| part.inputs != inputs) {
            return Err(ShapeError);
        }
        let outputs = parts.iter().map(|part| part.outputs).sum();
        let mut values = Vec::with_capacity(inputs * outputs);
        let mut rows = parts
            .iter()
            .map(|part| part.values.chunks_exact(part.outputs))
            .collect::<Vec<_>>();
        for _ in 0..inputs {
            for row in &mut rows {
                values.extend(row.next().into_iter().flatten());
            }
        }
        let bias = parts
            .iter()
            .flat_map(|part| part.bias.iter().copied())
            .collect();
        Self::new(values, inputs, outputs, bias)
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

impl std::fmt::Debug for EmbeddingTable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EmbeddingTable")
            .field("values", &self.0.len())
            .finish_non_exhaustive()
    }
}

impl From<Vec<f32>> for EmbeddingTable {
    fn from(values: Vec<f32>) -> Self {
        Self(values)
    }
}

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

impl LayerNormWeights {

    /// Number of features normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::LayerNormWeights;
    ///
    /// assert_eq!(LayerNormWeights::new(vec![1.0; 4], vec![0.0; 4])?.width(), 4);
    /// # Ok::<(), thai_syllable_segmenter::inference::ShapeError>(())
    /// ```
    #[must_use]
    pub fn width(&self) -> usize {
        self.gamma.len()
    }

    pub(crate) fn gamma(&self) -> &[f32] {
        &self.gamma
    }

    pub(crate) fn beta(&self) -> &[f32] {
        &self.beta
    }
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

impl FeedForwardWeights {

    pub(crate) fn into_parts(self) -> (LayerNormWeights, WeightMatrix, WeightMatrix) {
        (self.norm, self.inner, self.outer)
    }
}
