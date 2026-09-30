#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

use std::ops::RangeInclusive;

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

/// Embedding row of a scalar inside a rule syllable, and of any other scalar.
const SYLLABLE_KIND: usize = 0;
const OTHER_KIND: usize = 1;
/// Embedding rows of a rule-part boundary flag that is unset and set.
const UNMARKED: usize = 0;
const MARKED: usize = 1;
/// Logit columns of the fused heads: each head emits a `[not boundary, boundary]` pair.
const START_BOUNDARY_COLUMN: usize = 1;
const END_BOUNDARY_COLUMN: usize = 3;
/// Logits emitted per scalar by the fused heads.
const LOGITS_PER_SCALAR: usize = 4;

/// Neighbouring characters embedded around each scalar.
const CHARACTER_OFFSETS: RangeInclusive<i8> = -2..=2;

/// Concatenated embeddings per scalar: five character offsets, three offsets for each of the
/// bigram and trigram features, and the rule-part kind, start, end, and position embeddings.
const INPUT_PIECES: usize = 5 + 2 * 3 + 4;

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

impl ContextualModel {
    /// Reads the weights of a network trained on `vocabulary` from `checkpoint`.
    pub(crate) fn new(
        checkpoint: &mut Checkpoint,
        config: &ModelConfig,
        vocabulary: CharacterVocabulary,
    ) -> Result<Self, Error> {
        config.validate()?;
        if config.character_vocabulary_size != vocabulary.character_rows()
            || config.vocabulary_fingerprint != vocabulary.fingerprint()
        {
            return Err(Error::Invariant(
                "model and sentence vocabulary do not match",
            ));
        }
        let dimension = config.embedding_dimension;
        let heads = WeightMatrix::concatenate_outputs(&[
            checkpoint.linear("start_head", config.model_dimension, 2)?,
            checkpoint.linear("end_head", config.model_dimension, 2)?,
        ])?;
        let characters = checkpoint
            .embedding("characters", vocabulary.character_rows(), dimension)?
            .into_values();
        let ngrams = checkpoint
            .embedding("ngrams", vocabulary.ngrams.len() + 1, dimension)?
            .into_values();
        let kinds = checkpoint.embedding("kinds", 2, dimension)?.into_values();
        let starts = checkpoint.embedding("starts", 2, dimension)?.into_values();
        let ends = checkpoint.embedding("ends", 2, dimension)?.into_values();
        let positions = checkpoint
            .embedding("positions", config.position_vocabulary_size, dimension)?
            .into_values();
        let projection = checkpoint.linear(
            "projection",
            dimension * INPUT_PIECES,
            config.model_dimension,
        )?;
        let layers = (0..config.transformer_layers)
            .map(|index| {
                checkpoint.encoder_layer(
                    &format!("encoder.layers.{index}"),
                    config.model_dimension,
                    config.feed_forward_dimension,
                    config.attention_heads,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            characters,
            ngrams,
            kinds,
            starts,
            ends,
            positions,
            embedding_dimension: dimension,
            position_capacity: config.position_vocabulary_size,
            projection: projection.try_into()?,
            layers,
            heads: heads.try_into()?,
            blocks: BlockPool::default(),
            vocabulary,
        })
    }
}
