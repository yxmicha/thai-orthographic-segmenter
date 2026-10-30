use burn::tensor::Device;
use thai_word_segmenter::{
    Error,
    encoding::EncodedBatch,
    inference::InferenceModel,
    model::ClusterModelConfig,
    network::{ClusterInput, ClusterModel},
};

fn small_config() -> ClusterModelConfig {
    ClusterModelConfig::new()
        .with_model_dimension(32)
        .with_feed_forward_dimension(64)
        .with_heads(4)
        .with_layers(2)
        .with_character_dimension(4)
        .with_syllable_dimension(8)
        .with_max_characters(6)
        .with_syllable_buckets(64)
        .with_dropout(0.0)
}

#[test]
fn inference_model_matches_burn_forward_for_stacked_windows() -> Result<(), Error> {
    let config = small_config();
    let device = Device::flex();
    let model = ClusterModel::new(&config, &device)?;
    let inference = InferenceModel::new(model.clone(), &config)?;
    // Mixed lengths, a syllable longer than the six character slots, and a non-Thai syllable.
    let windows: [Vec<&str>; 3] = [
        vec!["การ", "เรียน", "สำคัญมากเกินกว่าจะเชื่อ", "ดี", "abc"],
        vec!["ฉัน", "กิน"],
        (0..40).map(|_| "ข้าว").collect(),
    ];
    let slices = windows.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let scores = inference.differences(&slices)?;
    assert_eq!(scores.len(), windows.len());
    for (window, scores) in windows.iter().zip(&scores) {
        let owned = window
            .iter()
            .map(|text| (*text).to_owned())
            .collect::<Vec<_>>();
        let logits = model
            .forward(ClusterInput::new(
                EncodedBatch::new(&[&owned], &config)?,
                &device,
            ))
            .into_data()
            .try_to_vec::<f32>()
            .map_err(|error| Error::CheckpointLoad(Box::new(error)))?;
        assert_eq!(scores.len(), window.len());
        for (score, [keep, break_logit]) in scores.iter().zip(logits.as_chunks::<2>().0) {
            let expected = keep - break_logit;
            assert!(
                (score - expected).abs() < 1.0e-4,
                "{score} differs from {expected}"
            );
        }
    }
    Ok(())
}
