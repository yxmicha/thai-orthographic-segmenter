//! Public types are `Send + Sync` and `Debug`, and their `Debug` output is never empty.

use thai_syllable_segmenter::{
    Error, SyllableSegmenter,
    inference::{
        AttentionWeights, BlockPool, EmbeddingTable, EncoderLayer, FeedForwardWeights, InputBlocks,
        LayerNormWeights, PackedMatrix, Scratch, ShapeError, WeightMatrix,
    },
};

fn assert_send_sync_debug<T: Send + Sync + std::fmt::Debug>() {}

#[test]
fn public_types_are_thread_safe_and_inspectable() {
    assert_send_sync_debug::<SyllableSegmenter>();
    assert_send_sync_debug::<Error>();
    assert_send_sync_debug::<ShapeError>();
    assert_send_sync_debug::<LayerNormWeights>();
    assert_send_sync_debug::<AttentionWeights>();
    assert_send_sync_debug::<FeedForwardWeights>();
    assert_send_sync_debug::<EncoderLayer>();
    assert_send_sync_debug::<PackedMatrix>();
    assert_send_sync_debug::<Scratch>();
    assert_send_sync_debug::<InputBlocks>();
    assert_send_sync_debug::<BlockPool>();
    assert_send_sync_debug::<WeightMatrix>();
    assert_send_sync_debug::<EmbeddingTable>();
}

#[test]
fn debug_output_is_never_empty() -> Result<(), Error> {
    assert_ne!(format!("{:?}", SyllableSegmenter::new()?), "");
    assert_ne!(format!("{:?}", InputBlocks::default()), "");
    assert_ne!(format!("{:?}", BlockPool::default()), "");
    Ok(())
}
