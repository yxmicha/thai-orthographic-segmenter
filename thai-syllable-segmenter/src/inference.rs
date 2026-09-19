//! CPU Transformer inference primitives: weights packed once at load time, runtime selection of
//! fused multiply-add instructions, and pre-norm encoder layers over stacked variable-length
//! windows. `thai-word-segmenter` reuses them for its contextual model.

mod checkpoint;
mod numerics;
mod packed_matrix;
mod weights;
