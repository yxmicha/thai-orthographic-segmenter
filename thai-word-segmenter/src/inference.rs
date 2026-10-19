//! CPU forward pass equivalent to the Burn network's (`network::ClusterModel::forward`, with the
//! `burn` feature) up to float rounding, specialised for inference: no padding, no tensor-graph copies, weights packed once, and the character
//! embedding folded into the input projection.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

/// Rows processed together by one forward pass; bounds scratch memory and keeps it cache-sized.
const ROW_BUDGET: usize = 512;
