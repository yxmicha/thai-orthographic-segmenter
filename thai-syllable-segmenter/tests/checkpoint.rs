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

/// binary16 bit patterns and the single-precision values they denote: zero, one, minus 2.5, the
/// smallest subnormal, the largest finite value, and positive infinity.
const HALVES: [(u16, f32); 6] = [
    (0x0000, 0.0),
    (0x3c00, 1.0),
    (0xc100, -2.5),
    (0x0001, 5.960_464_5e-8),
    (0x7bff, 65_504.0),
    (0x7c00, f32::INFINITY),
];

fn half_tensor(planes: bool) -> Stored<'static> {
    let values = HALVES.iter().map(|(bits, _)| *bits).collect::<Vec<_>>();
    if planes {
        let mut bytes = values
            .iter()
            .map(|bits| (bits >> 8) as u8)
            .collect::<Vec<_>>();
        bytes.extend(values.iter().map(|bits| (bits & 0xff) as u8));
        ("t.weight", Dtype::U8, vec![2, 3, 2], bytes)
    } else {
        let bytes = values.iter().flat_map(|bits| bits.to_le_bytes()).collect();
        ("t.weight", Dtype::F16, vec![3, 2], bytes)
    }
}
