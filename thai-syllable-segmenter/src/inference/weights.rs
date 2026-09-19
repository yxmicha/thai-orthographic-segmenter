//! Plain-data weight containers for the CPU inference engine.
//!
//! Trained weights arrive as row-major `f32` vectors. These types validate their shapes once, so
//! the engine can trust them, and own no framework types: any trainer can fill them.


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
