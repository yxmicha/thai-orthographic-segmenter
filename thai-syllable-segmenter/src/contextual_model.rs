#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

use crate::{
    Error,
    inference::{
        Activations, BlockPool, Checkpoint, EncoderLayer, PackedMatrix, Scratch, ShapeError,
        WeightMatrix, run_on_cpu,
    },
    lattice::{CandidateKind, RelaxedLattice},
    model_config::ModelConfig,
    vocabulary::CharacterVocabulary,
};

/// Thai-run Transformer scoring over the relaxed candidate space, run on the shared CPU engine.
/// Combines local n-gram embeddings and rule features with run-wide attention.
pub(crate) struct ContextualModel {
    characters: Vec<f32>,
    ngrams: Vec<f32>,
    kinds: Vec<f32>,
    starts: Vec<f32>,
    ends: Vec<f32>,
    positions: Vec<f32>,
    embedding_dimension: usize,
    position_capacity: usize,
    projection: PackedMatrix,
    layers: Vec<EncoderLayer>,
    /// Start logits in columns 0 and 1, end logits in columns 2 and 3.
    heads: PackedMatrix,
    vocabulary: CharacterVocabulary,
    blocks: BlockPool,
}
