//! Activation, softmax and reduction math written so the compiler vectorizes it.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

/// Interleaved partial sums so reductions vectorize.
pub(super) const LANES: usize = 8;

/// Values to reduce to one number.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::Values;
///
/// assert_eq!(Values::new(&[1.0, 2.0, 3.0]).sum(), 6.0);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Values<'a>(pub(crate) &'a [f32]);

/// Values to transform in place with the Gaussian error linear unit.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::Activations;
///
/// let mut values = [0.0, 8.0];
/// Activations::new(&mut values).gelu();
/// assert_eq!(values[0], 0.0);
/// assert!((values[1] - 8.0).abs() < 1e-5);
/// ```
#[derive(Debug)]
pub struct Activations<'a>(pub(crate) &'a mut [f32]);

/// One row of attention scores, before and during its softmax.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::AttentionRow;
///
/// let mut scores = [1.0, 3.0];
/// let mut row = AttentionRow::new(&mut scores);
/// let maximum = row.scale_and_maximum(1.0);
/// let total = row.exponentiate(maximum);
/// assert_eq!(maximum, 3.0);
/// assert!((total - (1.0 + (-2.0_f32).exp())).abs() < 1e-5);
/// ```
#[derive(Debug)]
pub struct AttentionRow<'a>(pub(crate) &'a mut [f32]);
