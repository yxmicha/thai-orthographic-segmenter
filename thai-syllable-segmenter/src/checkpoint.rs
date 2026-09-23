use crate::{
    Error,
    contextual_model::ContextualModel,
    inference::Checkpoint,
    lattice_config::{INPUT_SCALAR_LIMIT, LatticeConfig},
    model_config::ModelConfig,
    vocabulary::CharacterVocabulary,
};

const WEIGHTS: &[u8] = include_bytes!("../model/weights.safetensors");
const LATTICE: LatticeConfig = LatticeConfig {
    max_input_scalars: INPUT_SCALAR_LIMIT,
    max_span_scalars: 48,
    syllable_cost: 0.25,
    unsupported_cost: 2.0,
    extra_nucleus_cost: 4.0,
    anchor_cut_cost: 0.5,
};
