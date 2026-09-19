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
