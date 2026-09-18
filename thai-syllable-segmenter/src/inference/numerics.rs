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
