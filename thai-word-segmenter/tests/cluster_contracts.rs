use std::{fs, io::Cursor, path::Path};
use thai_word_segmenter::{
    Error, WordSegmenter,
    decoding::{BoundaryScores, SourceText},
    dictionary::ClusterDictionary,
    encoding::EncodedBatch,
    model::ClusterModelConfig,
    network::{ClusterInput, ClusterModel},
};

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

#[test]
fn dictionary_protects_initial_and_final_forms_without_attaching_the_next_word() -> Result<(), Error>
{
    let dictionary = ClusterDictionary::read(Cursor::new("การเรียน\nบริการ\nความสุข\nข้อความ\n"))?;
    for parts in [
        vec!["การ", "เรียน", "ดี"],
        vec!["บริ", "การ", "ดี"],
        vec!["ความ", "สุข", "ดี"],
        vec!["ข้อ", "ความ", "ดี"],
    ] {
        let syllables = parts.into_iter().map(str::to_owned).collect::<Vec<_>>();
        let breaks =
            BoundaryScores::new(&[0.99, 0.99, 0.99], 0.5).select_breaks(&syllables, &dictionary)?;
        assert_eq!(breaks, [false, true, false]);
    }
    Ok(())
}
