//! CPU Transformer inference primitives: weights packed once at load time, runtime selection of
//! fused multiply-add instructions, and pre-norm encoder layers over stacked variable-length
//! windows. `thai-word-segmenter` reuses them for its contextual model.

mod checkpoint;
mod cpu;
mod encoder_layer;
mod numerics;
mod packed_matrix;
mod weights;

pub use checkpoint::{Checkpoint, CheckpointError, F16_LAYOUT_KEY, F16_PLANES};
pub use cpu::run_on_cpu;
pub use encoder_layer::{EncoderLayer, Scratch};
pub use numerics::{Activations, AttentionRow, Values};
pub use packed_matrix::{BlockPool, InputBlocks, PackedMatrix, ShapeError, multiply_add};
pub use weights::{
    AttentionWeights, EmbeddingTable, FeedForwardWeights, LayerNormWeights, WeightMatrix,
};
