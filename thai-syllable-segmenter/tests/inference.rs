/// Deterministic values in `[-1, 1)`.
fn values(count: usize, seed: u64) -> Vec<f32> {
    let mut state = seed;
    (0..count)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            #[allow(clippy::cast_precision_loss, reason = "both values fit in 24 bits")]
            let unit = ((state >> 40) as f32) / (1u64 << 23) as f32;
            unit - 1.0
        })
        .collect()
}
