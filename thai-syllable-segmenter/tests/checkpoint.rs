//! Reading safetensors checkpoints with 32-bit, plain 16-bit and byte-plane 16-bit tensors.


use safetensors::{Dtype, tensor::TensorView};

/// One tensor to store: its name, element type, shape and bytes.
type Stored<'a> = (&'a str, Dtype, Vec<usize>, Vec<u8>);
