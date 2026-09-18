//! Activation, softmax and reduction math written so the compiler vectorizes it.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

/// Interleaved partial sums so reductions vectorize.
pub(super) const LANES: usize = 8;
