/// Dimensions of the bundled contextual network.
pub(crate) struct ModelConfig {
    pub(crate) character_vocabulary_size: usize,
    pub(crate) position_vocabulary_size: usize,
    pub(crate) embedding_dimension: usize,
    pub(crate) model_dimension: usize,
    pub(crate) feed_forward_dimension: usize,
    pub(crate) attention_heads: usize,
    pub(crate) transformer_layers: usize,
    pub(crate) dropout: f64,
    pub(crate) vocabulary_fingerprint: u64,
}
