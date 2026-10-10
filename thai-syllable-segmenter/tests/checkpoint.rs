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

fn single_tensor(values: &[f32]) -> Stored<'static> {
    let bytes = values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    ("t.weight", Dtype::F32, vec![2, 2], bytes)
}

#[test]
fn single_precision_tensors_are_read_unchanged() -> Result<(), CheckpointError> {
    let tensor = single_tensor(&[1.5, -2.0, 0.25, 8.0]);
    let mut checkpoint = Checkpoint::from_bytes(&pack(&[tensor], false))?;
    assert_eq!(
        checkpoint.embedding("t", 2, 2)?.values(),
        [1.5, -2.0, 0.25, 8.0]
    );
    checkpoint.finish()
}

#[test]
fn plain_and_byte_plane_half_tensors_widen_exactly() -> Result<(), CheckpointError> {
    let expected = HALVES.iter().map(|(_, value)| *value).collect::<Vec<_>>();
    for planes in [false, true] {
        let mut checkpoint = Checkpoint::from_bytes(&pack(&[half_tensor(planes)], planes))?;
        assert_eq!(
            checkpoint.embedding("t", 3, 2)?.values(),
            expected,
            "planes: {planes}"
        );
    }
    Ok(())
}

#[test]
fn byte_planes_need_the_layout_metadata() {
    assert!(matches!(
        Checkpoint::from_bytes(&pack(&[half_tensor(true)], false)),
        Err(CheckpointError::UnsupportedType { name }) if name == "t.weight"
    ));
}

#[test]
fn not_a_number_halves_stay_not_a_number() -> Result<(), CheckpointError> {
    let bytes = 0x7e00_u16.to_le_bytes().to_vec();
    let tensor = ("t.weight", Dtype::F16, vec![1, 1], bytes);
    let mut checkpoint = Checkpoint::from_bytes(&pack(&[tensor], false))?;
    assert!(checkpoint.embedding("t", 1, 1)?.values()[0].is_nan());
    Ok(())
}

#[test]
fn other_element_types_are_rejected() {
    let tensor = ("t.weight", Dtype::I64, vec![1], vec![0; 8]);
    assert!(matches!(
        Checkpoint::from_bytes(&pack(&[tensor], false)),
        Err(CheckpointError::UnsupportedType { name }) if name == "t.weight"
    ));
}

#[test]
fn missing_mismatched_and_unused_tensors_are_reported() -> Result<(), CheckpointError> {
    let tensor = single_tensor(&[0.0; 4]);
    let mut checkpoint = Checkpoint::from_bytes(&pack(&[tensor], false))?;
    assert!(matches!(
        checkpoint.embedding("absent", 2, 2),
        Err(CheckpointError::Missing(_))
    ));
    assert!(matches!(
        checkpoint.embedding("t", 4, 1),
        Err(CheckpointError::Shape { .. })
    ));
    assert!(
        matches!(checkpoint.finish(), Err(CheckpointError::Unused(names)) if names == ["t.weight"])
    );
    Ok(())
}

/// An 8-bit matrix of two rows and three columns with the given scale tensor.
fn int8_file(scale_shape: &[usize], scales: &[f32]) -> Vec<u8> {
    let integers = [10_i8, -20, 30, 127, -127, 0]
        .iter()
        .map(|value| value.to_le_bytes()[0])
        .collect::<Vec<_>>();
    let scale_bytes = scales
        .iter()
        .flat_map(|scale| scale.to_le_bytes())
        .collect::<Vec<_>>();
    pack(
        &[
            ("t.weight", Dtype::I8, vec![2, 3], integers),
            (
                "t.weight.scale",
                Dtype::F32,
                scale_shape.to_vec(),
                scale_bytes,
            ),
        ],
        false,
    )
}

#[test]
fn int8_matrices_are_scaled_per_column_or_per_row() -> Result<(), CheckpointError> {
    let mut per_column = Checkpoint::from_bytes(&int8_file(&[1, 3], &[0.5, 1.0, 2.0]))?;
    assert_eq!(
        per_column.embedding("t", 2, 3)?.values(),
        [5.0, -20.0, 60.0, 63.5, -127.0, 0.0]
    );
    // The scale tensor belongs to the matrix: nothing is left over.
    per_column.finish()?;
    let mut per_row = Checkpoint::from_bytes(&int8_file(&[2, 1], &[0.5, 2.0]))?;
    assert_eq!(
        per_row.embedding("t", 2, 3)?.values(),
        [5.0, -10.0, 15.0, 254.0, -254.0, 0.0]
    );
    per_row.finish()
}

#[test]
fn int8_matrices_need_a_scale_that_fits_their_shape() {
    for (shape, scales) in [
        (vec![1, 2], vec![1.0, 1.0]),
        (vec![3], vec![1.0; 3]),
        (vec![3, 1], vec![1.0; 3]),
    ] {
        assert!(
            matches!(
                Checkpoint::from_bytes(&int8_file(&shape, &scales)),
                Err(CheckpointError::Scale { name }) if name == "t.weight"
            ),
            "scale shape {shape:?}"
        );
    }
    let alone = pack(&[("t.weight", Dtype::I8, vec![2, 3], vec![0; 6])], false);
    assert!(matches!(
        Checkpoint::from_bytes(&alone),
        Err(CheckpointError::Scale { .. })
    ));
}
