use crate::Error;

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

impl ModelConfig {
    /// Rejects zero dimensions, indivisible attention heads, invalid dropout, and a missing
    /// vocabulary fingerprint.
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if [
            self.character_vocabulary_size,
            self.position_vocabulary_size,
            self.embedding_dimension,
            self.model_dimension,
            self.feed_forward_dimension,
            self.attention_heads,
            self.transformer_layers,
        ]
        .contains(&0)
            || !self.model_dimension.is_multiple_of(self.attention_heads)
            || !self.dropout.is_finite()
            || !(0.0..1.0).contains(&self.dropout)
            || self.vocabulary_fingerprint == 0
        {
            return Err(Error::Invariant("invalid distilled model dimensions"));
        }
        Ok(())
    }
}
