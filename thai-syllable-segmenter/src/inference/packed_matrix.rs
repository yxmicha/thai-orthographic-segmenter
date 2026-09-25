//! Single-precision matrix multiplication against weights packed once at load time.
//!
//! General matrix-multiply libraries repack the weight operand on every call, which costs more
//! than the multiplication when a window has only a handful of syllables. Here the weights are
//! stored as panels of [`PANEL_WIDTH`] output columns, so each call streams them once.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

use std::sync::{Mutex, PoisonError};

/// Output columns held by one packed panel and one micro-kernel accumulator row.
const PANEL_WIDTH: usize = 16;

/// Operands of a matrix operation whose lengths do not match the matrix shape.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{PackedMatrix, ShapeError};
///
/// // Three weights cannot form a 2x2 matrix.
/// assert_eq!(PackedMatrix::new(&[1.0, 0.0, 0.0], 2, 2, vec![0.0; 2]).unwrap_err(), ShapeError);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[error("operand lengths do not match the matrix shape")]
pub struct ShapeError;

/// Reusable storage for input rows copied into micro-kernel order by
/// [`PackedMatrix::multiply_into`]. Keeping one per forward pass avoids an allocation per product.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{InputBlocks, PackedMatrix};
///
/// let matrix = PackedMatrix::new(&[1.0, 0.0, 0.0, 1.0], 2, 2, vec![0.0; 2])?;
/// // One buffer serves every product of a pass.
/// let mut blocks = InputBlocks::default();
/// let mut output = [0.0; 2];
/// matrix.multiply_into::<false>(&[3.0, 4.0], &mut blocks, &mut output)?;
/// matrix.multiply_into::<false>(&[5.0, 6.0], &mut blocks, &mut output)?;
/// assert_eq!(output, [5.0, 6.0]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Default)]
pub struct InputBlocks(Vec<f32>);

/// Input-row storage shared by the forward passes of one model, so each pass starts from buffers
/// that earlier passes already grew instead of allocating and faulting in fresh memory.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::BlockPool;
///
/// let pool = BlockPool::default();
/// let blocks = pool.take();
/// pool.give_back(blocks);
/// ```
#[derive(Default)]
pub struct BlockPool(Mutex<Vec<InputBlocks>>);

impl BlockPool {
    /// Hands out storage left by an earlier pass, or empty storage when none is idle.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::BlockPool;
    ///
    /// let pool = BlockPool::default();
    /// let blocks = pool.take(); // empty: no earlier pass left storage
    /// pool.give_back(blocks);
    /// let reused = pool.take(); // the storage given back
    /// # drop(reused);
    /// ```
    pub fn take(&self) -> InputBlocks {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop()
            .unwrap_or_default()
    }
}

/// A `[inputs, outputs]` weight matrix with a bias vector, packed for [`Self::multiply`] and [`Self::multiply_into`].
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{InputBlocks, PackedMatrix};
///
/// // y = x · W + b for a 2x2 identity W and a bias of one.
/// let matrix = PackedMatrix::new(&[1.0, 0.0, 0.0, 1.0], 2, 2, vec![1.0; 2])?;
/// let mut output = [0.0; 2];
/// matrix.multiply_into::<false>(&[3.0, 4.0], &mut InputBlocks::default(), &mut output)?;
/// assert_eq!(output, [4.0, 5.0]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct PackedMatrix {
    inputs: usize,
    outputs: usize,
    /// `[panel][input][PANEL_WIDTH]`; columns past `outputs` are zero.
    panels: Vec<f32>,
    /// One value per output, padded with zeros to a whole number of panels so every panel reads
    /// [`PANEL_WIDTH`] values.
    bias: Vec<f32>,
}

/// Multiply-add that fuses rounding only where the CPU has the instruction; without it a fused
/// operation would fall back to a slow software routine.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::multiply_add;
///
/// assert_eq!(multiply_add::<false>(2.0, 3.0, 1.0), 7.0);
/// ```
#[inline(always)]
#[must_use]
pub fn multiply_add<const FMA: bool>(left: f32, right: f32, sum: f32) -> f32 {
    if FMA {
        left.mul_add(right, sum)
    } else {
        left * right + sum
    }
}

impl std::fmt::Debug for PackedMatrix {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PackedMatrix")
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .finish_non_exhaustive()
    }
}

impl PackedMatrix {
    /// Packs a row-major `[inputs, outputs]` weight matrix. Returns [`ShapeError`] unless
    /// `weight` holds `inputs * outputs` values, `bias` one value per output, and both
    /// dimensions are positive.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError`] when a dimension is zero, or `weight` or `bias` has the wrong
    /// length.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::PackedMatrix;
    ///
    /// let matrix = PackedMatrix::new(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 2, 3, vec![0.0; 3])?;
    /// assert_eq!((matrix.inputs(), matrix.outputs()), (2, 3));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(
        weight: &[f32],
        inputs: usize,
        outputs: usize,
        bias: Vec<f32>,
    ) -> Result<Self, ShapeError> {
        if inputs == 0 || outputs == 0 || weight.len() != inputs * outputs || bias.len() != outputs
        {
            return Err(ShapeError);
        }
        let panel_count = outputs.div_ceil(PANEL_WIDTH);
        let mut panels = vec![0.0; panel_count * inputs * PANEL_WIDTH];
        for (input, row) in weight.chunks_exact(outputs).enumerate() {
            for (panel, columns) in row.chunks(PANEL_WIDTH).enumerate() {
                let start = (panel * inputs + input) * PANEL_WIDTH;
                if let Some(target) = panels.get_mut(start..start + columns.len()) {
                    target.copy_from_slice(columns);
                }
            }
        }
        let mut bias = bias;
        bias.resize(panel_count * PANEL_WIDTH, 0.0);
        Ok(Self {
            inputs,
            outputs,
            panels,
            bias,
        })
    }
}
