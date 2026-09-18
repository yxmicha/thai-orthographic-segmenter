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


/// Value of the `f16_layout` metadata key that marks 16-bit matrices stored as byte planes.
pub const F16_PLANES: &str = "planes";
