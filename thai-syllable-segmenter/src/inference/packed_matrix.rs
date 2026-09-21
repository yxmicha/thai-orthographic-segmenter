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
