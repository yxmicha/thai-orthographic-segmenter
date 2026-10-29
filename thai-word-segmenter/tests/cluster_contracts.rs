use std::{fs, io::Cursor, path::Path};

use burn::tensor::Device;
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

#[test]
fn overlapping_dictionary_entries_do_not_chain_into_a_larger_cluster() -> Result<(), Error> {
    let dictionary = ClusterDictionary::read(Cursor::new("การเรียน\nเรียนรู้\n"))?;
    let syllables = ["การ", "เรียน", "รู้"].map(str::to_owned);
    assert_eq!(dictionary.protected_gaps(&syllables), [true, false, false]);
    Ok(())
}

#[test]
fn punctuation_attachment_preserves_spaces_quotes_and_original_text() -> Result<(), Error> {
    for (parts, expected) in [
        (
            vec!["“", "การเรียน", "” ", "ดี", "!"],
            vec!["“การเรียน” ", "ดี!"],
        ),
        (vec!["\" ", "คำ", " \""], vec!["\" คำ \""]),
    ] {
        let text = parts.concat();
        let mut cursor = 0;
        let ranges = parts
            .iter()
            .map(|part| {
                let start = cursor;
                cursor += part.len();
                start..cursor
            })
            .collect::<Vec<_>>();
        let attached = SourceText::new(&text).attach_punctuation(&ranges)?;
        let pieces = attached
            .into_iter()
            .map(|range| &text[range])
            .collect::<Vec<_>>();
        assert_eq!(pieces.concat(), text);
        assert_eq!(pieces, expected);
    }
    assert!(
        SourceText::new("abc")
            .attach_punctuation(&[0..2, std::ops::Range { start: 2, end: 1 }, 1..3])
            .is_err()
    );
    Ok(())
}

#[test]
fn full_precision_checkpoint_roundtrip_runs_source_preserving_inference()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let config = ClusterModelConfig::new()
        .with_model_dimension(32)
        .with_feed_forward_dimension(64)
        .with_heads(4)
        .with_layers(1)
        .with_character_dimension(4)
        .with_syllable_dimension(8)
        .with_max_characters(8)
        .with_syllable_buckets(16)
        .with_dropout(0.0);
    let device = Device::flex();
    let model = ClusterModel::new(&config, &device)?;
    model.save(directory.path().join("model.bpk"))?;
    burnpack_to_safetensors(
        &directory.path().join("model.bpk"),
        &directory.path().join("model.safetensors"),
    )?;
    fs::write(
        directory.path().join("config.json"),
        serde_json::to_string(&config)?,
    )?;
    fs::write(directory.path().join("dictionary.txt"), "การเรียน\nบริการ\n")?;
    let restored = ClusterModel::load(&config, directory.path().join("model.bpk"), &device)?;
    let syllables = ["การ".to_owned(), "เรียน".to_owned()];
    let encoded = ClusterInput::new(EncodedBatch::new(&[&syllables], &config)?, &device);
    let before = model
        .forward(encoded.clone())
        .into_data()
        .try_to_vec::<f32>()?;
    let after = restored.forward(encoded).into_data().try_to_vec::<f32>()?;
    assert!(
        before
            .iter()
            .zip(&after)
            .all(|(left, right)| (left - right).abs() < 1.0e-6),
        "before={before:?} after={after:?}"
    );
    let segmenter = WordSegmenter::load(directory.path())?;
    let text = "“การเรียน” ดี! English 123 และบริการ";
    assert_eq!(segmenter.split(text)?.concat(), text);
    assert_eq!(segmenter.split("")?, [] as [&str; 0]);

    assert!(matches!(
        WordSegmenter::with_dictionary("การเรียน\nabc\n"),
        Err(Error::DictionaryEntry(entry)) if entry == "abc"
    ));
    Ok(())
}
