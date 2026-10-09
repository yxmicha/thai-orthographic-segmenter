//! Reading safetensors checkpoints with 32-bit, plain 16-bit and byte-plane 16-bit tensors.

use std::collections::HashMap;

use safetensors::{Dtype, tensor::TensorView};
use thai_syllable_segmenter::inference::{Checkpoint, CheckpointError, F16_LAYOUT_KEY, F16_PLANES};

/// One tensor to store: its name, element type, shape and bytes.
type Stored<'a> = (&'a str, Dtype, Vec<usize>, Vec<u8>);

fn pack(tensors: &[Stored<'_>], planes: bool) -> Vec<u8> {
    let views = tensors.iter().map(|(name, dtype, shape, data)| {
        (
            (*name).to_owned(),
            TensorView::new(*dtype, shape.clone(), data).expect("a valid tensor"),
        )
    });
    let metadata =
        planes.then(|| HashMap::from([(F16_LAYOUT_KEY.to_owned(), F16_PLANES.to_owned())]));
    safetensors::serialize(views, metadata).expect("a valid file")
}
