//! Reading trained weights from a safetensors checkpoint into the engine's plain-data types.
//!
//! A [safetensors](https://github.com/huggingface/safetensors) file stores named tensors.
//! [`Checkpoint`] reads one without a deep-learning framework and hands out validated weights
//! by name, so a model loader asks for exactly the tensors it expects and
//! [`Checkpoint::finish`] reports any it left unread.
//!
//! Tensors may be stored as 32-bit floats (`F32`) or as 16-bit floats (`F16`) to halve the
//! file. A 16-bit matrix may also be stored as byte planes, which compress better because the
//! high bytes hold the sign and exponent: the file's `f16_layout` metadata is [`F16_PLANES`],
//! and the tensor is a `U8` tensor of shape `[2, ...shape]` whose first row holds the high
//! bytes of every value and whose second row holds the low bytes.
//!
//! A matrix may instead be quantized to 8 bits: an `I8` tensor of shape `[rows, columns]`
//! next to an `F32` tensor `{name}.scale` of shape `[1, columns]` (one scale per column) or
//! `[rows, 1]` (one scale per row). Each value is the integer times its scale. The
//! `convert-checkpoint` example of this crate writes all of these forms.

use std::collections::BTreeMap;

use super::{
    encoder_layer::EncoderLayer,
    packed_matrix::ShapeError,
    weights::{
        AttentionWeights, EmbeddingTable, FeedForwardWeights, LayerNormWeights, WeightMatrix,
    },
};

/// Value of the `f16_layout` metadata key that marks 16-bit matrices stored as byte planes.
pub const F16_PLANES: &str = "planes";

/// Metadata key naming the layout of 16-bit tensors.
pub const F16_LAYOUT_KEY: &str = "f16_layout";

/// Failures reading a checkpoint or asking it for weights.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{Checkpoint, CheckpointError};
///
/// // Bytes that are not a safetensors file are rejected.
/// assert!(matches!(Checkpoint::from_bytes(b"not a checkpoint"), Err(CheckpointError::Read(_))));
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CheckpointError {
    /// The bytes are not a readable safetensors file.
    #[error("read safetensors checkpoint")]
    Read(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// A tensor is stored as neither 32-bit nor 16-bit floats.
    #[error("checkpoint tensor {name:?} is not 32-bit or 16-bit floating point")]
    UnsupportedType {
        /// Name of the tensor.
        name: String,
    },
    /// A requested tensor is not in the checkpoint.
    #[error("checkpoint has no tensor {0:?}")]
    Missing(String),
    /// A tensor has a different shape from the one requested.
    #[error("checkpoint tensor {name:?} has shape {found:?}, expected {expected:?}")]
    Shape {
        /// Name of the tensor.
        name: String,
        /// Shape stored in the checkpoint.
        found: Vec<usize>,
        /// Shape the caller asked for.
        expected: Vec<usize>,
    },
    /// An 8-bit tensor lacks a usable `{name}.scale` tensor.
    #[error(
        "int8 checkpoint tensor {name:?} needs an f32 `{name}.scale` of shape [1, columns] or [rows, 1]"
    )]
    Scale {
        /// Name of the quantized tensor.
        name: String,
    },
    /// A tensor given to [`Checkpoint::from_tensors`] holds a different number of values than
    /// its shape implies.
    #[error("checkpoint tensor {name:?} has {found} values for shape {shape:?}")]
    Length {
        /// Name of the tensor.
        name: String,
        /// Number of values supplied.
        found: usize,
        /// Shape supplied.
        shape: Vec<usize>,
    },
    /// Tensors remain that no loader asked for.
    #[error("checkpoint has unused tensors {0:?}")]
    Unused(Vec<String>),
    /// The requested weights do not form a valid layer.
    #[error(transparent)]
    Layer(#[from] ShapeError),
}

/// Named tensors of a trained network, read from a safetensors file.
///
/// Each loader method removes the tensors it reads, so data moves out without a copy and
/// [`Checkpoint::finish`] can report what was never used.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::Checkpoint;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/model/weights.safetensors"))?;
/// let mut checkpoint = Checkpoint::from_bytes(&bytes)?;
/// // The bundled syllable model has a 72-row, 16-wide character embedding.
/// let characters = checkpoint.embedding("characters", 72, 16)?;
/// assert_eq!(characters.values().len(), 72 * 16);
/// # Ok(())
/// # }
/// ```
pub struct Checkpoint {
    tensors: BTreeMap<String, (Vec<usize>, Vec<f32>)>,
}
