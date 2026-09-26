use crate::Error;

/// Largest Thai run the position embeddings and lattice search are sized for.
pub(crate) const INPUT_SCALAR_LIMIT: usize = 4096;
/// Largest Thai candidate span the lattice may consider.
const SPAN_SCALAR_LIMIT: usize = 128;

/// Search bounds and soft evidence weights of the trained model.
#[derive(Clone, Debug)]
pub(crate) struct LatticeConfig {
    /// Maximum Thai run length; longer input is rejected without truncation.
    pub(crate) max_input_scalars: usize,
    /// Maximum Thai candidate length. Gold coverage reports this search limit.
    pub(crate) max_span_scalars: usize,
    /// Cost for every Thai candidate (a tunable prior on syllable count).
    pub(crate) syllable_cost: f32,
    /// Cost when the rules do not recognize a complete candidate.
    pub(crate) unsupported_cost: f32,
    /// Cost per extra recognized vowel nucleus.
    pub(crate) extra_nucleus_cost: f32,
    /// Cost for cutting inside a flat rule anchor.
    pub(crate) anchor_cut_cost: f32,
}

impl LatticeConfig {
    /// Rejects unsafe resource bounds and non-finite or negative penalties.
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if !(1..=INPUT_SCALAR_LIMIT).contains(&self.max_input_scalars)
            || !(1..=SPAN_SCALAR_LIMIT).contains(&self.max_span_scalars)
        {
            return Err(Error::Invariant(
                "input or span scalar limit is outside the supported range",
            ));
        }
        let weights = [
            self.syllable_cost,
            self.unsupported_cost,
            self.extra_nucleus_cost,
            self.anchor_cut_cost,
        ];
        if weights
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(Error::Invariant(
                "evidence weights must be finite and nonnegative",
            ));
        }
        Ok(())
    }
}
