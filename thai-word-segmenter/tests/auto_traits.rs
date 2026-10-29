//! Public types are `Send + Sync` and `Debug`, and their `Debug` output is never empty.

use thai_word_segmenter::{
    Error, WordSegmenter,
    decoding::{BoundaryScores, SourceText},
    dictionary::ClusterDictionary,
    encoding::SyllableFeatures,
    inference::InferenceModel,
    model::ClusterModelConfig,
};

fn assert_send_sync_debug<T: Send + Sync + std::fmt::Debug>() {}

#[test]
fn public_types_are_thread_safe_and_inspectable() {
    assert_send_sync_debug::<WordSegmenter>();
    assert_send_sync_debug::<Error>();
    assert_send_sync_debug::<BoundaryScores<'static>>();
    assert_send_sync_debug::<SourceText<'static>>();
    assert_send_sync_debug::<ClusterDictionary>();
    assert_send_sync_debug::<SyllableFeatures<'static>>();
    assert_send_sync_debug::<InferenceModel>();
    assert_send_sync_debug::<ClusterModelConfig>();
}

#[test]
fn debug_output_is_never_empty() -> Result<(), Error> {
    assert_ne!(format!("{:?}", WordSegmenter::new()?), "");
    Ok(())
}

#[cfg(feature = "burn")]
#[test]
fn burn_pipeline_types_are_thread_safe_and_inspectable() {
    use thai_word_segmenter::{
        encoding::EncodedBatch,
        network::{ClusterInput, ClusterModel},
    };

    assert_send_sync_debug::<EncodedBatch>();

    assert_send_sync_debug::<ClusterInput>();
    assert_send_sync_debug::<ClusterModel>();
}
