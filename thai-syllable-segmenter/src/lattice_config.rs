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
