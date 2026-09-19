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
