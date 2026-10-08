#![deny(unsafe_code)]
#![warn(missing_docs)]
//! Source-preserving Thai syllable separation using the bundled distilled model.
//!
//! [`SyllableSegmenter`] resolves the ambiguity that [`thai_syllable_rules`] leaves open: it
//! scores every candidate syllable with a small Transformer and returns the best-scoring complete
//! path. The syllables are slices of the input, so concatenating them gives the input back, and
//! non-Thai text passes through untouched.
//!
//! # Examples
//!
//! ```
//! use thai_syllable_segmenter::SyllableSegmenter;
//!
//! let segmenter = SyllableSegmenter::new()?;
//! let text = "พระวาทะ";
//! assert_eq!(segmenter.split(text)?, ["พระ", "วา", "ทะ"]);
//! assert_eq!(segmenter.ranges(text)?, [0..9, 9..15, 15..21]);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! A Thai run over 4,096 scalars is rejected with [`Error::ThaiRunTooLong`] instead of being
//! truncated.
//!
//! # Edge cases, invariants and panics
//!
//! - Empty input gives an empty vector; non-Thai text is returned untouched, in maximal runs.
//! - A Thai run over 4,096 scalars is [`Error::ThaiRunTooLong`] and a written cell over 48
//!   scalars is [`Error::CellTooLong`]. Input is never truncated.
//! - Segmentation never panics for any input, which property-based tests check on arbitrary
//!   Unicode and on corrupted checkpoints. The [`inference`] primitives return
//!   [`inference::ShapeError`] for operands that do not fit instead of panicking; only
//!   allocation failure aborts, as for any `Vec`.
//! - Segmentation is deterministic: the same text always gives the same syllables.
//!
//! # Unsafe code
//!
//! The crate denies `unsafe_code` except at two audited sites in [`inference`]: the call into
//! the `#[target_feature(enable = "avx2,fma")]` function in `run_on_cpu`, made only after both
//! features were detected at run time, and the `_mm_prefetch` hint in `PackedMatrix`, which has
//! no architectural effect and cannot fault. Everything else, including all indexing, is safe
//! Rust.
//!
//! # Performance and threads
//!
//! Building a [`SyllableSegmenter`] takes a few milliseconds, so build one and reuse it. It is
//! `Send + Sync`: share one between threads, for example behind an `Arc`.
//!
//! The [`inference`] module holds the CPU Transformer engine this crate runs on. It is public so
//! that `thai-word-segmenter` can run its own model on the same primitives; most users never
//! need it.

mod checkpoint;
mod contextual_model;
mod error;
/// CPU Transformer inference primitives shared with `thai-word-segmenter`.
pub mod inference;
mod lattice;
mod lattice_config;
mod model_config;
mod vocabulary;

use checkpoint::BundledModel;

pub use error::Error;

/// A reusable rules-aware syllable segmenter with its trained model loaded.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::SyllableSegmenter;
///
/// let segmenter = SyllableSegmenter::new()?;
/// assert_eq!(segmenter.split("พระวาทะ")?, ["พระ", "วา", "ทะ"]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct SyllableSegmenter {
    checkpoint: BundledModel,
}

impl std::fmt::Debug for SyllableSegmenter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SyllableSegmenter")
            .finish_non_exhaustive()
    }
}

impl SyllableSegmenter {
    /// Loads the bundled distilled student model and its rule settings.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Weights`] when the bundled weights cannot be decoded, or another
    /// [`Error`] when the bundled vocabulary or settings are inconsistent.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::SyllableSegmenter;
    ///
    /// // Building a segmenter takes a few milliseconds; build one and reuse it.
    /// let segmenter = SyllableSegmenter::new()?;
    /// # drop(segmenter);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            checkpoint: BundledModel::load()?,
        })
    }
}
