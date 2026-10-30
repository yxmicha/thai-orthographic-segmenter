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
