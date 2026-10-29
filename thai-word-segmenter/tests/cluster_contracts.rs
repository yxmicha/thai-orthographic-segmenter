use std::{fs, io::Cursor, path::Path};

/// Rewrites a Burn burnpack of 32-bit tensors as the safetensors file `WordSegmenter::load` reads.
fn burnpack_to_safetensors(
    burnpack: &Path,
    safetensors: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let reader =
        burn_pack::Reader::from_bytes(burn_pack::Bytes::from_bytes_vec(fs::read(burnpack)?))?;
    let mut stored = Vec::new();
    for tensor in reader.into_tensors()? {
        let (name, _, shape, _, data) = tensor.into_parts()?;
        stored.push((name, shape.as_slice().to_vec(), data.to_vec()));
    }
    let views = stored
        .iter()
        .map(|(name, shape, data)| {
            ::safetensors::tensor::TensorView::new(::safetensors::Dtype::F32, shape.clone(), data)
                .map(|view| (name.clone(), view))
        })
        .collect::<Result<Vec<_>, _>>()?;
    ::safetensors::serialize_to_file(views, None, safetensors)?;
    Ok(())
}
