//! Runtime selection of the fastest instruction set for one computation.


#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
fn run_with_fma<R>(computation: impl FnOnce() -> R) -> R {
    computation()
}
