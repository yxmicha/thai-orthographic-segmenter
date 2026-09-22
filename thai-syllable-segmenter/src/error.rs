use crate::inference::{CheckpointError, ShapeError};

/// Failures from loading the bundled model or segmenting text.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::{Error, SyllableSegmenter};
///
/// let segmenter = SyllableSegmenter::new()?;
/// // Input is never truncated: an over-long Thai run is an error.
/// match segmenter.split(&"ก".repeat(5000)) {
///     Err(Error::ThaiRunTooLong { scalars, limit }) => assert!(scalars > limit),
///     other => panic!("unexpected result: {other:?}"),
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A Thai run is longer than the model's position capacity; input is never truncated.
    #[error("Thai run of {scalars} scalars exceeds the {limit}-scalar limit")]
    ThaiRunTooLong {
        /// Unicode scalars in the rejected Thai run.
        scalars: usize,
        /// Largest Thai run the model accepts.
        limit: usize,
    },
    /// A written cell (a base with its marks) is longer than the longest candidate span.
    #[error("a written Thai cell exceeds the {limit}-scalar candidate span limit")]
    CellTooLong {
        /// Longest candidate span in Unicode scalars.
        limit: usize,
    },
    /// The bundled weights could not be read or do not match the network.
    #[error("load bundled syllable model weights")]
    Weights(#[from] CheckpointError),
    /// The inference engine received operands that do not fit the model's shape.
    #[error(transparent)]
    Shape(#[from] ShapeError),
    /// A bundled-model, vocabulary, or lattice invariant does not hold.
    #[error("syllable model invariant violated: {0}")]
    Invariant(&'static str),
}
