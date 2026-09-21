//! Pre-norm Transformer encoder layer for inference over stacked variable-length windows.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

#[derive(Clone)]
struct Norm {
    gamma: Vec<f32>,
    beta: Vec<f32>,
}
