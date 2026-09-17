# thai-syllable-segmenter

Source-preserving Thai syllable segmentation. It combines the candidates from
[`thai-syllable-rules`](https://docs.rs/thai-syllable-rules) with a small contextual Transformer
and picks the best complete segmentation with a dynamic-programming decoder. It runs on the
CPU with bundled weights: no model download, CUDA runtime or lexicon.

## Installation

```sh
cargo add thai-syllable-segmenter
```

## Usage

```rust
use thai_syllable_segmenter::SyllableSegmenter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let segmenter = SyllableSegmenter::new()?;
    let pieces = segmenter.split("ฉันกินข้าว สวัสดี!")?;
    assert_eq!(pieces, ["ฉัน", "กิน", "ข้าว", " ", "สวัส", "ดี", "!"]);
    Ok(())
}
```

Construct the segmenter once and reuse it. `split` returns borrowed slices whose
concatenation is the input. Empty input gives an empty vector.

## How it works

- Text is cut into maximal runs accepted by `ThaiScalar::is_accepted` from `thai-syllable-rules`. Every
  other run (Latin, digits, whitespace, punctuation, the baht sign) is returned unchanged.
  Each Thai run is segmented independently, with no context across the spans between runs.
- Within a run, a leading vowel is grouped with its following base and dependent marks stay
  with the preceding base. Candidate boundaries fall only between these cells, so a mark is
  never separated from its base.
- Candidates are spans of consecutive cells up to 48 scalars. The rules' parts guide the
  model but do not fix the answer: a candidate may split or merge them. Rule disagreement is
  a soft cost (0.25 for every candidate, 2.0 when the rules do not recognize a complete
  one, 4.0 per extra vowel nucleus, 0.5 for cutting inside a rule anchor), not a constraint.
- A 2-layer, 4-head Transformer encoder (width 64) over character embeddings, overlapping
  character bigrams and trigrams, and rule features scores each candidate's start and end.
  The decoder returns the best-scoring complete path. It returns no confidence score and no
  alternative paths.
- The network runs on this crate's own CPU inference code, public as
  `thai_syllable_segmenter::inference` so that `thai-word-segmenter` can run its Transformer
  on the same primitives: weights packed once for matrix products, pre-norm encoder layers over
  stacked windows, and AVX2 with fused multiply-add chosen at run time when the CPU has them.
  The weights are a safetensors file, which `inference::Checkpoint` reads directly; no
  deep-learning framework is a dependency.
- Failures are `thai_syllable_segmenter::Error`: `ThaiRunTooLong` for a Thai run over 4,096
  scalars (input is never truncated), `CellTooLong` for one written cell over the 48-scalar
  span limit, plus weight, tensor and invariant errors.

## Training

The bundled model was trained by rules-aware structured distillation: one compact student
learns the path distribution of a larger ensemble while keeping supervised training on
annotated orthographic paths. The objective combines edge marginals and complete-path
cross-entropy over the lattice with negative log-likelihood on annotated paths.

The crate ships frozen weights; the training pipeline is not part of it.

