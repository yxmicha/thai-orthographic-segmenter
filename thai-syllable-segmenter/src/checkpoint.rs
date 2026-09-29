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

/// The bundled model and the lattice settings it was trained with.
pub(crate) struct BundledModel {
    pub(crate) model: ContextualModel,
    pub(crate) lattice: LatticeConfig,
}

impl BundledModel {
    /// Loads the bundled vocabulary and weights.
    pub(crate) fn load() -> Result<Self, Error> {
        let vocabulary = CharacterVocabulary::bundled()?;
        LATTICE.validate()?;
        let config = ModelConfig {
            character_vocabulary_size: vocabulary.character_rows(),
            position_vocabulary_size: LATTICE.max_input_scalars,
            embedding_dimension: 16,
            model_dimension: 64,
            feed_forward_dimension: 128,
            attention_heads: 4,
            transformer_layers: 2,
            dropout: 0.0,
            vocabulary_fingerprint: vocabulary.fingerprint(),
        };
        let mut checkpoint = Checkpoint::from_bytes(WEIGHTS)?;
        let model = ContextualModel::new(&mut checkpoint, &config, vocabulary)?;
        checkpoint.finish()?;
        Ok(Self {
            model,
            lattice: LATTICE,
        })
    }
}
