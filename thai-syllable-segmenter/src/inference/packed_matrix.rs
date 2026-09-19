//! Single-precision matrix multiplication against weights packed once at load time.
//!
//! General matrix-multiply libraries repack the weight operand on every call, which costs more
//! than the multiplication when a window has only a handful of syllables. Here the weights are
//! stored as panels of [`PANEL_WIDTH`] output columns, so each call streams them once.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

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
