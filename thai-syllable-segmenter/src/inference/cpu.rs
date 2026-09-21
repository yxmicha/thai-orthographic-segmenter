//! Runtime selection of the fastest instruction set for one computation.

/// Runs `with_fma` when the CPU has AVX2 and fused multiply-add, and `baseline` otherwise.
///
/// Both closures should run the same computation, instantiated with and without fused
/// multiply-add. Each closure and every function it calls must be `#[inline(always)]`, so
/// the code is compiled with the instruction set `run_with_fma` enables; a function that is
/// not inlined silently runs without AVX2.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{multiply_add, run_on_cpu};
///
/// // The same computation, with and without fused multiply-add.
/// let result = run_on_cpu(
///     || multiply_add::<true>(2.0, 3.0, 1.0),
///     || multiply_add::<false>(2.0, 3.0, 1.0),
/// );
/// assert_eq!(result, 7.0);
/// ```
///
/// # Safety
///
/// The function is safe to call. It enters the AVX2/FMA code path only after
/// `is_x86_feature_detected!` confirmed both features, which is the only requirement of the
/// `#[target_feature]` function it calls.
#[allow(
    unsafe_code,
    reason = "calls a #[target_feature] function after run-time detection"
)]
pub fn run_on_cpu<R>(with_fma: impl FnOnce() -> R, baseline: impl FnOnce() -> R) -> R {
    #[cfg(target_arch = "x86_64")]
    if std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma") {
        // SAFETY: the CPU features enabled on `run_with_fma` were just detected.
        return unsafe { run_with_fma(with_fma) };
    }
    baseline()
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
fn run_with_fma<R>(computation: impl FnOnce() -> R) -> R {
    computation()
}
