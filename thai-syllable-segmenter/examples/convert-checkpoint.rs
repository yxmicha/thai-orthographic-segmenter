//! Converts a Burn burnpack checkpoint with 32-bit float tensors, as a training run saves it,
//! into the safetensors file the bundled models use.
//!
//! ```sh
//! cargo run --release --example convert-checkpoint -- model-f32.bpk model.safetensors
//! cargo run --release --example convert-checkpoint -- --int8 model-f32.bpk model.safetensors
//! ```
//!
//! Matrices of rank 2 with more than 4,096 values are shrunk; biases, norms and small tables
//! stay 32-bit. By default they are rounded to the nearest 16-bit float and stored as byte
//! planes (a `U8` tensor of shape `[2, rows, columns]`, high bytes first, with the `f16_layout`
//! metadata). With `--int8` they are quantized to 8 bits instead: an `I8` tensor plus a
//! `{name}.scale` tensor holding one scale per column or per row, whichever reconstructs the
//! matrix more exactly. Check segmentation on representative text before shipping a rounded
//! model.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "bit manipulation of IEEE 754 fields whose ranges the code checks"
)]

use std::{collections::HashMap, env, error::Error, path::Path};

use burn_pack::{Bytes, DType, Reader};
use safetensors::{Dtype, tensor::TensorView};
use thai_syllable_segmenter::inference::{F16_LAYOUT_KEY, F16_PLANES};

/// Rounds to the nearest binary16 value, ties to even, and returns its bits.
fn single_to_half(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32;
    let mantissa = bits & 0x007f_ffff;
    if exponent == 0xff {
        return sign | 0x7c00 | if mantissa == 0 { 0 } else { 0x0200 };
    }
    let rebased = exponent - 127 + 15;
    if rebased >= 0x1f {
        return sign | 0x7c00;
    }
    if rebased <= 0 {
        if rebased < -10 {
            return sign;
        }
        let mantissa = mantissa | 0x0080_0000;
        let shift = (14 - rebased) as u32;
        let half = mantissa >> shift;
        let round = (mantissa >> (shift - 1)) & 1;
        let sticky = mantissa & ((1 << (shift - 1)) - 1);
        let rounded = half + u32::from(round == 1 && (sticky != 0 || half & 1 == 1));
        return sign | rounded as u16;
    }
    let half = ((rebased as u32) << 10) | (mantissa >> 13);
    let round = (mantissa >> 12) & 1;
    let sticky = mantissa & 0x0fff;
    let rounded = half + u32::from(round == 1 && (sticky != 0 || half & 1 == 1));
    sign | rounded as u16
}

/// Quantizes a `[rows, columns]` matrix to 8 bits with one symmetric scale per column or per row,
/// choosing the axis with the smaller squared error. Returns the integers, the scales and the
/// shape of the scale tensor.
fn quantize(values: &[f32], rows: usize, columns: usize) -> (Vec<i8>, Vec<f32>, Vec<usize>) {
    let scales_for = |channels: usize, channel_of: &dyn Fn(usize) -> usize| {
        let mut largest = vec![0.0_f32; channels];
        for (index, value) in values.iter().enumerate() {
            let largest = &mut largest[channel_of(index)];
            *largest = largest.max(value.abs());
        }
        largest
            .into_iter()
            .map(|largest| if largest == 0.0 { 1.0 } else { largest / 127.0 })
            .collect::<Vec<_>>()
    };
    let by_column = |index: usize| index % columns;
    let by_row = |index: usize| index / columns;
    let column_scales = scales_for(columns, &by_column);
    let row_scales = scales_for(rows, &by_row);
    let round = |value: f32, scale: f32| (value / scale).round().clamp(-127.0, 127.0) as i8;
    let error = |scales: &[f32], channel_of: &dyn Fn(usize) -> usize| {
        values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let scale = scales[channel_of(index)];
                let restored = f32::from(round(*value, scale)) * scale;
                f64::from(value - restored).powi(2)
            })
            .sum::<f64>()
    };
    if error(&column_scales, &by_column) <= error(&row_scales, &by_row) {
        let integers = values
            .iter()
            .enumerate()
            .map(|(index, value)| round(*value, column_scales[by_column(index)]))
            .collect();
        (integers, column_scales, vec![1, columns])
    } else {
        let integers = values
            .iter()
            .enumerate()
            .map(|(index, value)| round(*value, row_scales[by_row(index)]))
            .collect();
        (integers, row_scales, vec![rows, 1])
    }
}

/// A tensor ready to be written: its name, element type, shape and bytes.
struct Stored {
    name: String,
    dtype: Dtype,
    shape: Vec<usize>,
    data: Vec<u8>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = env::args().collect::<Vec<_>>();
    let (int8, paths) = match arguments.as_slice() {
        [_, flag, paths @ ..] if flag == "--int8" => (true, paths),
        [_, paths @ ..] => (false, paths),
        [] => (false, &arguments[..]),
    };
    let [input, output] = paths else {
        return Err("usage: convert-checkpoint [--int8] <input.bpk> <output.safetensors>".into());
    };
    let reader = Reader::from_bytes(Bytes::from_bytes_vec(std::fs::read(input)?))?;
    let mut stored = Vec::new();
    for tensor in reader.into_tensors()? {
        let (name, dtype, shape, _, data) = tensor.into_parts()?;
        if dtype != DType::F32 {
            return Err(format!("tensor {name} is not 32-bit float").into());
        }
        let dims = shape.as_slice().to_vec();
        let values = data
            .as_chunks::<4>()
            .0
            .iter()
            .map(|chunk| f32::from_le_bytes(*chunk))
            .collect::<Vec<_>>();
        if int8 && dims.len() == 2 && values.len() > 4096 {
            let (integers, scales, scale_shape) = quantize(&values, dims[0], dims[1]);
            stored.push(Stored {
                name: format!("{name}.scale"),
                dtype: Dtype::F32,
                shape: scale_shape,
                data: scales
                    .iter()
                    .flat_map(|scale| scale.to_le_bytes())
                    .collect(),
            });
            stored.push(Stored {
                name,
                dtype: Dtype::I8,
                shape: dims,
                data: integers
                    .iter()
                    .map(|value| value.to_le_bytes()[0])
                    .collect(),
            });
        } else if dims.len() == 2 && values.len() > 4096 {
            let halves = values
                .iter()
                .map(|value| single_to_half(*value))
                .collect::<Vec<_>>();
            let mut planes = halves
                .iter()
                .map(|half| (half >> 8) as u8)
                .collect::<Vec<_>>();
            planes.extend(halves.iter().map(|half| (half & 0xff) as u8));
            let mut planar = vec![2];
            planar.extend(&dims);
            stored.push(Stored {
                name,
                dtype: Dtype::U8,
                shape: planar,
                data: planes,
            });
        } else {
            stored.push(Stored {
                name,
                dtype: Dtype::F32,
                shape: dims,
                data: data.to_vec(),
            });
        }
    }
    let views = stored
        .iter()
        .map(|tensor| {
            TensorView::new(tensor.dtype, tensor.shape.clone(), &tensor.data)
                .map(|view| (tensor.name.clone(), view))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let metadata =
        (!int8).then(|| HashMap::from([(F16_LAYOUT_KEY.to_owned(), F16_PLANES.to_owned())]));
    safetensors::serialize_to_file(views, metadata, Path::new(output))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::single_to_half;

    #[test]
    fn rounds_to_the_nearest_half_with_ties_to_even() {
        assert_eq!(single_to_half(0.0), 0x0000);
        assert_eq!(single_to_half(-0.0), 0x8000);
        assert_eq!(single_to_half(1.0), 0x3c00);
        assert_eq!(single_to_half(-2.5), 0xc100);
        assert_eq!(single_to_half(65_504.0), 0x7bff);
        assert_eq!(single_to_half(1.0e9), 0x7c00); // overflows to infinity
        assert_eq!(single_to_half(5.960_464_5e-8), 0x0001); // smallest subnormal
        assert_eq!(single_to_half(1.0e-9), 0x0000); // underflows to zero
        // Exactly halfway between 1.0 (0x3c00) and the next half: ties to the even mantissa.
        assert_eq!(single_to_half(1.0 + 2.0_f32.powi(-11)), 0x3c00);
        // Halfway between 0x3c01 and 0x3c02 rounds up to the even 0x3c02.
        assert_eq!(single_to_half(1.0 + 3.0 * 2.0_f32.powi(-11)), 0x3c02);
        assert_eq!(single_to_half(f32::INFINITY), 0x7c00);
        assert_eq!(single_to_half(f32::NAN) & 0x7c00, 0x7c00);
    }
}
